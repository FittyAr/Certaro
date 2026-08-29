using Avalonia;
using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class FormField : UserControl
{
    public static readonly StyledProperty<string?> LabelProperty =
        AvaloniaProperty.Register<FormField, string?>(nameof(Label));

    public static readonly StyledProperty<object?> FieldContentProperty =
        AvaloniaProperty.Register<FormField, object?>(nameof(FieldContent));

    public static readonly StyledProperty<string?> ErrorProperty =
        AvaloniaProperty.Register<FormField, string?>(nameof(Error));

    public static readonly StyledProperty<bool> IsRequiredProperty =
        AvaloniaProperty.Register<FormField, bool>(nameof(IsRequired));

    public string? Label
    {
        get => GetValue(LabelProperty);
        set => SetValue(LabelProperty, value);
    }

    public object? FieldContent
    {
        get => GetValue(FieldContentProperty);
        set => SetValue(FieldContentProperty, value);
    }

    public string? Error
    {
        get => GetValue(ErrorProperty);
        set => SetValue(ErrorProperty, value);
    }

    public bool IsRequired
    {
        get => GetValue(IsRequiredProperty);
        set => SetValue(IsRequiredProperty, value);
    }

    public FormField() => InitializeComponent();
}
