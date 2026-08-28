using System.Threading.Tasks;
using FluentAssertions;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Infrastructure.Repositories;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Repositories;

public class RepositoryTests
{
    [Fact]
    public async Task AddAsync_ShouldAddEntityToDatabase()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new Repository<Categoria>(context);
        var categoria = new Categoria { Nombre = "Nueva" };

        await repository.AddAsync(categoria);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetByIdAsync(categoria.Id);
        result.Should().NotBeNull();
        result!.Nombre.Should().Be("Nueva");
    }

    [Fact]
    public async Task Update_ShouldModifyExistingEntity()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new Repository<Categoria>(context);
        var categoria = new Categoria { Nombre = "Original" };
        await repository.AddAsync(categoria);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        categoria.Nombre = "Modificada";
        repository.Update(categoria);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetByIdAsync(categoria.Id);
        result!.Nombre.Should().Be("Modificada");
    }

    [Fact]
    public async Task Remove_ShouldRemoveEntity()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new Repository<Categoria>(context);
        var categoria = new Categoria { Nombre = "ABorrar" };
        await repository.AddAsync(categoria);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        repository.Remove(categoria);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetByIdAsync(categoria.Id);
        result.Should().BeNull();
    }
}
