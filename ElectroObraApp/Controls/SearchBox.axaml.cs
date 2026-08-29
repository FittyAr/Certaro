using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class SearchBox : UserControl
{
    public static readonly StyledProperty<string?> TextProperty =
        AvaloniaProperty.Register<SearchBox, string?>(nameof(Text), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<string?> PlaceholderTextProperty =
        AvaloniaProperty.Register<SearchBox, string?>(nameof(PlaceholderText));

    public static readonly StyledProperty<ICommand?> SearchCommandProperty =
        AvaloniaProperty.Register<SearchBox, ICommand?>(nameof(SearchCommand));

    public string? Text
    {
        get => GetValue(TextProperty);
        set => SetValue(TextProperty, value);
    }

    public string? PlaceholderText
    {
        get => GetValue(PlaceholderTextProperty);
        set => SetValue(PlaceholderTextProperty, value);
    }

    public ICommand? SearchCommand
    {
        get => GetValue(SearchCommandProperty);
        set => SetValue(SearchCommandProperty, value);
    }

    public SearchBox() => InitializeComponent();
}
