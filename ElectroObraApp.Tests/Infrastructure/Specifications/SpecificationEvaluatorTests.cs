using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Specifications;
using ElectroObraApp.Infrastructure.Repositories;
using ElectroObraApp.Infrastructure.Specifications;
using ElectroObraApp.Tests.Infrastructure.Repositories;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Specifications;

public class SpecificationEvaluatorTests
{
    [Fact]
    public async Task GetPagedAsync_ShouldApplyCriteriaAndPaging()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new Repository<Movimiento>(context);

        var tipo = new TipoMovimiento { Nombre = "Ingreso" };
        context.Add(tipo);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        for (var i = 0; i < 5; i++)
        {
            await repository.AddAsync(new Movimiento
            {
                Concepto = i % 2 == 0 ? "Agua" : "Luz",
                Monto = 100 + i,
                TipoMovimientoId = tipo.Id,
                Fecha = DateTime.Today.AddDays(-i)
            });
        }

        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var spec = new MovimientosPagedSpecification("Agua", null, null, null, null, null, 1, 2);
        var result = await repository.GetPagedAsync(spec);

        result.TotalCount.Should().Be(3);
        result.Items.Should().HaveCount(2);
        result.PageNumber.Should().Be(1);
        result.PageSize.Should().Be(2);
        result.Items.Should().OnlyContain(m => m.Concepto.Contains("Agua", StringComparison.OrdinalIgnoreCase));
    }

    [Fact]
    public async Task GetQuery_ShouldApplyOrderByDescending()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var tipo = new TipoMovimiento { Nombre = "Gasto" };
        context.Add(tipo);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var older = new Movimiento { Concepto = "Old", Monto = 1, TipoMovimientoId = tipo.Id, Fecha = DateTime.Today.AddDays(-5) };
        var newer = new Movimiento { Concepto = "New", Monto = 2, TipoMovimientoId = tipo.Id, Fecha = DateTime.Today };
        context.AddRange(older, newer);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var spec = new MovimientosPagedSpecification(null, null, null, null, null, null, 1, 10);
        var query = SpecificationEvaluator.GetQuery(context.Set<Movimiento>().AsQueryable(), spec);
        var items = query.ToList();

        items.First().Concepto.Should().Be("New");
    }
}
