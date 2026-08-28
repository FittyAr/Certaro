using System;
using System.Collections.Concurrent;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore.Storage;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Infrastructure.Data;

namespace ElectroObraApp.Infrastructure.Repositories;

public class UnitOfWork : IUnitOfWork
{
    private readonly ApplicationDbContext _context;
    private readonly ConcurrentDictionary<Type, object> _repositories;
    private IMovimientoRepository? _movimientoRepository;
    private IClienteRepository? _clienteRepository;
    private ILiquidacionRepository? _liquidacionRepository;
    private ITrabajoRepository? _trabajoRepository;
    private IFacturaRepository? _facturaRepository;
    private IDbContextTransaction? _transaction;
    private bool _disposed;

    public UnitOfWork(ApplicationDbContext context)
    {
        _context = context;
        _repositories = new ConcurrentDictionary<Type, object>();
    }

    public IRepository<T> Repository<T>() where T : class
    {
        return (IRepository<T>)_repositories.GetOrAdd(typeof(T), _ => new Repository<T>(_context));
    }

    public IMovimientoRepository Movimientos => _movimientoRepository ??= new MovimientoRepository(_context);
    public IClienteRepository Clientes => _clienteRepository ??= new ClienteRepository(_context);
    public ILiquidacionRepository Liquidaciones => _liquidacionRepository ??= new LiquidacionRepository(_context);
    public ITrabajoRepository Trabajos => _trabajoRepository ??= new TrabajoRepository(_context);
    public IFacturaRepository Facturas => _facturaRepository ??= new FacturaRepository(_context);

    public async Task<int> SaveChangesAsync()
    {
        return await _context.SaveChangesAsync();
    }

    public async Task BeginTransactionAsync()
    {
        _transaction ??= await _context.Database.BeginTransactionAsync();
    }

    public async Task CommitTransactionAsync()
    {
        if (_transaction is null) return;

        await _transaction.CommitAsync();
        await _transaction.DisposeAsync();
        _transaction = null;
    }

    public async Task RollbackTransactionAsync()
    {
        if (_transaction is null) return;

        await _transaction.RollbackAsync();
        await _transaction.DisposeAsync();
        _transaction = null;
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!_disposed)
        {
            if (disposing)
            {
                _transaction?.Dispose();
                _context.Dispose();
            }
            _disposed = true;
        }
    }
}
