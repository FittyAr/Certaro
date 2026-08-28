using System;
using System.Threading.Tasks;

namespace ElectroObraApp.Core.Interfaces;

public interface IUnitOfWork : IDisposable
{
    IRepository<T> Repository<T>() where T : class;
    IMovimientoRepository Movimientos { get; }
    IClienteRepository Clientes { get; }
    ILiquidacionRepository Liquidaciones { get; }
    ITrabajoRepository Trabajos { get; }
    IFacturaRepository Facturas { get; }
    Task<int> SaveChangesAsync();
    Task BeginTransactionAsync();
    Task CommitTransactionAsync();
    Task RollbackTransactionAsync();
}
