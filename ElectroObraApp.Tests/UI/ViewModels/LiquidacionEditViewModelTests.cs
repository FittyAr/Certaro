using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Common;
using ElectroObraApp.ViewModels;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class LiquidacionEditViewModelTests
{
    private readonly ILiquidacionService _liquidacionService;
    private readonly IEmpleadoService _empleadoService;
    private readonly IUserSettingsService _settingsService;
    private readonly ILocalizationService _localizationService;
    private readonly LiquidacionEditViewModel _viewModel;

    public LiquidacionEditViewModelTests()
    {
        _liquidacionService = Substitute.For<ILiquidacionService>();
        _empleadoService = Substitute.For<IEmpleadoService>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());
        
        _settingsService.GetDefaultMultiplierSaturday().Returns(1.0m);
        _settingsService.GetDefaultMultiplierSunday().Returns(1.0m);
        _settingsService.GetDefaultMultiplierHoliday().Returns(1.0m);

        _viewModel = new LiquidacionEditViewModel(_liquidacionService, _empleadoService, _settingsService, _localizationService);
    }

    [Fact]
    public async Task SaveCommand_ShouldCreate_WhenSuccess()
    {
        _liquidacionService.CreateAsync(_viewModel.Liquidacion).Returns(Result<LiquidacionDto>.Success(new LiquidacionDto()));
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _liquidacionService.Received(1).CreateAsync(Arg.Any<LiquidacionDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public async Task SugerirCommand_ShouldUpdateLiquidacion()
    {
        var empleadoId = Guid.NewGuid();
        _viewModel.Liquidacion.EmpleadoId = empleadoId;
        _viewModel.Liquidacion.FechaInicio = new DateTime(2026, 5, 4);
        _viewModel.Liquidacion.FechaFin = new DateTime(2026, 5, 8);
        
        var sugerencia = new LiquidacionDto { TarifaAplicada = 1000, TotalAdelantos = 500 };
        _liquidacionService.SugerirLiquidacionAsync(empleadoId, Arg.Any<DateTime>(), Arg.Any<DateTime>(), Arg.Any<decimal>())
            .Returns(sugerencia);

        await _viewModel.SugerirCommand.ExecuteAsync(null);

        _viewModel.Liquidacion.TotalBruto.Should().Be(5000);
        _viewModel.Liquidacion.TotalNeto.Should().Be(4500);
        _viewModel.Liquidacion.DiasTrabajados.Should().Be(5);
    }

    [Fact]
    public async Task LoadData_ShouldPopulateEmpleados()
    {
        var emps = new List<EmpleadoDto> { new() { Nombre = "Emp 1" } };
        _empleadoService.GetAllAsync().Returns(emps);

        await _viewModel.LoadDataCommand.ExecuteAsync(null);

        _viewModel.Empleados.Should().HaveCount(1);
    }

    [Fact]
    public void CancelCommand_ShouldInvokeCloseRequestWithFalse()
    {
        bool closedWithSuccess = true;
        _viewModel.CloseRequest += (s, success) => closedWithSuccess = success;

        _viewModel.CancelCommand.Execute(null);

        closedWithSuccess.Should().BeFalse();
    }

    [Fact]
    public void FechaOffsets_ShouldUpdateDates()
    {
        var newStart = new DateTimeOffset(2026, 1, 1, 0, 0, 0, TimeSpan.Zero);
        var newEnd = new DateTimeOffset(2026, 1, 15, 0, 0, 0, TimeSpan.Zero);

        _viewModel.FechaInicioOffset = newStart.DateTime;
        _viewModel.FechaFinOffset = newEnd.DateTime;

        _viewModel.Liquidacion.FechaInicio.Should().Be(newStart.DateTime);
        _viewModel.Liquidacion.FechaFin.Should().Be(newEnd.DateTime);
    }
}
