using System;
using System.Collections.Generic;

namespace ElectroObraApp.Application.DTOs;

public class DashboardStatsDto
{
    public decimal TotalIngresos { get; set; }
    public decimal TotalGastos { get; set; }
    public decimal Balance => TotalIngresos - TotalGastos;
    public int ClientesActivos { get; set; }
    public decimal Rentabilidad { get; set; }
    public int TrabajosPendientes { get; set; }
    public int LiquidacionesPendientes { get; set; }
    public bool DatabaseHealthy { get; set; }
    public string DatabaseStatus { get; set; } = string.Empty;
    public List<DashboardTopClienteDto> TopClientes { get; set; } = new();
    public List<MovimientoDto> RecentMovimientos { get; set; } = new();
    public double[] MonthlyIncome { get; set; } = new double[12];
    public double[] MonthlyExpenses { get; set; } = new double[12];
    public List<DashboardCategoryStatDto> CategoryExpenses { get; set; } = new();
    public int FacturasVencidasCount { get; set; }
    public int ObrasPausadasCount { get; set; }
    public decimal PreviousPeriodIngresos { get; set; }
    public decimal PreviousPeriodGastos { get; set; }
    public decimal? IngresosChangePercent { get; set; }
    public decimal? GastosChangePercent { get; set; }
    public List<DashboardObraRentabilidadDto> RankingObras { get; set; } = new();
}

public class DashboardObraRentabilidadDto
{
    public Guid ObraId { get; set; }
    public string Nombre { get; set; } = string.Empty;
    public decimal Ingresos { get; set; }
    public decimal Gastos { get; set; }
    public decimal Rentabilidad => Ingresos - Gastos;
    public decimal MargenPorcentaje { get; set; }
}

public class DashboardTopClienteDto
{
    public string Nombre { get; set; } = string.Empty;
    public decimal Total { get; set; }
}

public class DashboardCategoryStatDto
{
    public string Name { get; set; } = string.Empty;
    public double Value { get; set; }
}
