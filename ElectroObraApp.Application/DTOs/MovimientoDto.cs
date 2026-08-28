using System;
using ElectroObraApp.Application.Common;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Application.DTOs;

public class MovimientoDto : IHasGuidId
{
    public Guid Id { get; set; }
    public DateTime Fecha { get; set; }
    public string Concepto { get; set; } = string.Empty;
    public decimal Monto { get; set; }
    public decimal Cantidad { get; set; }
    public decimal Total { get; set; }
    
    public Guid TipoMovimientoId { get; set; }
    public string TipoMovimientoNombre { get; set; } = string.Empty;
    public bool TipoMovimientoSuma { get; set; }
    public bool EsIngreso { get; set; }

    public Moneda Moneda { get; set; }
    
    public Guid? CategoriaId { get; set; }
    public string? CategoriaNombre { get; set; }

    public Guid? EmpleadoId { get; set; }
    public Guid? ClienteId { get; set; }
    public string? ClienteNombre { get; set; }
    public Guid? TrabajoId { get; set; }
}

