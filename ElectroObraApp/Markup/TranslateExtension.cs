using System;
using System.ComponentModel;
using Avalonia;
using Avalonia.Data;
using Avalonia.Markup.Xaml;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.Markup;

public sealed class LocalizationBindingSource : INotifyPropertyChanged
{
    public static LocalizationBindingSource Instance { get; } = new();

    private LocalizationBindingSource()
    {
    }

    public event PropertyChangedEventHandler? PropertyChanged;

    public string this[string key] => Resolve(key);

    public void Refresh()
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs("Item[]"));
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(string.Empty));
    }

    private static string Resolve(string key)
    {
        if (global::Avalonia.Application.Current is App app && app.Services is not null)
        {
            return app.Services.GetRequiredService<ILocalizationService>().GetString(key);
        }

        return key;
    }
}

public class TranslateExtension : MarkupExtension
{
    public string Key { get; set; } = string.Empty;

    public override object ProvideValue(IServiceProvider serviceProvider)
    {
        if (string.IsNullOrWhiteSpace(Key))
        {
            return string.Empty;
        }

        var provideValueTarget = serviceProvider.GetService(typeof(IProvideValueTarget)) as IProvideValueTarget;
        if (provideValueTarget?.TargetObject is AvaloniaObject)
        {
            return new Avalonia.Data.Binding
            {
                Path = $"[{Key}]",
                Source = LocalizationBindingSource.Instance,
                Mode = Avalonia.Data.BindingMode.OneWay
            };
        }

        return LocalizationBindingSource.Instance[Key];
    }
}
