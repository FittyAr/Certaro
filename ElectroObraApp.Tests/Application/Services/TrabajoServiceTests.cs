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

public class TrabajoServiceTests
{
    private readonly IUnitOfWork _uow;
    private readonly IRepository<Trabajo> _repo;
    private readonly ITrabajoRepository _trabajoRepo;
    private readonly ILogger<TrabajoService> _logger;
    private readonly TrabajoService _service;

    public TrabajoServiceTests()
    {
        _uow = Substitute.For<IUnitOfWork>();
        _repo = Substitute.For<IRepository<Trabajo>>();
        _trabajoRepo = Substitute.For<ITrabajoRepository>();
        _uow.Repository<Trabajo>().Returns(_repo);
        _uow.Trabajos.Returns(_trabajoRepo);
        _logger = Substitute.For<ILogger<TrabajoService>>();
        var validator = Substitute.For<IValidator<TrabajoDto>>();
        validator.ValidateAsync(Arg.Any<TrabajoDto>(), Arg.Any<CancellationToken>())
            .Returns(new FluentValidation.Results.ValidationResult());
        _service = new TrabajoService(_uow, _logger, validator);
    }

    [Fact]
    public async Task GetAllAsync_ShouldReturnList()
    {
        var list = new List<Trabajo> { new() { Descripcion = "Trabajo 1" } };
        _trabajoRepo.GetAllWithDeepLoadAsync().Returns(list);

        var result = await _service.GetAllAsync();

        result.Should().HaveCount(1);
        result.First().Descripcion.Should().Be("Trabajo 1");
    }

    [Fact]
    public async Task GetByIdAsync_ShouldReturnDto_WhenFound()
    {
        var id = Guid.NewGuid();
        var entity = new Trabajo { Id = id, Descripcion = "Test" };
        _trabajoRepo.GetByIdWithDeepLoadAsync(id).Returns(entity);

        var result = await _service.GetByIdAsync(id);

        result.Should().NotBeNull();
        result!.Descripcion.Should().Be("Test");
    }

    [Fact]
    public async Task CreateAsync_ShouldReturnTrue_WhenSuccess()
    {
        var dto = new TrabajoDto { Descripcion = "Nuevo", ClienteId = Guid.NewGuid() };
        _uow.SaveChangesAsync().Returns(1);

        var result = await _service.CreateAsync(dto);

        result.IsSuccess.Should().BeTrue();
        await _repo.Received(1).AddAsync(Arg.Any<Trabajo>());
    }
}
