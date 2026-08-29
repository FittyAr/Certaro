using System;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Core.Entities;

public class Movimiento : BaseEntity
{
    public DateTime Fecha { get; set; } = DateTime.Now;
    public string Concepto { get; set; } = string.Empty;
    public decimal Monto { get; set; }
    public decimal Cantidad { get; set; } = 1;
    public decimal Total => Monto * Cantidad;
    
    public Guid TipoMovimientoId { get; set; }
    public virtual TipoMovimiento TipoMovimiento { get; set; } = null!;

    public Moneda Moneda { get; set; } = Moneda.ARS;
    public decimal? CotizacionAplicada { get; set; }

    public Guid? TipoConceptoPagoId { get; set; }
    public TipoConceptoPago? TipoConceptoPago { get; set; }

    // Relaciones
    public Guid? CategoriaId { get; set; }
    public Categoria? Categoria { get; set; }
    
    public Guid? ClienteId { get; set; }
    public Cliente? Cliente { get; set; }
    
    public Guid? TrabajoId { get; set; }
    public Trabajo? Trabajo { get; set; }
    
    public Guid? EmpleadoId { get; set; }
    public Empleado? Empleado { get; set; }
    
    public Guid? FacturaId { get; set; }
    public virtual Factura? Factura { get; set; }
}

