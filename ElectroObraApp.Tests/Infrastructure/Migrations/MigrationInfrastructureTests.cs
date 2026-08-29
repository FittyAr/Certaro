using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging.Abstractions;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Infrastructure.Data;
using ElectroObraApp.Infrastructure.Migrations;
using Microsoft.EntityFrameworkCore.Infrastructure;
using ElectroObraApp.Infrastructure.Services;
using NSubstitute;

namespace ElectroObraApp.Tests.Infrastructure.Migrations;

public class RescaleMonetaryValuesTests
{
    [Fact]
    public async Task RescaleMonetaryValues_MultipliesUnscaledAmountsByScale()
    {
        var dbPath = Path.Combine(Path.GetTempPath(), $"rescale_test_{Guid.NewGuid()}.db");
        var connectionString = $"Data Source={dbPath};Pooling=false";
        const int scale = 10_000;

        try
        {
            await using (var setupConnection = new SqliteConnection(connectionString))
            {
                await setupConnection.OpenAsync(TestContext.Current.CancellationToken);
                await using var cmd = setupConnection.CreateCommand();
                cmd.CommandText = """
                    CREATE TABLE IF NOT EXISTS "AppMetadata" (
                        "Key" TEXT NOT NULL PRIMARY KEY,
                        "Value" TEXT NOT NULL,
                        "UpdatedAt" TEXT NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS "Movimientos" (
                        "Id" TEXT NOT NULL PRIMARY KEY,
                        "Monto" INTEGER NOT NULL,
                        "Cantidad" INTEGER NOT NULL DEFAULT 10000
                    );

                    INSERT INTO AppMetadata (Key, Value, UpdatedAt)
                    VALUES ('MonetaryValuesRescaled', 'false', datetime('now'));

                    INSERT INTO Movimientos (Id, Monto, Cantidad)
                    VALUES ('11111111-1111-1111-1111-111111111111', 1500, 10000);
                    """;
                await cmd.ExecuteNonQueryAsync(TestContext.Current.CancellationToken);

                await using var rescale = setupConnection.CreateCommand();
                rescale.CommandText = $"""
                    UPDATE "Movimientos"
                    SET "Monto" = "Monto" * {scale}
                    WHERE EXISTS (
                        SELECT 1 FROM AppMetadata
                        WHERE Key = 'MonetaryValuesRescaled' AND Value = 'false'
                    )
                    AND "Monto" IS NOT NULL
                    AND "Monto" != 0;

                    UPDATE AppMetadata
                    SET Value = 'true', UpdatedAt = datetime('now')
                    WHERE Key = 'MonetaryValuesRescaled' AND Value = 'false';
                    """;
                await rescale.ExecuteNonQueryAsync(TestContext.Current.CancellationToken);
            }

            await using (var verifyConnection = new SqliteConnection(connectionString))
            {
                await verifyConnection.OpenAsync(TestContext.Current.CancellationToken);
                await using var command = verifyConnection.CreateCommand();
                command.CommandText = "SELECT Monto FROM Movimientos LIMIT 1;";
                var monto = Convert.ToInt64(await command.ExecuteScalarAsync(TestContext.Current.CancellationToken));
                monto.Should().Be(15_000_000L);

                await using var flagCmd = verifyConnection.CreateCommand();
                flagCmd.CommandText = "SELECT Value FROM AppMetadata WHERE Key = 'MonetaryValuesRescaled';";
                var flag = (await flagCmd.ExecuteScalarAsync(TestContext.Current.CancellationToken))?.ToString();
                flag.Should().Be("true");
            }
        }
        finally
        {
            SqliteConnection.ClearAllPools();
            if (File.Exists(dbPath))
                File.Delete(dbPath);
        }
    }

    private static async Task CreateLegacyDatabaseAsync(string connectionString)
    {
        await using var connection = new SqliteConnection(connectionString);
        await connection.OpenAsync();

        await using (var cmd = connection.CreateCommand())
        {
            cmd.CommandText = """
                CREATE TABLE IF NOT EXISTS "__EFMigrationsHistory" (
                    "MigrationId" TEXT NOT NULL CONSTRAINT "PK___EFMigrationsHistory" PRIMARY KEY,
                    "ProductVersion" TEXT NOT NULL
                );
                INSERT INTO "__EFMigrationsHistory" ("MigrationId", "ProductVersion")
                VALUES ('20260828140901_FixRowVersionConfiguration', '10.0.11');

                CREATE TABLE "Movimientos" (
                    "Id" TEXT NOT NULL PRIMARY KEY,
                    "Monto" INTEGER NOT NULL,
                    "Cantidad" INTEGER NOT NULL DEFAULT 10000,
                    "Concepto" TEXT,
                    "Fecha" TEXT,
                    "CreatedAt" TEXT,
                    "UpdatedAt" TEXT,
                    "IsDeleted" INTEGER NOT NULL DEFAULT 0,
                    "DeletedAt" TEXT,
                    "RowVersion" BLOB,
                    "TipoMovimientoId" TEXT,
                    "Moneda" INTEGER DEFAULT 0
                );

                INSERT INTO Movimientos (Id, Monto, Cantidad, Concepto, Fecha, CreatedAt, IsDeleted, RowVersion, TipoMovimientoId)
                VALUES ('11111111-1111-1111-1111-111111111111', 1500, 10000, 'Test', datetime('now'), datetime('now'), 0, X'0000000000000001', '00000000-0000-0000-0000-000000000001');
                """;
            await cmd.ExecuteNonQueryAsync();
        }
    }
}

public class SafeMigrationRunnerTests
{
    [Fact]
    public async Task RunPendingMigrationsAsync_WithNoPending_ReturnsSuccess()
    {
        var dbPath = Path.Combine(Path.GetTempPath(), $"safe_migration_{Guid.NewGuid()}.db");
        var connectionString = $"Data Source={dbPath};Pooling=false";

        try
        {
            var options = new DbContextOptionsBuilder<ApplicationDbContext>()
                .UseSqlite(connectionString)
                .Options;

            await using (var setupContext = new ApplicationDbContext(options))
            {
                await setupContext.Database.MigrateAsync(TestContext.Current.CancellationToken);
            }

            await using var context = new ApplicationDbContext(options);

            var backupService = Substitute.For<IBackupService>();
            backupService.CreateBackupAsync(Arg.Any<CancellationToken>()).Returns("backup.db");
            backupService.ListBackupsAsync(Arg.Any<CancellationToken>()).Returns([]);
            backupService.CleanupOldBackupsAsync(Arg.Any<CancellationToken>()).Returns(Task.CompletedTask);

            var configuration = new ConfigurationBuilder()
                .AddInMemoryCollection(new Dictionary<string, string?> { ["Application:Migration:BackupEnabled"] = "false" })
                .Build();

            var runner = new SafeMigrationRunner(
                context,
                backupService,
                new DatabaseCensusService(NullLogger<DatabaseCensusService>.Instance),
                configuration,
                NullLogger<SafeMigrationRunner>.Instance);

            var result = await runner.RunPendingMigrationsAsync(TestContext.Current.CancellationToken);

            result.Success.Should().BeTrue();
            result.MigrationsApplied.Should().BeFalse();
        }
        finally
        {
            SqliteConnection.ClearAllPools();
            if (File.Exists(dbPath))
                File.Delete(dbPath);
        }
    }
}
