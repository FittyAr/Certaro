using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class ListStateOverlay : UserControl
{
    public static readonly StyledProperty<bool> IsLoadingProperty =
        AvaloniaProperty.Register<ListStateOverlay, bool>(nameof(IsLoading));

    public static readonly StyledProperty<bool> IsEmptyProperty =
        AvaloniaProperty.Register<ListStateOverlay, bool>(nameof(IsEmpty));

    public static readonly StyledProperty<bool> HasErrorProperty =
        AvaloniaProperty.Register<ListStateOverlay, bool>(nameof(HasError));

    public static readonly StyledProperty<string?> ErrorMessageProperty =
        AvaloniaProperty.Register<ListStateOverlay, string?>(nameof(ErrorMessage));

    public static readonly StyledProperty<ICommand?> RetryCommandProperty =
        AvaloniaProperty.Register<ListStateOverlay, ICommand?>(nameof(RetryCommand));

    public bool IsLoading
    {
        get => GetValue(IsLoadingProperty);
        set => SetValue(IsLoadingProperty, value);
    }

    public bool IsEmpty
    {
        get => GetValue(IsEmptyProperty);
        set => SetValue(IsEmptyProperty, value);
    }

    public bool HasError
    {
        get => GetValue(HasErrorProperty);
        set => SetValue(HasErrorProperty, value);
    }

    public string? ErrorMessage
    {
        get => GetValue(ErrorMessageProperty);
        set => SetValue(ErrorMessageProperty, value);
    }

    public ICommand? RetryCommand
    {
        get => GetValue(RetryCommandProperty);
        set => SetValue(RetryCommandProperty, value);
    }

    public ListStateOverlay()
    {
        InitializeComponent();
    }
}
