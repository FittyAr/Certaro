using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Enums;
using ElectroObraApp.Infrastructure.Repositories;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Repositories;

public class TrabajoRepositoryTests
{
    [Fact]
    public async Task GetAllWithDeepLoadAsync_ShouldLoadAllRelations()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new TrabajoRepository(context);

        var cliente = new Cliente { Nombre = "Cliente X" };
        var obra = new Obra { Nombre = "Obra X", Numero = 1, Cliente = cliente };
        var trabajo = new Trabajo { Descripcion = "Trabajo X", Obra = obra };
        var orden = new OrdenTrabajo { Titulo = "Orden 1", Trabajo = trabajo };
        orden.Items.Add(new OrdenTrabajoItem { Descripcion = "Item 1", OrdenTrabajo = orden });
        trabajo.OrdenesTrabajo.Add(orden);

        context.Add(trabajo);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetAllWithDeepLoadAsync();

        var item = result.First();
        item.Obra.Should().NotBeNull();
        item.Obra.Cliente.Should().NotBeNull();
        item.OrdenesTrabajo.Should().NotBeEmpty();
        item.OrdenesTrabajo.First().Items.Should().NotBeEmpty();
    }

    [Fact]
    public async Task GetByIdWithDeepLoadAsync_ShouldReturnEntityWithRelations()
    {
        using var context = await RepositoryTestHelper.CreateInMemoryContextAsync();
        var repository = new TrabajoRepository(context);

        var cliente = new Cliente { Nombre = "Cliente Y" };
        var obra = new Obra { Nombre = "Obra Y", Numero = 2, Cliente = cliente };
        var trabajo = new Trabajo { Descripcion = "Trabajo Y", Obra = obra, Estado = EstadoTrabajo.EnProceso };
        context.Add(trabajo);
        await context.SaveChangesAsync(RepositoryTestHelper.CancellationToken);

        var result = await repository.GetByIdWithDeepLoadAsync(trabajo.Id);

        result.Should().NotBeNull();
        result!.Obra.Should().NotBeNull();
        result.Obra.Cliente.Nombre.Should().Be("Cliente Y");
    }
}
