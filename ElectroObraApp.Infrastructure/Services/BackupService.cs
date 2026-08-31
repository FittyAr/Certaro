using System.Reflection;
using System.Text.Json;
using System.Text.RegularExpressions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Helpers;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Services;

public sealed class BackupService : IBackupService
{
    private readonly ApplicationDbContext _context;
    private readonly IConfiguration _configuration;
    private readonly ILogger<BackupService> _logger;

    public BackupService(
        ApplicationDbContext context,
        IConfiguration configuration,
        ILogger<BackupService> logger)
    {
        _context = context;
        _configuration = configuration;
        _logger = logger;
    }

    public async Task<string> CreateBackupAsync(CancellationToken cancellationToken = default)
    {
        var backupDirectory = GetBackupDirectory();
        Directory.CreateDirectory(backupDirectory);

        var timestamp = DateTime.UtcNow.ToString("yyyyMMdd_HHmmss");
        var backupPath = Path.Combine(backupDirectory, $"electroobra_{timestamp}.db");

        var connectionString = PathHelper.GetSqliteConnectionString();
        await using var connection = new SqliteConnection(connectionString);
        await connection.OpenAsync(cancellationToken);

        await using var command = connection.CreateCommand();
        command.CommandText = $"VACUUM INTO '{backupPath.Replace("'", "''")}';";
        await command.ExecuteNonQueryAsync(cancellationToken);

        await VerifyIntegrityAsync(backupPath, cancellationToken);
        _logger.LogInformation("Backup creado en {BackupPath}", backupPath);
        return backupPath;
    }

    public async Task RestoreFromBackupAsync(string backupPath, CancellationToken cancellationToken = default)
    {
        if (!File.Exists(backupPath))
            throw new FileNotFoundException("No se encontró el archivo de backup.", backupPath);

        await VerifyIntegrityAsync(backupPath, cancellationToken);

        var databasePath = PathHelper.GetDatabasePath();
        var tempPath = databasePath + ".restore.tmp";

        File.Copy(backupPath, tempPath, overwrite: true);
        File.Copy(tempPath, databasePath, overwrite: true);
        File.Delete(tempPath);

        _logger.LogWarning("Base de datos restaurada desde {BackupPath}", backupPath);
        await Task.CompletedTask;
    }

    public async Task<IReadOnlyList<BackupInfo>> ListBackupsAsync(CancellationToken cancellationToken = default)
    {
        var backupDirectory = GetBackupDirectory();
        if (!Directory.Exists(backupDirectory))
            return [];

        var backups = Directory.GetFiles(backupDirectory, "electroobra_*.db")
            .Select(path => new FileInfo(path))
            .OrderByDescending(f => f.CreationTimeUtc)
            .Select(f => new BackupInfo
            {
                FilePath = f.FullName,
                CreatedAt = f.CreationTimeUtc,
                SizeBytes = f.Length
            })
            .ToList();

        return await Task.FromResult(backups);
    }

