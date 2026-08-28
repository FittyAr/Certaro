using System;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Common;
using ElectroObraApp.ViewModels;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class EmpleadoEditViewModelTests
{
    private readonly IEmpleadoService _empleadoService;
    private readonly ILocalizationService _localizationService;
    private readonly EmpleadoEditViewModel _viewModel;

    public EmpleadoEditViewModelTests()
    {
        _empleadoService = Substitute.For<IEmpleadoService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());
        _viewModel = new EmpleadoEditViewModel(_empleadoService, _localizationService);
    }

    [Fact]
    public async Task SaveCommand_ShouldCreateEmpleado_WhenIdIsEmpty()
    {
        _viewModel.Empleado.Nombre = "Nuevo";
        _empleadoService.CreateAsync(_viewModel.Empleado).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _empleadoService.Received(1).CreateAsync(Arg.Any<EmpleadoDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public void FechaIngresoOffset_ShouldUpdateEmpleadoFechaIngreso()
    {
        var newDate = new DateTimeOffset(2026, 5, 1, 0, 0, 0, TimeSpan.Zero);

        _viewModel.FechaIngresoOffset = newDate;

        _viewModel.Empleado.FechaIngreso.Should().Be(newDate.DateTime);
    }

    [Fact]
    public async Task SaveCommand_ShouldUpdateEmpleado_WhenIdIsNotEmpty()
    {
        _viewModel.Empleado = new EmpleadoDto { Id = Guid.NewGuid(), Nombre = "Update" };
        _empleadoService.UpdateAsync(_viewModel.Empleado).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _empleadoService.Received(1).UpdateAsync(Arg.Any<EmpleadoDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public void CancelCommand_ShouldInvokeCloseRequestWithFalse()
    {
        bool closedWithSuccess = true;
        _viewModel.CloseRequest += (s, success) => closedWithSuccess = success;

        _viewModel.CancelCommand.Execute(null);

        closedWithSuccess.Should().BeFalse();
    }
}
