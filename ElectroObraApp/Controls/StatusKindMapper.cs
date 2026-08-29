using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Controls;

public static class StatusKindMapper
{
    public static StatusKind From(EstadoTrabajo estado) => estado switch
    {
        EstadoTrabajo.EnProceso => StatusKind.Accent,
        EstadoTrabajo.Finalizado => StatusKind.Success,
        EstadoTrabajo.Pausado => StatusKind.Warning,
        EstadoTrabajo.Cancelado => StatusKind.Error,
        _ => StatusKind.Neutral
    };

    public static StatusKind From(EstadoObra estado) => estado switch
    {
        EstadoObra.Activa => StatusKind.Success,
        EstadoObra.Pausada => StatusKind.Warning,
        EstadoObra.Finalizada => StatusKind.Info,
        EstadoObra.Cancelada => StatusKind.Error,
        _ => StatusKind.Neutral
    };

    public static StatusKind From(EstadoFactura estado) => estado switch
    {
        EstadoFactura.Emitida => StatusKind.Accent,
        EstadoFactura.Pagada => StatusKind.Success,
        EstadoFactura.Vencida => StatusKind.Warning,
        EstadoFactura.Anulada => StatusKind.Error,
        _ => StatusKind.Neutral
    };
}