    public async Task CleanupOldBackupsAsync(CancellationToken cancellationToken = default)
    {
        var retentionDays = _configuration.GetValue("Application:Migration:BackupRetentionDays", 30);
        var cutoff = DateTime.UtcNow.AddDays(-retentionDays);
        var backups = await ListBackupsAsync(cancellationToken);

        foreach (var backup in backups.Where(b => b.CreatedAt < cutoff))
        {
            try
            {
                File.Delete(backup.FilePath);
                _logger.LogInformation("Backup antiguo eliminado: {Path}", backup.FilePath);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "No se pudo eliminar el backup {Path}", backup.FilePath);
            }
        }
    }

    public async Task<DatabaseExportResult> ExportToJsonAsync(string outputPath, CancellationToken cancellationToken = default)
    {
        try
        {
            var export = new DatabaseJsonExport
            {
                Version = GetAppVersion(),
                ExportedAt = DateTime.UtcNow,
                Tables = new Dictionary<string, List<Dictionary<string, object?>>>(StringComparer.OrdinalIgnoreCase)
            };

            var connectionString = PathHelper.GetSqliteConnectionString();
            await using var connection = new SqliteConnection(connectionString);
            await connection.OpenAsync(cancellationToken);

            var tables = await GetUserTablesAsync(connection, cancellationToken);
            foreach (var table in tables)
            {
                export.Tables[table] = await ReadTableAsync(connection, table, cancellationToken);
            }

            var json = JsonSerializer.Serialize(export, new JsonSerializerOptions { WriteIndented = true });
            await File.WriteAllTextAsync(outputPath, json, cancellationToken);

            return new DatabaseExportResult { Success = true, FilePath = outputPath };
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error al exportar base de datos a JSON");
            return new DatabaseExportResult { Success = false, ErrorMessage = ex.Message };
        }
    }

    public async Task<DatabaseExportResult> ImportFromJsonAsync(string inputPath, CancellationToken cancellationToken = default)
    {
        try
        {
            if (!File.Exists(inputPath))
                return new DatabaseExportResult { Success = false, ErrorMessage = "Archivo no encontrado." };

            var json = await File.ReadAllTextAsync(inputPath, cancellationToken);
            var export = JsonSerializer.Deserialize<DatabaseJsonExport>(json)
                ?? throw new InvalidOperationException("Formato JSON inválido.");

            await _context.Database.ExecuteSqlRawAsync("PRAGMA foreign_keys = OFF;", cancellationToken);

            var tableAllowlist = BuildTableAllowlist();

            foreach (var (table, rows) in export.Tables)
            {
                if (!tableAllowlist.TryGetValue(table, out var allowedColumns))
                {
                    throw new InvalidOperationException($"Tabla no permitida en importación: {table}");
                }

                await _context.Database.ExecuteSqlRawAsync(
                    BuildDeleteTableSql(table),
                    cancellationToken);

                foreach (var row in rows)
                {
                    ValidateRowColumns(row.Keys, allowedColumns);
                    var sql = BuildInsertSql(table, row.Keys);
                    var dbParams = row.Values.Select((v, i) => new SqliteParameter("@p" + i, v ?? DBNull.Value)).ToArray();
                    await _context.Database.ExecuteSqlRawAsync(sql, dbParams, cancellationToken);
                }
            }

            await _context.Database.ExecuteSqlRawAsync("PRAGMA foreign_keys = ON;", cancellationToken);
            return new DatabaseExportResult { Success = true, FilePath = inputPath };
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error al importar base de datos desde JSON");
            return new DatabaseExportResult { Success = false, ErrorMessage = ex.Message };
        }
    }

    private Dictionary<string, HashSet<string>> BuildTableAllowlist()
    {
        var allowlist = new Dictionary<string, HashSet<string>>(StringComparer.OrdinalIgnoreCase);

        foreach (var entityType in _context.Model.GetEntityTypes())
        {
            var tableName = entityType.GetTableName();
            if (string.IsNullOrWhiteSpace(tableName))
            {
                continue;
            }

            var columns = entityType.GetProperties()
                .Select(p => p.GetColumnName())
                .Where(name => !string.IsNullOrWhiteSpace(name))
                .Select(name => name!)
                .ToHashSet(StringComparer.OrdinalIgnoreCase);

            allowlist[tableName] = columns;
        }

        return allowlist;
    }

    private static void ValidateRowColumns(IEnumerable<string> columns, HashSet<string> allowedColumns)
    {
        foreach (var column in columns)
        {
            if (!allowedColumns.Contains(column))
            {
                throw new InvalidOperationException($"Columna no permitida en importación: {column}");
            }
        }
    }

    private static string BuildDeleteTableSql(string tableName)
    {
        ValidateSqlIdentifier(tableName);
        return "DELETE FROM \"" + tableName + "\";";
    }

    private static string BuildInsertSql(string tableName, IEnumerable<string> columnNames)
    {
        ValidateSqlIdentifier(tableName);

        var columns = columnNames.ToList();
        foreach (var column in columns)
        {
            ValidateSqlIdentifier(column);
        }

        var quotedColumns = string.Join(", ", columns.Select(c => "\"" + c + "\""));
        var parameters = string.Join(", ", columns.Select((_, i) => "@p" + i));
        return "INSERT INTO \"" + tableName + "\" (" + quotedColumns + ") VALUES (" + parameters + ");";
    }

    private static void ValidateSqlIdentifier(string identifier)
    {
        if (!Regex.IsMatch(identifier, "^[A-Za-z_][A-Za-z0-9_]*$"))
        {
            throw new InvalidOperationException($"Identificador SQL no válido: {identifier}");
        }
    }

    private string GetBackupDirectory()
    {
        var configured = _configuration["Application:Migration:BackupDirectory"];
        return string.IsNullOrWhiteSpace(configured)
            ? Path.Combine(PathHelper.GetAppDataPath(), "Backups")
            : Path.Combine(PathHelper.GetAppDataPath(), configured);
    }

    private static string GetAppVersion()
    {
        return Assembly.GetEntryAssembly()?.GetName().Version?.ToString(3) ?? "1.0.0";
    }

    private static async Task VerifyIntegrityAsync(string databasePath, CancellationToken cancellationToken)
    {
        await using var connection = new SqliteConnection($"Data Source={databasePath}");
        await connection.OpenAsync(cancellationToken);

        await using var command = connection.CreateCommand();
        command.CommandText = "PRAGMA integrity_check;";
        var result = (await command.ExecuteScalarAsync(cancellationToken))?.ToString();
        if (!string.Equals(result, "ok", StringComparison.OrdinalIgnoreCase))
            throw new InvalidOperationException($"Integridad del backup fallida: {result}");
    }

    private static async Task<List<string>> GetUserTablesAsync(SqliteConnection connection, CancellationToken cancellationToken)
    {
        var tables = new List<string>();
        await using var command = connection.CreateCommand();
        command.CommandText = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '__EF%' ORDER BY name;";
        await using var reader = await command.ExecuteReaderAsync(cancellationToken);
        while (await reader.ReadAsync(cancellationToken))
            tables.Add(reader.GetString(0));
        return tables;
    }

    private static async Task<List<Dictionary<string, object?>>> ReadTableAsync(
        SqliteConnection connection,
        string table,
        CancellationToken cancellationToken)
    {
        var rows = new List<Dictionary<string, object?>>();
        await using var command = connection.CreateCommand();
        command.CommandText = $"SELECT * FROM \"{table}\";";
        await using var reader = await command.ExecuteReaderAsync(cancellationToken);
        while (await reader.ReadAsync(cancellationToken))
        {
            var row = new Dictionary<string, object?>(StringComparer.OrdinalIgnoreCase);
            for (var i = 0; i < reader.FieldCount; i++)
                row[reader.GetName(i)] = reader.IsDBNull(i) ? null : reader.GetValue(i);
            rows.Add(row);
        }

        return rows;
    }

    private sealed class DatabaseJsonExport
    {
        public string Version { get; set; } = string.Empty;
        public DateTime ExportedAt { get; set; }
        public Dictionary<string, List<Dictionary<string, object?>>> Tables { get; set; } = new(StringComparer.OrdinalIgnoreCase);
    }
}
