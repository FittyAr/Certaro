namespace ElectroObraApp.Application.Interfaces;

public sealed class MigrationRunResult
{
    public bool Success { get; init; }
    public bool MigrationsApplied { get; init; }
    public string? BackupPath { get; init; }
    public string? ErrorMessage { get; init; }
    public IReadOnlyList<string> AppliedMigrations { get; init; } = [];
}

public interface IMigrationRunner
{
    Task<MigrationRunResult> RunPendingMigrationsAsync(CancellationToken cancellationToken = default);
    Task<IReadOnlyList<string>> GetAppliedMigrationsAsync(CancellationToken cancellationToken = default);
    Task<IReadOnlyList<string>> GetPendingMigrationsAsync(CancellationToken cancellationToken = default);
    Task<IReadOnlyList<string>> GetBackupFilesAsync(CancellationToken cancellationToken = default);
}
