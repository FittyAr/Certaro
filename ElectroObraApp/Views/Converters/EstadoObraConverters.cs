using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Globalization;
using Avalonia.Data.Converters;
using ElectroObraApp.Controls;
using ElectroObraApp.Core.Enums;
using ElectroObraApp.Markup;

namespace ElectroObraApp.Views.Converters;

public sealed class EstadoObraToStatusKindConverter : IValueConverter
{
    public static readonly EstadoObraToStatusKindConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is EstadoObra estado
            ? StatusKindMapper.From(estado)
            : StatusKind.Neutral;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

public sealed class EstadoObraDisplayConverter : IValueConverter
{
    public static readonly EstadoObraDisplayConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is not EstadoObra estado)
            return string.Empty;

        var key = estado switch
        {
            EstadoObra.Activa => "Obras.StatusActive",
            EstadoObra.Pausada => "Obras.StatusPaused",
            EstadoObra.Finalizada => "Obras.StatusFinished",
            EstadoObra.Cancelada => "Obras.StatusCancelled",
            _ => string.Empty
        };

        return string.IsNullOrEmpty(key)
            ? estado.ToString()
            : LocalizationBindingSource.Instance[key];
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}
