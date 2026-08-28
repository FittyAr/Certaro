using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Repositories;

public class FacturaRepository : Repository<Factura>, IFacturaRepository
{
    public FacturaRepository(ApplicationDbContext context) : base(context) { }

    public async Task<IEnumerable<Factura>> GetAllWithClienteAsync()
    {
        return await _context.Facturas
            .AsNoTracking()
            .Include(f => f.Cliente)
            .OrderByDescending(f => f.Fecha)
            .ToListAsync();
    }

    public async Task<Factura?> GetByIdWithClienteAsync(Guid id)
    {
        return await _context.Facturas
            .AsNoTracking()
            .Include(f => f.Cliente)
            .FirstOrDefaultAsync(f => f.Id == id);
    }
}
