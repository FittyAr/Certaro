using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Infrastructure.Repositories;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Repositories;

public class MovimientoRepositoryTests
{
    [Fact]
    public async Task GetAllWithIncludesAsync_ShouldLoadRelatedEntities()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new MovimientoRepository(context);

        var tipo = new TipoMovimiento { Nombre = "Ingreso" };
        var categoria = new Categoria { Nombre = "Ventas" };
        context.Add(tipo);
        context.Add(categoria);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var movimiento = new Movimiento
        {
            Concepto = "Test",
            Monto = 100,
            TipoMovimientoId = tipo.Id,
            CategoriaId = categoria.Id
        };
        await repository.AddAsync(movimiento);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetAllWithIncludesAsync();

        var item = result.First();
        item.TipoMovimiento.Should().NotBeNull();
        item.TipoMovimiento.Nombre.Should().Be("Ingreso");
        item.Categoria.Should().NotBeNull();
        item.Categoria!.Nombre.Should().Be("Ventas");
    }

    [Fact]
    public async Task GetByIdWithIncludesAsync_ShouldReturnEntityWithRelated()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new MovimientoRepository(context);

        var tipo = new TipoMovimiento { Nombre = "Gasto" };
        context.Add(tipo);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var movimiento = new Movimiento
        {
            Concepto = "Gasto 1",
            Monto = 50,
            TipoMovimientoId = tipo.Id
        };
        await repository.AddAsync(movimiento);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetByIdWithIncludesAsync(movimiento.Id);

        result.Should().NotBeNull();
        result!.TipoMovimiento.Should().NotBeNull();
        result.TipoMovimiento.Nombre.Should().Be("Gasto");
    }
}
