using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Common;
using ElectroObraApp.ViewModels;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class MovimientosViewModelTests
{
    private readonly IMovimientoService _movimientoService;
    private readonly ITipoMovimientoService _tipoMovimientoService;
    private readonly IExportService _exportService;
    private readonly IUserSettingsService _settingsService;
    private readonly IServiceProvider _serviceProvider;
    private readonly MovimientosViewModel _vm;

    public MovimientosViewModelTests()
    {
        _movimientoService = Substitute.For<IMovimientoService>();
        _tipoMovimientoService = Substitute.For<ITipoMovimientoService>();
        _exportService = Substitute.For<IExportService>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _serviceProvider = Substitute.For<IServiceProvider>();

        _settingsService.GetPageSize().Returns(10);
        _tipoMovimientoService.GetAllAsync().Returns(Array.Empty<TipoMovimientoDto>());

        _vm = new MovimientosViewModel(
            _movimientoService,
            _tipoMovimientoService,
            _exportService,
            _settingsService,
            Substitute.For<IConfirmDialogService>(),
            Substitute.For<ILocalizationService>(),
            Substitute.For<IFileSaveDialogService>(),
            _serviceProvider);
    }

    [Fact]
    public async Task LoadMovimientosCommand_ShouldPopulateMovimientos()
    {
        var paged = new PagedResult<MovimientoDto>
        {
            Items = new List<MovimientoDto> { new() { Concepto = "Test" } },
            TotalCount = 1,
            PageNumber = 1,
            PageSize = 10
        };
        _movimientoService.GetPagedAsync(Arg.Any<MovimientoFilterDto>()).Returns(paged);

        await _vm.LoadMovimientosCommand.ExecuteAsync(null);

        _vm.Movimientos.Should().HaveCount(1);
        _vm.Movimientos.First().Concepto.Should().Be("Test");
        _vm.IsEmpty.Should().BeFalse();
    }

    [Fact]
    public async Task FiltroConcepto_ShouldFilterList()
    {
        var paged = new PagedResult<MovimientoDto>
        {
            Items = new List<MovimientoDto> { new() { Concepto = "Agua" } },
            TotalCount = 1,
            PageNumber = 1,
            PageSize = 10
        };
        _movimientoService.GetPagedAsync(Arg.Any<MovimientoFilterDto>()).Returns(paged);

        await _vm.LoadMovimientosCommand.ExecuteAsync(null);
        _vm.FiltroConcepto = "Agua";
        await Task.Delay(350);

        _vm.Movimientos.Should().HaveCount(1);
        _vm.Movimientos.First().Concepto.Should().Be("Agua");
    }
}
