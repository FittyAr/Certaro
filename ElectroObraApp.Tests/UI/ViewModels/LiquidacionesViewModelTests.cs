using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.ViewModels;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class LiquidacionesViewModelTests
{
    private readonly ILiquidacionService _liquidacionService;
    private readonly IExportService _exportService;
    private readonly IMovimientoService _movimientoService;
    private readonly IEmpleadoService _empleadoService;
    private readonly IUserSettingsService _settingsService;
    private readonly LiquidacionesViewModel _viewModel;

    public LiquidacionesViewModelTests()
    {
        _liquidacionService = Substitute.For<ILiquidacionService>();
        _exportService = Substitute.For<IExportService>();
        _movimientoService = Substitute.For<IMovimientoService>();
        _empleadoService = Substitute.For<IEmpleadoService>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _viewModel = new LiquidacionesViewModel(
            _liquidacionService,
            _exportService,
            _movimientoService,
            _empleadoService,
            _settingsService,
            Substitute.For<INotificationService>(),
            Substitute.For<ILocalizationService>());
    }

    [Fact]
    public async Task LoadAsync_ShouldPopulateLiquidaciones()
    {
        // Arrange
        var list = new List<LiquidacionDto> { new() { EmpleadoNombre = "Test" } };
        _liquidacionService.GetAllAsync().Returns(list);

        // Act
        await _viewModel.LoadCommand.ExecuteAsync(null);

        // Assert
        _viewModel.Liquidaciones.Should().HaveCount(1);
        _viewModel.HasLiquidaciones.Should().BeTrue();
        _viewModel.ShowEmptyMessage.Should().BeFalse();
    }

    [Fact]
    public async Task LoadAsync_ShouldShowEmptyMessage_WhenNoData()
    {
        // Arrange
        _liquidacionService.GetAllAsync().Returns(new List<LiquidacionDto>());

        // Act
        await _viewModel.LoadCommand.ExecuteAsync(null);

        // Assert
        _viewModel.Liquidaciones.Should().BeEmpty();
        _viewModel.IsLoading.Should().BeFalse();
        _viewModel.ShowEmptyMessage.Should().BeTrue();
    }

    [Fact]
    public async Task ExportPdfCommand_ShouldInvokeExportService()
    {
        // Arrange
        var dto = new LiquidacionDto { EmpleadoId = Guid.NewGuid(), EmpleadoNombre = "Test", FechaFin = DateTime.Now };
        _movimientoService.GetAllAsync().Returns(new List<MovimientoDto>());
        _exportService.ExportLiquidacionToPdfAsync(Arg.Any<LiquidacionDto>(), Arg.Any<IEnumerable<MovimientoDto>>())
            .Returns(Task.FromResult(new byte[] { 1, 2, 3 }));

        // Act & Assert (Using try-catch because it tries to write to Desktop which might fail in some environments)
        try 
        {
            await _viewModel.ExportPdfCommand.ExecuteAsync(dto);
        }
        catch (Exception) { } // Ignore IO errors in tests

        // Assert
        await _exportService.Received(1).ExportLiquidacionToPdfAsync(Arg.Is<LiquidacionDto>(d => d.EmpleadoNombre == "Test"), Arg.Any<IEnumerable<MovimientoDto>>());
    }
}

