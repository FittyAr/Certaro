using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;

namespace ElectroObraApp.Controls;

public partial class StatusPill : UserControl
{
    public static readonly StyledProperty<string?> TextProperty =
        AvaloniaProperty.Register<StatusPill, string?>(nameof(Text));

    public static readonly StyledProperty<StatusKind> KindProperty =
        AvaloniaProperty.Register<StatusPill, StatusKind>(nameof(Kind), StatusKind.Neutral);

    static StatusPill()
    {
        TextProperty.Changed.AddClassHandler<StatusPill>((c, _) => c.UpdateAppearance());
        KindProperty.Changed.AddClassHandler<StatusPill>((c, _) => c.UpdateAppearance());
    }

    public string? Text
    {
        get => GetValue(TextProperty);
        set => SetValue(TextProperty, value);
    }

    public StatusKind Kind
    {
        get => GetValue(KindProperty);
        set => SetValue(KindProperty, value);
    }

    public StatusPill()
    {
        InitializeComponent();
        UpdateAppearance();
    }

    private void UpdateAppearance()
    {
        if (PillText is null || PillBorder is null)
            return;

        PillText.Text = Text;

        var resources = Avalonia.Application.Current?.Resources;
        if (resources is null)
            return;

        var (background, foreground) = StatusBrushHelper.GetBrushes(Kind, resources);
        PillBorder.Background = background;
        PillText.Foreground = foreground;
    }
}
