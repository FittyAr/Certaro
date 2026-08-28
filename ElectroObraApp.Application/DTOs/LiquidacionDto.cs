using System;
using ElectroObraApp.Application.Common;

namespace ElectroObraApp.Application.DTOs;

public class LiquidacionDto : IHasGuidId
{
    public Guid Id { get; set; }
    public Guid EmpleadoId { get; set; }
    public string EmpleadoNombre { get; set; } = string.Empty;
    public DateTime FechaInicio { get; set; }
    public DateTime FechaFin { get; set; }
    public decimal DiasTrabajados { get; set; }
    public decimal TarifaAplicada { get; set; }
    
    public bool IncluirSabados { get; set; }
    public bool IncluirDomingos { get; set; }
    public bool IncluirFeriados { get; set; }

    public decimal MultiplicadorSabado { get; set; }
    public decimal MultiplicadorDomingo { get; set; }
    public decimal MultiplicadorFeriado { get; set; }

    public decimal TotalBruto { get; set; }
    public decimal TotalAdelantos { get; set; }
    public decimal TotalNeto { get; set; }
    public string? Observaciones { get; set; }
}

