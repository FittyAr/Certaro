using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Services;
using ElectroObraApp.Application.Validators;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using Microsoft.Extensions.Logging;
using Xunit;

namespace ElectroObraApp.Tests.Application.Services;

public class MovimientoServiceTests
{
    private readonly IUnitOfWork _uow;
    private readonly IRepository<Movimiento> _repo;
    private readonly IMovimientoRepository _movimientoRepo;
    private readonly ILogger<MovimientoService> _logger;
    private readonly MovimientoService _service;

    public MovimientoServiceTests()
    {
        _uow = Substitute.For<IUnitOfWork>();
        _repo = Substitute.For<IRepository<Movimiento>>();
        _movimientoRepo = Substitute.For<IMovimientoRepository>();
        _uow.Repository<Movimiento>().Returns(_repo);
        _uow.Movimientos.Returns(_movimientoRepo);
        _logger = Substitute.For<ILogger<MovimientoService>>();
        _service = new MovimientoService(_uow, _logger, new MovimientoValidator());
    }

    private static MovimientoDto ValidDto() => new()
    {
        Concepto = "Test",
        Monto = 100,
        Cantidad = 1,
        TipoMovimientoId = Guid.NewGuid()
    };

    [Fact]
    public async Task GetAllAsync_ShouldReturnList()
    {
        var list = new List<Movimiento> { new() { Concepto = "Sueldo" } };
        _movimientoRepo.GetAllWithIncludesAsync().Returns(list);

        var result = await _service.GetAllAsync();

        result.Should().HaveCount(1);
        result.First().Concepto.Should().Be("Sueldo");
    }

    [Fact]
    public async Task GetPagedAsync_ShouldReturnPagedResult()
    {
        var paged = new PagedResult<Movimiento>
        {
            Items = new List<Movimiento> { new() { Concepto = "Filtrado" } },
            TotalCount = 1,
            PageNumber = 1,
            PageSize = 10
        };
        _movimientoRepo.GetPagedAsync(Arg.Any<ElectroObraApp.Core.Specifications.ISpecification<Movimiento>>()).Returns(paged);

        var result = await _service.GetPagedAsync(new MovimientoFilterDto { PageNumber = 1, PageSize = 10 });

        result.Items.Should().HaveCount(1);
        result.Items.First().Concepto.Should().Be("Filtrado");
        result.TotalCount.Should().Be(1);
    }

    [Fact]
    public async Task CreateAsync_ShouldReturnSuccess_WhenSaveIsSuccessful()
    {
        _uow.SaveChangesAsync().Returns(1);

        var result = await _service.CreateAsync(ValidDto());

        result.IsSuccess.Should().BeTrue();
        await _repo.Received(1).AddAsync(Arg.Any<Movimiento>());
        await _uow.Received(1).SaveChangesAsync();
    }

    [Fact]
    public async Task CreateAsync_ShouldReturnFailure_WhenValidationFails()
    {
        var result = await _service.CreateAsync(new MovimientoDto());

        result.IsSuccess.Should().BeFalse();
        result.Errors.Should().NotBeEmpty();
        await _repo.DidNotReceive().AddAsync(Arg.Any<Movimiento>());
    }

    [Fact]
    public async Task DeleteAsync_ShouldReturnFailure_WhenEntityDoesNotExist()
    {
        var id = Guid.NewGuid();
        _repo.GetByIdAsync(id).Returns((Movimiento?)null);

        var result = await _service.DeleteAsync(id);

        result.IsSuccess.Should().BeFalse();
        _repo.DidNotReceive().Remove(Arg.Any<Movimiento>());
    }
}
