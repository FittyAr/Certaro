using System;
using System.Globalization;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;

namespace ElectroObraApp.Controls;

public partial class MoneyText : UserControl
{
    public static readonly StyledProperty<decimal> AmountProperty =
        AvaloniaProperty.Register<MoneyText, decimal>(nameof(Amount));

    public static readonly StyledProperty<string> CurrencySymbolProperty =
        AvaloniaProperty.Register<MoneyText, string>(nameof(CurrencySymbol), "$");

    public static readonly StyledProperty<bool> ColorBySignProperty =
        AvaloniaProperty.Register<MoneyText, bool>(nameof(ColorBySign), true);

    public static readonly StyledProperty<bool> ShowPositiveSignProperty =
        AvaloniaProperty.Register<MoneyText, bool>(nameof(ShowPositiveSign));

    static MoneyText()
    {
        AmountProperty.Changed.AddClassHandler<MoneyText>((c, _) => c.UpdateDisplay());
        CurrencySymbolProperty.Changed.AddClassHandler<MoneyText>((c, _) => c.UpdateDisplay());
        ColorBySignProperty.Changed.AddClassHandler<MoneyText>((c, _) => c.UpdateDisplay());
        ShowPositiveSignProperty.Changed.AddClassHandler<MoneyText>((c, _) => c.UpdateDisplay());
    }

    public decimal Amount
    {
        get => GetValue(AmountProperty);
        set => SetValue(AmountProperty, value);
    }

    public string CurrencySymbol
    {
        get => GetValue(CurrencySymbolProperty);
        set => SetValue(CurrencySymbolProperty, value);
    }

    public bool ColorBySign
    {
        get => GetValue(ColorBySignProperty);
        set => SetValue(ColorBySignProperty, value);
    }

    public bool ShowPositiveSign
    {
        get => GetValue(ShowPositiveSignProperty);
        set => SetValue(ShowPositiveSignProperty, value);
    }

    public MoneyText()
    {
        InitializeComponent();
        UpdateDisplay();
    }

    private void UpdateDisplay()
    {
        if (AmountText is null)
            return;

        var culture = CultureInfo.CurrentCulture;
        var formatted = Math.Abs(Amount).ToString("N2", culture);
        var sign = Amount switch
        {
            < 0 => "-",
            > 0 when ShowPositiveSign => "+",
            _ => string.Empty
        };

        AmountText.Text = $"{sign}{CurrencySymbol}{formatted}";

        if (!ColorBySign)
            return;

        var resources = Avalonia.Application.Current?.Resources;
        var key = Amount switch
        {
            > 0 => "AmountPositive",
            < 0 => "AmountNegative",
            _ => "TextPrimary"
        };

        if (resources?.TryGetValue(key, out var brush) == true && brush is IBrush b)
            AmountText.Foreground = b;
    }
}
