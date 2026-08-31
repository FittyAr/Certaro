using Avalonia;
using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class MasterDetailLayout : UserControl
{
    private const double StackBreakpoint = 960;

    public static readonly StyledProperty<object?> MasterProperty =
        AvaloniaProperty.Register<MasterDetailLayout, object?>(nameof(Master));

    public static readonly StyledProperty<object?> DetailProperty =
        AvaloniaProperty.Register<MasterDetailLayout, object?>(nameof(Detail));

    public object? Master
    {
        get => GetValue(MasterProperty);
        set => SetValue(MasterProperty, value);
    }

    public object? Detail
    {
        get => GetValue(DetailProperty);
        set => SetValue(DetailProperty, value);
    }

    public MasterDetailLayout()
    {
        InitializeComponent();
        UpdateLayout(Bounds.Width);
    }

    protected override void OnSizeChanged(SizeChangedEventArgs e)
    {
        base.OnSizeChanged(e);
        UpdateLayout(e.NewSize.Width);
    }

    private void UpdateLayout(double width)
    {
        if (LayoutGrid is null || MasterHost is null || DetailHost is null)
        {
            return;
        }

        if (width < StackBreakpoint)
        {
            LayoutGrid.ColumnDefinitions = ColumnDefinitions.Parse("*");
            LayoutGrid.RowDefinitions = RowDefinitions.Parse("*, Auto");

            Grid.SetColumn(MasterHost, 0);
            Grid.SetRow(MasterHost, 0);
            Grid.SetColumn(DetailHost, 0);
            Grid.SetRow(DetailHost, 1);
            DetailHost.MaxWidth = double.PositiveInfinity;
        }
        else
        {
            LayoutGrid.RowDefinitions = RowDefinitions.Parse("*");
            LayoutGrid.ColumnDefinitions = ColumnDefinitions.Parse("*, Auto");

            Grid.SetColumn(MasterHost, 0);
            Grid.SetRow(MasterHost, 0);
            Grid.SetColumn(DetailHost, 1);
            Grid.SetRow(DetailHost, 0);
            DetailHost.MaxWidth = 400;
        }
    }
}
