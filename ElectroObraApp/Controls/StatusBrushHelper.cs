using Avalonia.Controls;
using Avalonia.Media;

namespace ElectroObraApp.Controls;

internal static class StatusBrushHelper
{
    public static (IBrush Background, IBrush Foreground) GetBrushes(StatusKind kind, IResourceDictionary resources) =>
        kind switch
        {
            StatusKind.Success => (GetBrush(resources, "SuccessSubtle"), GetBrush(resources, "Success")),
            StatusKind.Warning => (GetBrush(resources, "WarningSubtle"), GetBrush(resources, "Warning")),
            StatusKind.Error => (GetBrush(resources, "ErrorSubtle"), GetBrush(resources, "Error")),
            StatusKind.Info => (GetBrush(resources, "InfoSubtle"), GetBrush(resources, "Info")),
            StatusKind.Accent => (GetBrush(resources, "AccentSubtle"), GetBrush(resources, "Accent")),
            _ => (GetBrush(resources, "SurfaceSunken"), GetBrush(resources, "TextSecondary"))
        };

    private static IBrush GetBrush(IResourceDictionary resources, string key) =>
        resources.TryGetValue(key, out var value) && value is IBrush brush
            ? brush
            : Brushes.Transparent;
}
