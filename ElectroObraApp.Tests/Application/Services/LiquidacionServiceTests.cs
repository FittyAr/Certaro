using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Expressions;
using System.Threading.Tasks;
using FluentAssertions;
using FluentValidation;
using Microsoft.Extensions.Logging;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.Services;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using Xunit;

namespace ElectroObraApp.Tests.Application.Services;

public class LiquidacionServiceTests
{
    private readonly IUnitOfWork _uow;
    private readonly ILogger<LiquidacionService> _logger;
    private readonly IUserSettingsService _settingsService;
    private readonly IHolidayService _holidayService;
    private readonly LiquidacionService _service;

    public LiquidacionServiceTests()
    {
        _uow = Substitute.For<IUnitOfWork>();
        _logger = Substitute.For<ILogger<LiquidacionService>>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _holidayService = Substitute.For<IHolidayService>();
        _holidayService.GetHolidaysAsync(Arg.Any<int>()).Returns(new List<HolidayModel>());
        var validator = Substitute.For<IValidator<LiquidacionDto>>();
        validator.ValidateAsync(Arg.Any<LiquidacionDto>(), Arg.Any<CancellationToken>())
            .Returns(new FluentValidation.Results.ValidationResult());
        _service = new LiquidacionService(_uow, _logger, validator, _settingsService, _holidayService);
    }

    [Fact]
    public async Task SugerirLiquidacionAsync_ShouldCalculateCorrectTotals_ExcludingWeekends()
    {
        var empleadoId = Guid.NewGuid();
        var inicio = new DateTime(2026, 5, 1);
        var fin = new DateTime(2026, 5, 15);
        var tarifaDiaria = 40000m;

        var empleado = new Empleado { Id = empleadoId, Nombre = "Juan Perez", TarifaDiaria = tarifaDiaria };
        _uow.Repository<Empleado>().GetByIdAsync(empleadoId).Returns(empleado);

        var adelantoTypeId = Guid.Parse("00000000-0000-0000-0000-000000000003");
        var adelantos = new List<Movimiento>
        {
            new() { Id = Guid.NewGuid(), TipoMovimientoId = adelantoTypeId, Monto = 50000, Cantidad = 1, Fecha = inicio.AddDays(2) },
            new() { Id = Guid.NewGuid(), TipoMovimientoId = adelantoTypeId, Monto = 30000, Cantidad = 1, Fecha = inicio.AddDays(5) }
        };

        _uow.Movimientos.FindAsync(Arg.Any<Expression<Func<Movimiento, bool>>>())
            .Returns(adelantos);

        var result = await _service.SugerirLiquidacionAsync(empleadoId, inicio, fin, 0);

        result.Should().NotBeNull();
        result.EmpleadoId.Should().Be(empleadoId);
        result.DiasTrabajados.Should().Be(11);
        result.TotalBruto.Should().Be(11 * tarifaDiaria);
        result.TotalAdelantos.Should().Be(80000);
        result.TotalNeto.Should().Be(360000);
    }

    [Fact]
    public async Task GetAllAsync_ShouldReturnList()
    {
        var list = new List<Liquidacion> { new() { Empleado = new Empleado { Nombre = "Juan" } } };
        _uow.Liquidaciones.GetAllWithEmpleadoAsync().Returns(list);

        var result = await _service.GetAllAsync();

        result.Should().HaveCount(1);
    }

    [Fact]
    public async Task CreateAsync_ShouldReturnDto_WhenSuccess()
    {
        _uow.SaveChangesAsync().Returns(1);

        var result = await _service.CreateAsync(new LiquidacionDto());

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().NotBeNull();
    }

    [Fact]
    public async Task Sugerir_ShouldHandleNoAdelantos()
    {
        var empleadoId = Guid.NewGuid();
        var empleado = new Empleado { Id = empleadoId, TarifaDiaria = 1000 };
        _uow.Repository<Empleado>().GetByIdAsync(empleadoId).Returns(empleado);
        _uow.Movimientos.FindAsync(Arg.Any<Expression<Func<Movimiento, bool>>>()).Returns(new List<Movimiento>());

        var result = await _service.SugerirLiquidacionAsync(empleadoId, DateTime.Now, DateTime.Now, 1);

        result.TotalAdelantos.Should().Be(0);
        result.TotalNeto.Should().Be(1000);
    }
}
