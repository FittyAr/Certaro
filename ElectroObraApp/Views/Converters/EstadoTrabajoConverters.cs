using System;
using System.Globalization;
using Avalonia.Data.Converters;
using ElectroObraApp.Controls;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Views.Converters;

public sealed class EstadoTrabajoToStatusKindConverter : IValueConverter
{
    public static readonly EstadoTrabajoToStatusKindConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is EstadoTrabajo estado
            ? StatusKindMapper.From(estado)
            : StatusKind.Neutral;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

public sealed class EstadoTrabajoDisplayConverter : IValueConverter
{
    public static readonly EstadoTrabajoDisplayConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is not EstadoTrabajo estado)
            return string.Empty;

        return estado switch
        {
            EstadoTrabajo.Presupuestado => "Presupuestado",
            EstadoTrabajo.EnProceso => "En Curso",
            EstadoTrabajo.Pausado => "Pausado",
            EstadoTrabajo.Finalizado => "Finalizado",
            EstadoTrabajo.Cancelado => "Cancelado",
            _ => estado.ToString()
        };
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}
