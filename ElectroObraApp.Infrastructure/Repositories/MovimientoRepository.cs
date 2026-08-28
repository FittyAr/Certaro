using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Repositories;

/// <summary>
/// Repositorio concreto para Movimiento.
/// Siempre hace Include de TipoMovimiento y Categoria para garantizar
/// que EsIngreso, TipoMovimientoSuma y CategoriaNombre sean mapeados correctamente.
/// </summary>
public class MovimientoRepository : Repository<Movimiento>, IMovimientoRepository
{
    public MovimientoRepository(ApplicationDbContext context) : base(context)
    {
    }

    public async Task<IEnumerable<Movimiento>> GetAllWithIncludesAsync()
    {
        return await _dbSet
            .Include(m => m.TipoMovimiento)
            .Include(m => m.Categoria)
            .AsNoTracking()
            .ToListAsync();
    }

    public async Task<Movimiento?> GetByIdWithIncludesAsync(Guid id)
    {
        return await _dbSet
            .Include(m => m.TipoMovimiento)
            .Include(m => m.Categoria)
            .AsNoTracking()
            .FirstOrDefaultAsync(m => m.Id == id);
    }
}

