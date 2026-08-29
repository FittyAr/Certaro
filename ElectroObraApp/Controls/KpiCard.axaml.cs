using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Material.Icons;

namespace ElectroObraApp.Controls;

public partial class KpiCard : UserControl
{
    public static readonly StyledProperty<string?> TitleProperty =
        AvaloniaProperty.Register<KpiCard, string?>(nameof(Title));

    public static readonly StyledProperty<string?> ValueProperty =
        AvaloniaProperty.Register<KpiCard, string?>(nameof(Value));

    public static readonly StyledProperty<string?> TrendProperty =
        AvaloniaProperty.Register<KpiCard, string?>(nameof(Trend));

    public static readonly StyledProperty<bool?> TrendIsPositiveProperty =
        AvaloniaProperty.Register<KpiCard, bool?>(nameof(TrendIsPositive));

    static KpiCard()
    {
        TrendProperty.Changed.AddClassHandler<KpiCard>((c, _) => c.UpdateTrend());
        TrendIsPositiveProperty.Changed.AddClassHandler<KpiCard>((c, _) => c.UpdateTrend());
    }

    public string? Title
    {
        get => GetValue(TitleProperty);
        set => SetValue(TitleProperty, value);
    }

    public string? Value
    {
        get => GetValue(ValueProperty);
        set => SetValue(ValueProperty, value);
    }

    public string? Trend
    {
        get => GetValue(TrendProperty);
        set => SetValue(TrendProperty, value);
    }

    public bool? TrendIsPositive
    {
        get => GetValue(TrendIsPositiveProperty);
        set => SetValue(TrendIsPositiveProperty, value);
    }

    private bool HasTrend => !string.IsNullOrWhiteSpace(Trend);

    public KpiCard()
    {
        InitializeComponent();
        UpdateTrend();
    }

    private void UpdateTrend()
    {
        if (TrendTextBlock is null || TrendIcon is null)
            return;

        TrendTextBlock.Text = Trend;
        TrendTextBlock.IsVisible = HasTrend;
        TrendIcon.IsVisible = HasTrend;

        if (!HasTrend)
            return;

        var resources = Avalonia.Application.Current?.Resources;
        IBrush color = TrendIsPositive switch
        {
            true when resources?.TryGetValue("Success", out var success) == true && success is IBrush s => s,
            false when resources?.TryGetValue("Error", out var error) == true && error is IBrush e => e,
            _ when resources?.TryGetValue("TextTertiary", out var neutral) == true && neutral is IBrush n => n,
            _ => Brushes.Gray
        };

        TrendIcon.Foreground = color;
        TrendTextBlock.Foreground = color;
        TrendIcon.Kind = TrendIsPositive switch
        {
            true => MaterialIconKind.TrendingUp,
            false => MaterialIconKind.TrendingDown,
            _ => MaterialIconKind.TrendingNeutral
        };
    }
}
