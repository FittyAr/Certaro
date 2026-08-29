using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Enums;
using ElectroObraApp.Infrastructure.Data;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;

namespace ElectroObraApp.Infrastructure.Services;

public class ComercialService : IComercialService
{
    private const int OverdueDaysThreshold = 30;

    private readonly ApplicationDbContext _context;
    private readonly ILogger<ComercialService> _logger;

    public ComercialService(ApplicationDbContext context, ILogger<ComercialService> logger)
    {
        _context = context;
        _logger = logger;
    }

    public async Task<CuentaCorrienteClienteDto> GetCuentaCorrienteClienteAsync(Guid clienteId)
    {
        var cliente = await _context.Clientes
            .AsNoTracking()
            .FirstOrDefaultAsync(c => c.Id == clienteId);

        if (cliente is null)
        {
            _logger.LogWarning("Cliente no encontrado para cuenta corriente: {ClienteId}", clienteId);
            return new CuentaCorrienteClienteDto { ClienteId = clienteId };
        }

        var facturas = await _context.Facturas
            .AsNoTracking()
            .Include(f => f.Pagos)
            .Where(f => f.ClienteId == clienteId &&
                        (f.Estado == EstadoFactura.Emitida || f.Estado == EstadoFactura.Vencida))
            .OrderByDescending(f => f.Fecha)
            .ToListAsync();

        var items = facturas
            .Select(BuildCuentaCorrienteItem)
            .Where(i => i.Saldo > 0)
            .ToList();

        return new CuentaCorrienteClienteDto
        {
            ClienteId = clienteId,
            ClienteNombre = cliente.Nombre,
            TotalDeuda = items.Sum(i => i.Saldo),
            Items = items
        };
    }

    public async Task<AntiguedadDeudaDto> GetAntiguedadDeudaAsync(Guid? clienteId = null)
    {
        var query = _context.Facturas
            .AsNoTracking()
            .Include(f => f.Pagos)
            .Include(f => f.Cliente)
            .Where(f => f.Estado == EstadoFactura.Emitida || f.Estado == EstadoFactura.Vencida);

        if (clienteId.HasValue)
            query = query.Where(f => f.ClienteId == clienteId.Value);

        var facturas = await query.ToListAsync();
        var today = DateTime.Today;

        var aging = new AntiguedadDeudaDto
        {
            ClienteId = clienteId,
            ClienteNombre = clienteId.HasValue
                ? facturas.FirstOrDefault()?.Cliente?.Nombre
                : null
        };

        foreach (var factura in facturas)
        {
            var saldo = factura.Total - factura.Pagos.Sum(p => p.Monto);
            if (saldo <= 0)
                continue;

            var dias = (today - factura.Fecha.Date).Days;
            aging.TotalDeuda += saldo;

            if (dias <= 30)
                aging.Bucket0To30 += saldo;
            else if (dias <= 60)
                aging.Bucket31To60 += saldo;
            else if (dias <= 90)
                aging.Bucket61To90 += saldo;
            else
                aging.BucketOver90 += saldo;
        }

        return aging;
    }

    public async Task<IReadOnlyList<RentabilidadObraDto>> GetRentabilidadPorObraAsync()
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

        var obras = await _context.Obras
            .AsNoTracking()
            .Include(o => o.Cliente)
            .ToListAsync();

        return obras
            .Select(obra =>
            {
                var obraMovs = movimientos.Where(m => m.ObraId == obra.Id).ToList();
                var ingresos = obraMovs.Where(m => m.EsIngreso).Sum(m => m.Amount);
                var gastos = obraMovs.Where(m => !m.EsIngreso).Sum(m => m.Amount);

                return new RentabilidadObraDto
                {
                    ObraId = obra.Id,
                    Nombre = obra.Nombre,
                    ClienteNombre = obra.Cliente?.Nombre,
                    Ingresos = ingresos,
                    Gastos = gastos
                };
            })
            .OrderByDescending(o => o.Rentabilidad)
            .ToList();
    }

    private static CuentaCorrienteItemDto BuildCuentaCorrienteItem(Core.Entities.Factura factura)
    {
        var pagado = factura.Pagos.Sum(p => p.Monto);
        var saldo = factura.Total - pagado;
        var dias = Math.Max(0, (DateTime.Today - factura.Fecha.Date).Days - OverdueDaysThreshold);

        if (factura.Estado == EstadoFactura.Vencida)
            dias = Math.Max(dias, (DateTime.Today - factura.Fecha.Date).Days);

        return new CuentaCorrienteItemDto
        {
            FacturaId = factura.Id,
            Numero = factura.Numero,
            Fecha = factura.Fecha,
            Total = factura.Total,
            Pagado = pagado,
            Saldo = saldo,
            DiasVencido = saldo > 0 ? Math.Max(0, (DateTime.Today - factura.Fecha.Date).Days) : 0
        };
    }
}
