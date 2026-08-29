using System;

namespace ElectroObraApp.Core.Entities;

public class PagoFactura : BaseEntity
{
    public Guid FacturaId { get; set; }
    public Factura Factura { get; set; } = null!;

    public DateTime Fecha { get; set; }
    public decimal Monto { get; set; }
    public string MedioPago { get; set; } = string.Empty;
}
