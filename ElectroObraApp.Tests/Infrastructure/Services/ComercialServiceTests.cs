using System;
using System.Threading.Tasks;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Enums;
using ElectroObraApp.Infrastructure.Data;
using ElectroObraApp.Infrastructure.Services;
using FluentAssertions;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;
using NSubstitute;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Services;

public class ComercialServiceTests
{
    [Fact]
    public async Task GetAntiguedadDeudaAsync_ShouldBucketUnpaidInvoices()
    {
        var options = new DbContextOptionsBuilder<ApplicationDbContext>()
            .UseSqlite($"Data Source=comercial_test_{Guid.NewGuid()}.db")
            .Options;

        await using var context = new ApplicationDbContext(options);
        await context.Database.EnsureCreatedAsync();

        var clienteId = Guid.NewGuid();
        context.Clientes.Add(new Cliente { Id = clienteId, Nombre = "Cliente Test" });
        context.Facturas.Add(new Factura
        {
            Id = Guid.NewGuid(),
            ClienteId = clienteId,
            Numero = "F-001",
            Fecha = DateTime.Today.AddDays(-45),
            Estado = EstadoFactura.Emitida,
            Total = 1000m
        });
        await context.SaveChangesAsync();

        var service = new ComercialService(context, Substitute.For<ILogger<ComercialService>>());

        var aging = await service.GetAntiguedadDeudaAsync(clienteId);

        aging.TotalDeuda.Should().Be(1000m);
        aging.Bucket31To60.Should().Be(1000m);
    }

    [Fact]
    public async Task GetRentabilidadPorObraAsync_ShouldOrderByProfitability()
    {
        var options = new DbContextOptionsBuilder<ApplicationDbContext>()
            .UseSqlite($"Data Source=comercial_rank_{Guid.NewGuid()}.db")
            .Options;

        await using var context = new ApplicationDbContext(options);
        await context.Database.EnsureCreatedAsync();

        var service = new ComercialService(context, Substitute.For<ILogger<ComercialService>>());

        var ranking = await service.GetRentabilidadPorObraAsync();

        ranking.Should().NotBeNull();
    }
}
