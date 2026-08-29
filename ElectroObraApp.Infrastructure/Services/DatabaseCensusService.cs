using Microsoft.Data.Sqlite;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Infrastructure.Migrations;

namespace ElectroObraApp.Infrastructure.Services;

public sealed class DatabaseCensus
{
    public Dictionary<string, int> RowCounts { get; init; } = new(StringComparer.OrdinalIgnoreCase);
    public Dictionary<string, long> MonetarySums { get; init; } = new(StringComparer.OrdinalIgnoreCase);
}

public sealed class DatabaseCensusService
{
    private readonly ILogger<DatabaseCensusService> _logger;

    public DatabaseCensusService(ILogger<DatabaseCensusService> logger)
    {
        _logger = logger;
    }

    public async Task<DatabaseCensus> CollectAsync(string connectionString, CancellationToken cancellationToken = default)
    {
        var census = new DatabaseCensus();
        await using var connection = new SqliteConnection(connectionString);
        await connection.OpenAsync(cancellationToken);

        var tables = await GetUserTablesAsync(connection, cancellationToken);
        foreach (var table in tables)
        {
            census.RowCounts[table] = await GetRowCountAsync(connection, table, cancellationToken);
        }

        foreach (var (table, column) in MonetaryColumnRegistry.Columns)
        {
            if (!tables.Contains(table, StringComparer.OrdinalIgnoreCase))
                continue;

            if (!await ColumnExistsAsync(connection, table, column, cancellationToken))
                continue;

            var key = $"{table}.{column}";
            census.MonetarySums[key] = await GetMonetarySumAsync(connection, table, column, cancellationToken);
        }

        _logger.LogDebug("Censo de base de datos completado: {TableCount} tablas", census.RowCounts.Count);
        return census;
    }

    public static bool VerifyPreservation(DatabaseCensus before, DatabaseCensus after, out string? failureReason)
    {
        foreach (var (table, count) in before.RowCounts)
        {
            if (!after.RowCounts.TryGetValue(table, out var afterCount))
            {
                failureReason = $"La tabla '{table}' desapareció después de la migración.";
                return false;
            }

            if (afterCount < count)
            {
                failureReason = $"La tabla '{table}' perdió filas ({count} -> {afterCount}).";
                return false;
            }
        }

        failureReason = null;
        return true;
    }

    private static async Task<List<string>> GetUserTablesAsync(SqliteConnection connection, CancellationToken cancellationToken)
    {
        var tables = new List<string>();
        await using var command = connection.CreateCommand();
        command.CommandText = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '__EF%' ORDER BY name;";
        await using var reader = await command.ExecuteReaderAsync(cancellationToken);
        while (await reader.ReadAsync(cancellationToken))
        {
            tables.Add(reader.GetString(0));
        }

        return tables;
    }

    private static async Task<int> GetRowCountAsync(SqliteConnection connection, string table, CancellationToken cancellationToken)
    {
        await using var command = connection.CreateCommand();
        command.CommandText = $"SELECT COUNT(*) FROM \"{table}\";";
        var result = await command.ExecuteScalarAsync(cancellationToken);
        return Convert.ToInt32(result);
    }

    private static async Task<bool> ColumnExistsAsync(SqliteConnection connection, string table, string column, CancellationToken cancellationToken)
    {
        await using var command = connection.CreateCommand();
        command.CommandText = $"SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = $column;";
        command.Parameters.AddWithValue("$column", column);
        var result = await command.ExecuteScalarAsync(cancellationToken);
        return Convert.ToInt32(result) > 0;
    }

    private static async Task<long> GetMonetarySumAsync(SqliteConnection connection, string table, string column, CancellationToken cancellationToken)
    {
        await using var command = connection.CreateCommand();
        command.CommandText = $"SELECT COALESCE(SUM(CAST(\"{column}\" AS INTEGER)), 0) FROM \"{table}\";";
        var result = await command.ExecuteScalarAsync(cancellationToken);
        return Convert.ToInt64(result);
    }
}
