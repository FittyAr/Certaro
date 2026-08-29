namespace ElectroObraApp.Application.Models;

public sealed class MigrationRunResult
{
    public bool Success { get; init; }
    public bool HadPendingMigrations { get; init; }
    public string? BackupPath { get; init; }
    public IReadOnlyList<string> AppliedMigrations { get; init; } = [];
    public string? ErrorMessage { get; init; }
}

public sealed class AppliedMigrationInfo
{
    public required string MigrationId { get; init; }
    public required string ProductVersion { get; init; }
}

public sealed class SchemaVersionInfo
{
    public required string EfMigrationId { get; init; }
    public required string AppVersion { get; init; }
    public required DateTime AppliedAt { get; init; }
}

public sealed class BackupResult
{
    public required string Path { get; init; }
    public required long SizeBytes { get; init; }
    public required bool IntegrityOk { get; init; }
}

public sealed class BackupInfo
{
    public required string Path { get; init; }
    public required string FileName { get; init; }
    public required DateTime CreatedAt { get; init; }
    public required long SizeBytes { get; init; }
}

public sealed class DatabaseCensus
{
    public Dictionary<string, long> RowCounts { get; init; } = new(StringComparer.Ordinal);
    public Dictionary<string, long> MonetarySums { get; init; } = new(StringComparer.Ordinal);
}
