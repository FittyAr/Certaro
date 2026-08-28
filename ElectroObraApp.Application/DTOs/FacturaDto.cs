using System;
using ElectroObraApp.Application.Common;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Application.DTOs;

public class FacturaDto : IHasGuidId
{
    public Guid Id { get; set; }
    public string Numero { get; set; } = string.Empty;
    public DateTime Fecha { get; set; } = DateTime.Now;
    public Guid ClienteId { get; set; }
    public string? ClienteNombre { get; set; }
    public EstadoFactura Estado { get; set; } = EstadoFactura.Borrador;
    public decimal Subtotal { get; set; }
    public decimal Iva { get; set; }
    public decimal Total { get; set; }
    public string? Observaciones { get; set; }
}
