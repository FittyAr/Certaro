using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using CommunityToolkit.Mvvm.Input;
using Material.Icons;

namespace ElectroObraApp.Controls;

public partial class FilterBar : UserControl
{
    public static readonly StyledProperty<string?> HeaderProperty =
        AvaloniaProperty.Register<FilterBar, string?>(nameof(Header));

    public static readonly StyledProperty<object?> FilterContentProperty =
        AvaloniaProperty.Register<FilterBar, object?>(nameof(FilterContent));

    public static readonly StyledProperty<bool> IsExpandedProperty =
        AvaloniaProperty.Register<FilterBar, bool>(nameof(IsExpanded), true);

    static FilterBar()
    {
        IsExpandedProperty.Changed.AddClassHandler<FilterBar>((c, _) => c.UpdateToggleVisual());
    }

    public string? Header
    {
        get => GetValue(HeaderProperty);
        set => SetValue(HeaderProperty, value);
    }

    public object? FilterContent
    {
        get => GetValue(FilterContentProperty);
        set => SetValue(FilterContentProperty, value);
    }

    public bool IsExpanded
    {
        get => GetValue(IsExpandedProperty);
        set => SetValue(IsExpandedProperty, value);
    }

    public ICommand ToggleCommand { get; }

    public FilterBar()
    {
        ToggleCommand = new RelayCommand(() => IsExpanded = !IsExpanded);
        InitializeComponent();
        UpdateToggleVisual();
    }

    private void UpdateToggleVisual()
    {
        if (ExpandIcon is null)
            return;

        ExpandIcon.Kind = IsExpanded ? MaterialIconKind.ChevronUp : MaterialIconKind.ChevronDown;
    }
}
