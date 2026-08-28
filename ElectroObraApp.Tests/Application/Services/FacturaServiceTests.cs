using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FluentAssertions;
using Microsoft.Extensions.Logging;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Services;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Enums;
using ElectroObraApp.Core.Interfaces;
using FluentValidation;
using Xunit;

namespace ElectroObraApp.Tests.Application.Services;

public class FacturaServiceTests
{
    private readonly IUnitOfWork _uow;
    private readonly IFacturaRepository _facturaRepo;
    private readonly IValidator<FacturaDto> _validator;
    private readonly FacturaService _service;

    public FacturaServiceTests()
    {
        _uow = Substitute.For<IUnitOfWork>();
        _facturaRepo = Substitute.For<IFacturaRepository>();
        _uow.Facturas.Returns(_facturaRepo);
        _uow.Repository<Factura>().Returns(Substitute.For<IRepository<Factura>>());
        _validator = Substitute.For<IValidator<FacturaDto>>();
        _validator.ValidateAsync(Arg.Any<FacturaDto>(), default)
            .Returns(new FluentValidation.Results.ValidationResult());
        _service = new FacturaService(_uow, Substitute.For<ILogger<FacturaService>>(), _validator);
    }

    [Fact]
    public async Task GetAllAsync_ShouldReturnMappedList()
    {
        var list = new List<Factura>
        {
            new() { Numero = "F-001", Cliente = new Cliente { Nombre = "Cliente A" }, Total = 1210m }
        };
        _facturaRepo.GetAllWithClienteAsync().Returns(list);

        var result = await _service.GetAllAsync();

        result.Should().HaveCount(1);
        result.First().Numero.Should().Be("F-001");
        result.First().ClienteNombre.Should().Be("Cliente A");
    }

    [Fact]
    public async Task CreateAsync_ShouldCalculateTotal()
    {
        var repo = Substitute.For<IRepository<Factura>>();
        _uow.Repository<Factura>().Returns(repo);
        _uow.SaveChangesAsync().Returns(1);

        var dto = new FacturaDto
        {
            Numero = "F-002",
            ClienteId = Guid.NewGuid(),
            Subtotal = 1000m,
            Iva = 210m
        };

        var result = await _service.CreateAsync(dto);

        result.IsSuccess.Should().BeTrue();
        dto.Total.Should().Be(1210m);
        await repo.Received(1).AddAsync(Arg.Any<Factura>());
    }
}
