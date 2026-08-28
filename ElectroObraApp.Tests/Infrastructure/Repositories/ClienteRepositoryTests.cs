using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Infrastructure.Repositories;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Repositories;

public class ClienteRepositoryTests
{
    [Fact]
    public async Task GetAllWithContactosAsync_ShouldLoadContactos()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new ClienteRepository(context);

        var cliente = new Cliente { Nombre = "Empresa X" };
        cliente.Contactos.Add(new ClienteContacto { Etiqueta = "Ventas", Email = "v@x.com" });
        context.Add(cliente);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetAllWithContactosAsync();

        var item = result.First();
        item.Contactos.Should().NotBeEmpty();
        item.Contactos.First().Etiqueta.Should().Be("Ventas");
    }
}
