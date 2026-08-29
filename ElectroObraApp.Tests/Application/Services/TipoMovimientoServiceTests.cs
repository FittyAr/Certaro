using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using FluentValidation;
using Microsoft.Extensions.Logging.Abstractions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Services;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using Xunit;

namespace ElectroObraApp.Tests.Application.Services;

public class TipoMovimientoServiceTests
{
    private readonly IUnitOfWork _uow;
    private readonly IRepository<TipoMovimiento> _repo;
    private readonly TipoMovimientoService _service;

    public TipoMovimientoServiceTests()
    {
        _uow = Substitute.For<IUnitOfWork>();
        _repo = Substitute.For<IRepository<TipoMovimiento>>();
        _uow.Repository<TipoMovimiento>().Returns(_repo);
        _service = new TipoMovimientoService(
            _uow,
            NullLogger<TipoMovimientoService>.Instance,
            Substitute.For<IValidator<TipoMovimientoDto>>());
    }

    [Fact]
    public async Task GetAllAsync_ShouldReturnList()
    {
        var list = new List<TipoMovimiento> { new() { Nombre = "Efectivo" } };
        _repo.GetAllAsync().Returns(list);

        var result = await _service.GetAllAsync();

        result.Should().HaveCount(1);
        result.First().Nombre.Should().Be("Efectivo");
    }
}
