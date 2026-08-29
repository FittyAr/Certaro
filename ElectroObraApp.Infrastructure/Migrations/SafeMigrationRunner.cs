using System.Reflection;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Helpers;
using ElectroObraApp.Infrastructure.Data;
using ElectroObraApp.Infrastructure.Services;

namespace ElectroObraApp.Infrastructure.Migrations;

public sealed class SafeMigrationRunner : IMigrationRunner
{
    private readonly ApplicationDbContext _context;
    private readonly IBackupService _backupService;
    private readonly DatabaseCensusService _censusService;
    private readonly IConfiguration _configuration;
    private readonly ILogger<SafeMigrationRunner> _logger;

    public SafeMigrationRunner(
        ApplicationDbContext context,
        IBackupService backupService,
        DatabaseCensusService censusService,
        IConfiguration configuration,
        ILogger<SafeMigrationRunner> logger)
    {
        _context = context;
        _backupService = backupService;
        _censusService = censusService;
        _configuration = configuration;
        _logger = logger;
    }

    public async Task<MigrationRunResult> RunPendingMigrationsAsync(CancellationToken cancellationToken = default)
    {
        var connectionString = PathHelper.GetSqliteConnectionString();
        var pending = await GetPendingMigrationsAsync(cancellationToken);

        if (pending.Count == 0)
        {
            await EnsureSchemaVersionStampedAsync(cancellationToken);
            return new MigrationRunResult { Success = true, MigrationsApplied = false };
        }

        var downgradeError = await CheckDowngradeAsync(cancellationToken);
        if (downgradeError is not null)
        {
            return new MigrationRunResult { Success = false, ErrorMessage = downgradeError };
        }

        string? backupPath = null;
        DatabaseCensus? censusBefore = null;

        try
        {
            if (_configuration.GetValue("Application:Migration:BackupEnabled", true))
            {
                backupPath = await _backupService.CreateBackupAsync(cancellationToken);
                await _backupService.CleanupOldBackupsAsync(cancellationToken);
            }

            censusBefore = await _censusService.CollectAsync(connectionString, cancellationToken);

            _logger.LogInformation("Aplicando {Count} migraciones pendientes...", pending.Count);
            await _context.Database.MigrateAsync(cancellationToken);

            if (!await VerifyForeignKeysAsync(connectionString, cancellationToken))
            {
                throw new InvalidOperationException("La verificación de claves foráneas falló después de la migración.");
            }

            var censusAfter = await _censusService.CollectAsync(connectionString, cancellationToken);
            if (censusBefore is not null && !DatabaseCensusService.VerifyPreservation(censusBefore, censusAfter, out var failureReason))
            {
                throw new InvalidOperationException(failureReason);
            }

            await EnsureSchemaVersionStampedAsync(cancellationToken);

            return new MigrationRunResult
            {
                Success = true,
                MigrationsApplied = true,
                BackupPath = backupPath,
                AppliedMigrations = pending
            };
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error durante la migración segura.");

            if (!string.IsNullOrEmpty(backupPath))
            {
                try
                {
                    await _backupService.RestoreFromBackupAsync(backupPath, cancellationToken);
                    _logger.LogWarning("Base de datos restaurada desde backup tras fallo de migración.");
                }
                catch (Exception restoreEx)
                {
                    _logger.LogCritical(restoreEx, "No se pudo restaurar el backup {BackupPath}", backupPath);
                    return new MigrationRunResult
                    {
                        Success = false,
                        BackupPath = backupPath,
                        ErrorMessage = $"{ex.Message} Además, la restauración del backup falló: {restoreEx.Message}"
                    };
                }
            }

            return new MigrationRunResult
            {
                Success = false,
                BackupPath = backupPath,
                ErrorMessage = ex.Message
            };
        }
    }

    public async Task<IReadOnlyList<string>> GetAppliedMigrationsAsync(CancellationToken cancellationToken = default)
    {
        var applied = await _context.Database.GetAppliedMigrationsAsync(cancellationToken);
        return applied.ToList();
    }

    public async Task<IReadOnlyList<string>> GetPendingMigrationsAsync(CancellationToken cancellationToken = default)
    {
        var pending = await _context.Database.GetPendingMigrationsAsync(cancellationToken);
        return pending.ToList();
    }

    public Task<IReadOnlyList<string>> GetBackupFilesAsync(CancellationToken cancellationToken = default)
    {
        return _backupService.ListBackupsAsync(cancellationToken)
            .ContinueWith(t => (IReadOnlyList<string>)t.Result.Select(b => b.FilePath).ToList(), cancellationToken);
    }

    private async Task<string?> CheckDowngradeAsync(CancellationToken cancellationToken)
    {
        if (!await TableExistsAsync("SchemaVersions", cancellationToken))
            return null;

        var stored = await _context.Set<SchemaVersion>().AsNoTracking().FirstOrDefaultAsync(cancellationToken);
        if (stored is null)
            return null;

        var currentVersion = GetAppVersion();
        if (Version.TryParse(stored.AppVersion, out var storedVersion) &&
            Version.TryParse(currentVersion, out var runningVersion) &&
            storedVersion > runningVersion)
        {
            return $"La base de datos fue escrita por una versión más nueva ({stored.AppVersion}). " +
                   $"Actualice la aplicación. Versión actual: {currentVersion}.";
        }

        return null;
    }

    private async Task EnsureSchemaVersionStampedAsync(CancellationToken cancellationToken)
    {
        if (!await TableExistsAsync("SchemaVersions", cancellationToken))
            return;

        var latestMigration = (await GetAppliedMigrationsAsync(cancellationToken)).LastOrDefault() ?? string.Empty;
        var appVersion = GetAppVersion();
        var existing = await _context.Set<SchemaVersion>().FirstOrDefaultAsync(cancellationToken);

        if (existing is null)
        {
            _context.Set<SchemaVersion>().Add(new SchemaVersion
            {
                MigrationId = latestMigration,
                AppVersion = appVersion,
                AppliedAt = DateTime.UtcNow
            });
        }
        else
        {
            existing.MigrationId = latestMigration;
            existing.AppVersion = appVersion;
            existing.AppliedAt = DateTime.UtcNow;
        }

        await _context.SaveChangesAsync(cancellationToken);
    }

    private async Task<bool> TableExistsAsync(string tableName, CancellationToken cancellationToken)
    {
        var connectionString = PathHelper.GetSqliteConnectionString();
        await using var connection = new SqliteConnection(connectionString);
        await connection.OpenAsync(cancellationToken);

        await using var command = connection.CreateCommand();
        command.CommandText = "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = $name;";
        command.Parameters.AddWithValue("$name", tableName);
        var result = await command.ExecuteScalarAsync(cancellationToken);
        return Convert.ToInt32(result) > 0;
    }

    private static async Task<bool> VerifyForeignKeysAsync(string connectionString, CancellationToken cancellationToken)
    {
        await using var connection = new SqliteConnection(connectionString);
        await connection.OpenAsync(cancellationToken);

        await using var command = connection.CreateCommand();
        command.CommandText = "PRAGMA foreign_key_check;";
        await using var reader = await command.ExecuteReaderAsync(cancellationToken);
        return !await reader.ReadAsync(cancellationToken);
    }

    private static string GetAppVersion()
    {
        return Assembly.GetEntryAssembly()?.GetName().Version?.ToString(3) ?? "1.0.0";
    }
}
