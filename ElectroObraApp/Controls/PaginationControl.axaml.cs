using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class PaginationControl : UserControl
{
    public static readonly StyledProperty<int> CurrentPageProperty =
        AvaloniaProperty.Register<PaginationControl, int>(nameof(CurrentPage), 1);

    public static readonly StyledProperty<int> TotalPagesProperty =
        AvaloniaProperty.Register<PaginationControl, int>(nameof(TotalPages), 1);

    public static readonly StyledProperty<ICommand?> PreviousCommandProperty =
        AvaloniaProperty.Register<PaginationControl, ICommand?>(nameof(PreviousCommand));

    public static readonly StyledProperty<ICommand?> NextCommandProperty =
        AvaloniaProperty.Register<PaginationControl, ICommand?>(nameof(NextCommand));

    public int CurrentPage
    {
        get => GetValue(CurrentPageProperty);
        set => SetValue(CurrentPageProperty, value);
    }

    public int TotalPages
    {
        get => GetValue(TotalPagesProperty);
        set => SetValue(TotalPagesProperty, value);
    }

    public ICommand? PreviousCommand
    {
        get => GetValue(PreviousCommandProperty);
        set => SetValue(PreviousCommandProperty, value);
    }

    public ICommand? NextCommand
    {
        get => GetValue(NextCommandProperty);
        set => SetValue(NextCommandProperty, value);
    }

    public PaginationControl()
    {
        InitializeComponent();
    }
}
