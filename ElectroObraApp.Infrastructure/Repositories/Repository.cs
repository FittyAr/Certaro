using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Expressions;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Core.Specifications;
using ElectroObraApp.Infrastructure.Data;
using ElectroObraApp.Infrastructure.Specifications;

namespace ElectroObraApp.Infrastructure.Repositories;

public class Repository<T> : IRepository<T> where T : class
{
    protected readonly ApplicationDbContext _context;
    protected readonly DbSet<T> _dbSet;

    public Repository(ApplicationDbContext context)
    {
        _context = context;
        _dbSet = context.Set<T>();
    }

    public async Task<T?> GetByIdAsync(Guid id)
    {
        return await _dbSet.AsNoTracking().FirstOrDefaultAsync(e => EF.Property<Guid>(e, "Id") == id);
    }

    public async Task<IEnumerable<T>> GetAllAsync()
    {
        return await _dbSet.AsNoTracking().ToListAsync();
    }

    public async Task<IEnumerable<T>> FindAsync(Expression<Func<T, bool>> predicate)
    {
        return await _dbSet.AsNoTracking().Where(predicate).ToListAsync();
    }

    public async Task<PagedResult<T>> GetPagedAsync(ISpecification<T> spec)
    {
        var countQuery = SpecificationEvaluator.GetCountQuery(_dbSet.AsQueryable(), spec);
        var totalCount = await countQuery.CountAsync();

        var items = await SpecificationEvaluator
            .GetQuery(_dbSet.AsQueryable(), spec)
            .ToListAsync();

        var pageSize = spec.Take ?? (totalCount == 0 ? 1 : totalCount);
        var pageNumber = spec.Skip.HasValue && spec.Take is > 0
            ? (spec.Skip.Value / spec.Take.Value) + 1
            : 1;

        return new PagedResult<T>
        {
            Items = items,
            TotalCount = totalCount,
            PageNumber = pageNumber,
            PageSize = pageSize
        };
    }

    public async Task AddAsync(T entity)
    {
        await _dbSet.AddAsync(entity);
    }

    public void Update(T entity)
    {
        var idProperty = typeof(T).GetProperty("Id");
        if (idProperty?.PropertyType == typeof(Guid))
        {
            var id = (Guid)idProperty.GetValue(entity)!;
            var tracked = _dbSet.Local.FirstOrDefault(e => (Guid)idProperty.GetValue(e)! == id);

            if (tracked != null)
            {
                _context.Entry(tracked).CurrentValues.SetValues(entity);
                return;
            }
        }

        _dbSet.Update(entity);
    }

    public void Remove(T entity)
    {
        _dbSet.Remove(entity);
    }
}
