using System;
using System.Collections.Generic;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Core.Entities;

public class Factura : BaseEntity
{
    public string Numero { get; set; } = string.Empty;
    public DateTime Fecha { get; set; } = DateTime.Now;
    public Guid ClienteId { get; set; }
    public virtual Cliente Cliente { get; set; } = null!;
    public EstadoFactura Estado { get; set; } = EstadoFactura.Borrador;
    public decimal Subtotal { get; set; }
    public decimal Iva { get; set; }
    public decimal Total { get; set; }
    public string? Observaciones { get; set; }
    public ICollection<Movimiento> Movimientos { get; set; } = new List<Movimiento>();
    public ICollection<PagoFactura> Pagos { get; set; } = new List<PagoFactura>();
}
