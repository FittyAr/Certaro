using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Mapster;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Services;

public class DashboardService : IDashboardService
{
    private readonly ApplicationDbContext _context;
    private readonly ILogger<DashboardService> _logger;

    public DashboardService(ApplicationDbContext context, ILogger<DashboardService> logger)
    {
        _context = context;
        _logger = logger;
    }

    public async Task<DashboardStatsDto> GetStatsAsync(string period)
    {
        var filterDate = period switch
        {
            "Mensual" => DateTime.Now.AddMonths(-1),
            "Anual" => DateTime.Now.AddYears(-1),
            _ => DateTime.MinValue
        };

        var stats = new DashboardStatsDto();

        var movimientoAgg = await _context.Movimientos
            .Where(m => m.Fecha >= filterDate)
            .Join(_context.TiposMovimiento,
                m => m.TipoMovimientoId,
                t => t.Id,
                (m, t) => new { Amount = m.Monto * m.Cantidad, t.EsIngreso })
            .GroupBy(_ => 1)
            .Select(g => new
            {
                Ingresos = g.Where(x => x.EsIngreso).Sum(x => (decimal?)x.Amount) ?? 0m,
                Gastos = g.Where(x => !x.EsIngreso).Sum(x => (decimal?)x.Amount) ?? 0m
            })
            .FirstOrDefaultAsync();

        stats.TotalIngresos = movimientoAgg?.Ingresos ?? 0m;
        stats.TotalGastos = movimientoAgg?.Gastos ?? 0m;
        stats.Rentabilidad = stats.TotalIngresos > 0
            ? Math.Round((stats.Balance / stats.TotalIngresos) * 100m, 2)
            : 0m;

        stats.ClientesActivos = await _context.Clientes
            .Where(c => _context.Movimientos.Any(m =>
                m.ClienteId == c.Id &&
                m.Fecha >= filterDate &&
                _context.TiposMovimiento.Any(t => t.Id == m.TipoMovimientoId && t.EsIngreso)))
            .CountAsync();

        stats.TrabajosPendientes = await _context.Trabajos.CountAsync(t => !t.Finalizado);

        var mesActual = DateTime.Now.Month;
        var añoActual = DateTime.Now.Year;
        stats.LiquidacionesPendientes = await _context.Empleados
            .Where(e => e.Activo)
            .CountAsync(e => !_context.Liquidaciones.Any(l =>
                l.EmpleadoId == e.Id &&
                l.FechaFin.Month == mesActual &&
                l.FechaFin.Year == añoActual));

        stats.TopClientes = await _context.Movimientos
            .Where(m => m.Fecha >= filterDate && m.ClienteId != null)
            .Join(_context.TiposMovimiento, m => m.TipoMovimientoId, t => t.Id, (m, t) => new { m, t })
            .Where(x => x.t.EsIngreso)
            .Join(_context.Clientes, x => x.m.ClienteId, c => c.Id, (x, c) => new { c.Nombre, Amount = x.m.Monto * x.m.Cantidad })
            .GroupBy(x => x.Nombre)
            .Select(g => new DashboardTopClienteDto
            {
                Nombre = g.Key,
                Total = g.Sum(x => x.Amount)
            })
            .OrderByDescending(x => x.Total)
            .Take(3)
            .ToListAsync();

        var recentEntities = await _context.Movimientos
            .AsNoTracking()
            .Include(m => m.TipoMovimiento)
            .Include(m => m.Categoria)
            .Include(m => m.Cliente)
            .OrderByDescending(m => m.Fecha)
            .Take(5)
            .ToListAsync();

        stats.RecentMovimientos = recentEntities.Adapt<List<MovimientoDto>>();

        var yearMovimientos = await _context.Movimientos
            .Where(m => m.Fecha.Year == DateTime.Now.Year)
            .Join(_context.TiposMovimiento, m => m.TipoMovimientoId, t => t.Id, (m, t) => new { m.Fecha.Month, Amount = m.Monto * m.Cantidad, t.EsIngreso })
            .ToListAsync();

        foreach (var item in yearMovimientos)
        {
            var index = item.Month - 1;
            if (item.EsIngreso)
                stats.MonthlyIncome[index] += (double)item.Amount;
            else
                stats.MonthlyExpenses[index] += (double)item.Amount;
        }

        stats.CategoryExpenses = await _context.Movimientos
            .Join(_context.TiposMovimiento, m => m.TipoMovimientoId, t => t.Id, (m, t) => new { m, t })
            .Where(x => !x.t.EsIngreso && x.m.CategoriaId != null)
            .Join(_context.Categorias, x => x.m.CategoriaId, c => c.Id, (x, c) => new { c.Nombre, Amount = x.m.Monto * x.m.Cantidad })
            .GroupBy(x => x.Nombre)
            .Select(g => new DashboardCategoryStatDto
            {
                Name = g.Key,
                Value = (double)g.Sum(x => x.Amount)
            })
            .OrderByDescending(x => x.Value)
            .Take(5)
            .ToListAsync();

        stats.DatabaseHealthy = await CheckDatabaseHealthAsync();
        stats.DatabaseStatus = stats.DatabaseHealthy ? "Saludable" : "Error de conexión";

        return stats;
    }

    public async Task<bool> CheckDatabaseHealthAsync()
    {
        try
        {
            return await _context.Database.CanConnectAsync();
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Fallo en chequeo de salud de base de datos");
            return false;
        }
    }
}
