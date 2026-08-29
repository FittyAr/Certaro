using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class AppStatusBar : UserControl
{
    public static readonly StyledProperty<string?> StatusTextProperty =
        AvaloniaProperty.Register<AppStatusBar, string?>(nameof(StatusText));

    public static readonly StyledProperty<string?> RouteTextProperty =
        AvaloniaProperty.Register<AppStatusBar, string?>(nameof(RouteText));

    public static readonly StyledProperty<ICommand?> GoBackCommandProperty =
        AvaloniaProperty.Register<AppStatusBar, ICommand?>(nameof(GoBackCommand));

    public static readonly StyledProperty<bool> CanGoBackProperty =
        AvaloniaProperty.Register<AppStatusBar, bool>(nameof(CanGoBack));

    public string? StatusText
    {
        get => GetValue(StatusTextProperty);
        set => SetValue(StatusTextProperty, value);
    }

    public string? RouteText
    {
        get => GetValue(RouteTextProperty);
        set => SetValue(RouteTextProperty, value);
    }

    public ICommand? GoBackCommand
    {
        get => GetValue(GoBackCommandProperty);
        set => SetValue(GoBackCommandProperty, value);
    }

    public bool CanGoBack
    {
        get => GetValue(CanGoBackProperty);
        set => SetValue(CanGoBackProperty, value);
    }

    public AppStatusBar()
    {
        InitializeComponent();
    }
}
