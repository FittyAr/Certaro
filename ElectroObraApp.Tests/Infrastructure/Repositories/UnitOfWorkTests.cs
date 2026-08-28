using System.Threading.Tasks;
using FluentAssertions;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Infrastructure.Repositories;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Repositories;

public class UnitOfWorkTests
{
    [Fact]
    public async Task SaveChangesAsync_ShouldPersistData()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var uow = new UnitOfWork(context);
        var repo = uow.Repository<Categoria>();
        var categoria = new Categoria { Nombre = "Test" };

        await repo.AddAsync(categoria);
        await uow.SaveChangesAsync();

        var result = await repo.GetByIdAsync(categoria.Id);
        result.Should().NotBeNull();
        result!.Nombre.Should().Be("Test");
    }
}
