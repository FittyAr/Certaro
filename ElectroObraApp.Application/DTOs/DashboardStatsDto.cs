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
