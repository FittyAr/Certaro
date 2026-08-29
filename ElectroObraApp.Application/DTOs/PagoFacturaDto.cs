using System;
using ElectroObraApp.Application.Common;

namespace ElectroObraApp.Application.DTOs;

public class PagoFacturaDto : IHasGuidId
{
    public Guid Id { get; set; }
    public Guid FacturaId { get; set; }
    public string? FacturaNumero { get; set; }
    public DateTime Fecha { get; set; }
    public decimal Monto { get; set; }
    public string MedioPago { get; set; } = string.Empty;
}
