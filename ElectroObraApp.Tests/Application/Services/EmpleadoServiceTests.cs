using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using FluentValidation;
using Microsoft.Extensions.Logging;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Services;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using Xunit;

namespace ElectroObraApp.Tests.Application.Services;

public class EmpleadoServiceTests
{
    private readonly IUnitOfWork _uow;
    private readonly IRepository<Empleado> _repo;
    private readonly ILogger<EmpleadoService> _logger;
    private readonly EmpleadoService _service;

    public EmpleadoServiceTests()
    {
        _uow = Substitute.For<IUnitOfWork>();
        _repo = Substitute.For<IRepository<Empleado>>();
        _uow.Repository<Empleado>().Returns(_repo);
        _logger = Substitute.For<ILogger<EmpleadoService>>();
        var validator = Substitute.For<IValidator<EmpleadoDto>>();
        validator.ValidateAsync(Arg.Any<EmpleadoDto>(), Arg.Any<CancellationToken>())
            .Returns(new FluentValidation.Results.ValidationResult());
        _service = new EmpleadoService(_uow, _logger, validator);
    }

    [Fact]
    public async Task GetAllAsync_ShouldReturnList()
    {
        var list = new List<Empleado> { new() { Nombre = "Pablo" } };
        _repo.GetAllAsync().Returns(list);

        var result = await _service.GetAllAsync();

        result.Should().HaveCount(1);
        result.First().Nombre.Should().Be("Pablo");
    }

    [Fact]
    public async Task CreateAsync_ShouldReturnTrue_WhenSuccess()
    {
        var dto = new EmpleadoDto { Nombre = "Nuevo" };
        _uow.SaveChangesAsync().Returns(1);

        var result = await _service.CreateAsync(dto);

        result.IsSuccess.Should().BeTrue();
        await _repo.Received(1).AddAsync(Arg.Any<Empleado>());
    }

    [Fact]
    public async Task UpdateAsync_ShouldReturnTrue_WhenSuccess()
    {
        var dto = new EmpleadoDto { Id = Guid.NewGuid(), Nombre = "Update" };
        _repo.GetByIdAsync(dto.Id).Returns(new Empleado { Id = dto.Id, Nombre = "Original" });
        _uow.SaveChangesAsync().Returns(1);

        var result = await _service.UpdateAsync(dto);

        result.IsSuccess.Should().BeTrue();
        _repo.Received(1).Update(Arg.Any<Empleado>());
    }

    [Fact]
    public async Task DeleteAsync_ShouldReturnFalse_WhenNotFound()
    {
        var id = Guid.NewGuid();
        _repo.GetByIdAsync(id).Returns((Empleado?)null);

        var result = await _service.DeleteAsync(id);

        result.IsSuccess.Should().BeFalse();
        _repo.DidNotReceive().Update(Arg.Any<Empleado>());
    }
}
