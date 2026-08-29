using System.Globalization;
using Avalonia;
using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class PercentBar : UserControl
{
    public static readonly StyledProperty<double> ValueProperty =
        AvaloniaProperty.Register<PercentBar, double>(nameof(Value));

    public static readonly StyledProperty<string> FormatProperty =
        AvaloniaProperty.Register<PercentBar, string>(nameof(Format), "{0:N1}%");

    static PercentBar()
    {
        ValueProperty.Changed.AddClassHandler<PercentBar>((c, _) => c.UpdateDisplay());
        FormatProperty.Changed.AddClassHandler<PercentBar>((c, _) => c.UpdateDisplay());
    }

    public double Value
    {
        get => GetValue(ValueProperty);
        set => SetValue(ValueProperty, value);
    }

    public string Format
    {
        get => GetValue(FormatProperty);
        set => SetValue(FormatProperty, value);
    }

    public PercentBar()
    {
        InitializeComponent();
        UpdateDisplay();
    }

    private void UpdateDisplay()
    {
        if (Bar is null || PercentText is null)
            return;

        Bar.Value = Value;
        PercentText.Text = string.Format(CultureInfo.CurrentCulture, Format, Value);
    }
}
