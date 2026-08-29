using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using ElectroObraApp.Core.Enums;
using Mapster;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Services;

public class DashboardService : IDashboardService
{
    private const int OverdueInvoiceDays = 30;

    private readonly ApplicationDbContext _context;
    private readonly ILogger<DashboardService> _logger;

    public DashboardService(ApplicationDbContext context, ILogger<DashboardService> logger)
    {
        _context = context;
        _logger = logger;
    }

    public async Task<DashboardStatsDto> GetStatsAsync(string period)
    {
        var (currentStart, currentEnd) = GetCurrentPeriodRange(period);
        var previousRange = GetPreviousPeriodRange(period);

        var stats = new DashboardStatsDto();

        var currentAgg = await GetMovementAggregationAsync(currentStart, currentEnd);
        stats.TotalIngresos = currentAgg.Ingresos;
        stats.TotalGastos = currentAgg.Gastos;
        stats.Rentabilidad = stats.TotalIngresos > 0
            ? Math.Round((stats.Balance / stats.TotalIngresos) * 100m, 2)
            : 0m;

        if (previousRange.HasValue)
        {
            var (prevStart, prevEnd) = previousRange.Value;
            var previousAgg = await GetMovementAggregationAsync(prevStart, prevEnd);
            stats.PreviousPeriodIngresos = previousAgg.Ingresos;
            stats.PreviousPeriodGastos = previousAgg.Gastos;
            stats.IngresosChangePercent = CalculateChangePercent(previousAgg.Ingresos, currentAgg.Ingresos);
            stats.GastosChangePercent = CalculateChangePercent(previousAgg.Gastos, currentAgg.Gastos);
        }

        stats.ClientesActivos = await _context.Clientes
            .Where(c => _context.Movimientos.Any(m =>
                m.ClienteId == c.Id &&
                m.Fecha >= currentStart &&
                m.Fecha <= currentEnd &&
                _context.TiposMovimiento.Any(t => t.Id == m.TipoMovimientoId && t.EsIngreso)))
            .CountAsync();

        stats.TrabajosPendientes = await _context.Trabajos.CountAsync(t =>
            t.Estado != EstadoTrabajo.Finalizado && t.Estado != EstadoTrabajo.Cancelado);

        stats.ObrasPausadasCount = await _context.Obras.CountAsync(o => o.Estado == EstadoObra.Pausada);

        var overdueThreshold = DateTime.Today.AddDays(-OverdueInvoiceDays);
        stats.FacturasVencidasCount = await _context.Facturas.CountAsync(f =>
            f.Estado == EstadoFactura.Vencida ||
            (f.Estado == EstadoFactura.Emitida && f.Fecha <= overdueThreshold));

        var mesActual = DateTime.Now.Month;
        var añoActual = DateTime.Now.Year;
        stats.LiquidacionesPendientes = await _context.Empleados
            .Where(e => e.Activo)
            .CountAsync(e => !_context.Liquidaciones.Any(l =>
                l.EmpleadoId == e.Id &&
                l.FechaFin.Month == mesActual &&
                l.FechaFin.Year == añoActual));

        stats.TopClientes = await _context.Movimientos
            .Where(m => m.Fecha >= currentStart && m.Fecha <= currentEnd && m.ClienteId != null)
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

        stats.RankingObras = await BuildObraRankingAsync();

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

    private async Task<(decimal Ingresos, decimal Gastos)> GetMovementAggregationAsync(DateTime start, DateTime end)
    {
        var movimientoAgg = await _context.Movimientos
            .Where(m => m.Fecha >= start && m.Fecha <= end)
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

        return (movimientoAgg?.Ingresos ?? 0m, movimientoAgg?.Gastos ?? 0m);
    }

    private async Task<List<DashboardObraRentabilidadDto>> BuildObraRankingAsync()
    {
        var movimientos = await _context.Movimientos
            .AsNoTracking()
            .Where(m => m.TrabajoId != null)
            .Join(_context.Trabajos.AsNoTracking(),
                m => m.TrabajoId,
                t => t.Id,
                (m, t) => new { Movimiento = m, t.ObraId })
            .Join(_context.TiposMovimiento.AsNoTracking(),
                x => x.Movimiento.TipoMovimientoId,
                tm => tm.Id,
                (x, tm) => new
                {
                    x.ObraId,
                    Amount = x.Movimiento.Monto * x.Movimiento.Cantidad,
                    tm.EsIngreso
                })
            .ToListAsync();

        var obras = await _context.Obras.AsNoTracking().ToListAsync();

        return obras
            .Select(obra =>
            {
                var obraMovs = movimientos.Where(m => m.ObraId == obra.Id).ToList();
                var ingresos = obraMovs.Where(m => m.EsIngreso).Sum(m => m.Amount);
                var gastos = obraMovs.Where(m => !m.EsIngreso).Sum(m => m.Amount);
                var rentabilidad = ingresos - gastos;

                return new DashboardObraRentabilidadDto
                {
                    ObraId = obra.Id,
                    Nombre = obra.Nombre,
                    Ingresos = ingresos,
                    Gastos = gastos,
                    MargenPorcentaje = ingresos > 0
                        ? Math.Round((rentabilidad / ingresos) * 100m, 2)
                        : 0m
                };
            })
            .OrderByDescending(o => o.Rentabilidad)
            .Take(5)
            .ToList();
    }

    private static (DateTime Start, DateTime End) GetCurrentPeriodRange(string period)
    {
        var now = DateTime.Now;
        return period switch
        {
            "Mensual" => (now.AddMonths(-1), now),
            "Anual" => (now.AddYears(-1), now),
            _ => (DateTime.MinValue, now)
        };
    }

    private static (DateTime Start, DateTime End)? GetPreviousPeriodRange(string period)
    {
        var now = DateTime.Now;
        return period switch
        {
            "Mensual" => (now.AddMonths(-2), now.AddMonths(-1)),
            "Anual" => (now.AddYears(-2), now.AddYears(-1)),
            _ => null
        };
    }

    private static decimal? CalculateChangePercent(decimal previous, decimal current)
    {
        if (previous == 0m)
            return current == 0m ? 0m : null;

        return Math.Round(((current - previous) / previous) * 100m, 1);
    }
}
