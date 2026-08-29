using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Core.Entities;

namespace ElectroObraApp.Core.Interfaces;

public interface IObraRepository : IRepository<Obra>
{
    Task<IEnumerable<Obra>> GetAllWithClienteAsync();
    Task<Obra?> GetByIdWithClienteAsync(Guid id);
}
