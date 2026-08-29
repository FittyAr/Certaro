using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Repositories;

public class ObraRepository : Repository<Obra>, IObraRepository
{
    public ObraRepository(ApplicationDbContext context) : base(context) { }

    public async Task<IEnumerable<Obra>> GetAllWithClienteAsync()
    {
        return await _context.Obras
            .AsNoTracking()
            .Include(o => o.Cliente)
            .OrderBy(o => o.Numero)
            .ToListAsync();
    }

    public async Task<Obra?> GetByIdWithClienteAsync(Guid id)
    {
        return await _context.Obras
            .AsNoTracking()
            .Include(o => o.Cliente)
            .FirstOrDefaultAsync(o => o.Id == id);
    }
}
