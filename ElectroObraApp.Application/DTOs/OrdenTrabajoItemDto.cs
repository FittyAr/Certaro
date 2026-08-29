using System;

namespace ElectroObraApp.Application.DTOs;

public class OrdenTrabajoItemDto
{
    public Guid Id { get; set; }
    public Guid OrdenTrabajoId { get; set; }
    public string Descripcion { get; set; } = string.Empty;
    public decimal Cantidad { get; set; }
    public string Unidad { get; set; } = "u";
    public decimal PrecioUnitario { get; set; }
    public decimal PorcentajeAnterior { get; set; }
    public decimal PorcentajeActual { get; set; }
    public decimal PorcentajeAcumulado => PorcentajeAnterior + PorcentajeActual;
    public bool Ejecutado { get; set; }
    public string? Nota { get; set; }
    public decimal SubtotalActual => Cantidad * PrecioUnitario * (PorcentajeActual / 100);
    public decimal SubtotalAcumulado => Cantidad * PrecioUnitario * (PorcentajeAcumulado / 100);
}

