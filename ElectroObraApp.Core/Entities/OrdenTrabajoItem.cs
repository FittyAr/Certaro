using System;

namespace ElectroObraApp.Core.Entities;

public class OrdenTrabajoItem : BaseEntity
{
    public Guid OrdenTrabajoId { get; set; }
    public OrdenTrabajo OrdenTrabajo { get; set; } = null!;

    public string Descripcion { get; set; } = string.Empty;
    public decimal Cantidad { get; set; }
    public string Unidad { get; set; } = "u";
    public decimal PrecioUnitario { get; set; }
    
    public decimal PorcentajeAnterior { get; set; }
    public decimal PorcentajeActual { get; set; }
    public decimal PorcentajeAcumulado => PorcentajeAnterior + PorcentajeActual;

    public (decimal SubtotalActual, decimal SubtotalAcumulado) CalculateSubtotals()
    {
        var baseAmount = Cantidad * PrecioUnitario;
        return (
            baseAmount * (PorcentajeActual / 100m),
            baseAmount * (PorcentajeAcumulado / 100m)
        );
    }
}
