using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Repositories;

public class TrabajoRepository : Repository<Trabajo>, ITrabajoRepository
{
    public TrabajoRepository(ApplicationDbContext context) : base(context) { }

    public async Task<IEnumerable<Trabajo>> GetAllWithDeepLoadAsync()
    {
        return await _context.Trabajos
            .AsNoTracking()
            .Include(t => t.Obra)
                .ThenInclude(o => o.Cliente)
            .Include(t => t.OrdenesTrabajo)
                .ThenInclude(o => o.Items)
            .ToListAsync();
    }

    public async Task<Trabajo?> GetByIdWithDeepLoadAsync(Guid id)
    {
        return await _context.Trabajos
            .AsNoTracking()
            .Include(t => t.Obra)
                .ThenInclude(o => o.Cliente)
            .Include(t => t.OrdenesTrabajo)
                .ThenInclude(o => o.Items)
            .FirstOrDefaultAsync(t => t.Id == id);
    }
}

