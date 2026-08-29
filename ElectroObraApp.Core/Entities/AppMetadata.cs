namespace ElectroObraApp.Core.Entities;

/// <summary>
/// Key-value store for data migration flags and app metadata.
/// </summary>
public class AppMetadata
{
    public string Key { get; set; } = string.Empty;
    public string Value { get; set; } = string.Empty;
    public DateTime UpdatedAt { get; set; } = DateTime.UtcNow;
}
