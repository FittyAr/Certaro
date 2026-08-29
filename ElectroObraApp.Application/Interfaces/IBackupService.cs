namespace ElectroObraApp.Application.Interfaces;

public sealed class BackupInfo
{
    public string FilePath { get; init; } = string.Empty;
    public DateTime CreatedAt { get; init; }
    public long SizeBytes { get; init; }
}

public sealed class DatabaseExportResult
{
    public bool Success { get; init; }
    public string? FilePath { get; init; }
    public string? ErrorMessage { get; init; }
}

public interface IBackupService
{
    Task<string> CreateBackupAsync(CancellationToken cancellationToken = default);
    Task RestoreFromBackupAsync(string backupPath, CancellationToken cancellationToken = default);
    Task<IReadOnlyList<BackupInfo>> ListBackupsAsync(CancellationToken cancellationToken = default);
    Task CleanupOldBackupsAsync(CancellationToken cancellationToken = default);
    Task<DatabaseExportResult> ExportToJsonAsync(string outputPath, CancellationToken cancellationToken = default);
    Task<DatabaseExportResult> ImportFromJsonAsync(string inputPath, CancellationToken cancellationToken = default);
}
