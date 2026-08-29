namespace ElectroObraApp.Core.Entities;

/// <summary>
/// Tracks the schema and application version that last wrote the database.
/// </summary>
public class SchemaVersion
{
    public int Id { get; set; } = 1;
    public string MigrationId { get; set; } = string.Empty;
    public string AppVersion { get; set; } = string.Empty;
    public DateTime AppliedAt { get; set; } = DateTime.UtcNow;
}
