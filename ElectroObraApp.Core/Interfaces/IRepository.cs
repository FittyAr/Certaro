using System;
using System.Collections.Generic;
using System.Linq.Expressions;
using System.Threading.Tasks;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Specifications;

namespace ElectroObraApp.Core.Interfaces;

public interface IRepository<T> where T : class
{
    Task<T?> GetByIdAsync(Guid id);
    Task<IEnumerable<T>> GetAllAsync();
    Task<IEnumerable<T>> FindAsync(Expression<Func<T, bool>> predicate);
    Task<PagedResult<T>> GetPagedAsync(ISpecification<T> spec);
    Task AddAsync(T entity);
    void Update(T entity);
    void Remove(T entity);
}

