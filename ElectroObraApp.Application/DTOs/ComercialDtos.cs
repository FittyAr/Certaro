using System;
using System.Collections.Generic;

namespace ElectroObraApp.Application.DTOs;

public class CuentaCorrienteClienteDto
{
    public Guid ClienteId { get; set; }
    public string ClienteNombre { get; set; } = string.Empty;
    public decimal TotalDeuda { get; set; }
    public IReadOnlyList<CuentaCorrienteItemDto> Items { get; set; } = Array.Empty<CuentaCorrienteItemDto>();
}

public class CuentaCorrienteItemDto
{
    public Guid FacturaId { get; set; }
    public string Numero { get; set; } = string.Empty;
    public DateTime Fecha { get; set; }
    public decimal Total { get; set; }
    public decimal Pagado { get; set; }
    public decimal Saldo { get; set; }
    public int DiasVencido { get; set; }
}

public class AntiguedadDeudaDto
{
    public Guid? ClienteId { get; set; }
    public string? ClienteNombre { get; set; }
    public decimal TotalDeuda { get; set; }
    public decimal Bucket0To30 { get; set; }
    public decimal Bucket31To60 { get; set; }
    public decimal Bucket61To90 { get; set; }
    public decimal BucketOver90 { get; set; }
}

public class RentabilidadObraDto
{
    public Guid ObraId { get; set; }
    public string Nombre { get; set; } = string.Empty;
    public string? ClienteNombre { get; set; }
    public decimal Ingresos { get; set; }
    public decimal Gastos { get; set; }
    public decimal Rentabilidad => Ingresos - Gastos;
    public decimal MargenPorcentaje => Ingresos > 0
        ? Math.Round((Rentabilidad / Ingresos) * 100m, 2)
        : 0m;
}
