using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Core.Entities;

namespace ElectroObraApp.Core.Interfaces;

public interface IFacturaRepository : IRepository<Factura>
{
    Task<IEnumerable<Factura>> GetAllWithClienteAsync();
    Task<Factura?> GetByIdWithClienteAsync(Guid id);
}
