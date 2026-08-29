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

public class MovimientoEditViewModelTests
{
    private readonly IMovimientoService _movimientoService;
    private readonly ICategoriaService _categoriaService;
    private readonly ITipoMovimientoService _tipoMovimientoService;
    private readonly IEmpleadoService _empleadoService;
    private readonly IClienteService _clienteService;
    private readonly IObraService _obraService;
    private readonly ITrabajoService _trabajoService;
    private readonly IFacturaService _facturaService;
    private readonly ILocalizationService _localizationService;
    private readonly MovimientoEditViewModel _viewModel;

    public MovimientoEditViewModelTests()
    {
        _movimientoService = Substitute.For<IMovimientoService>();
        _categoriaService = Substitute.For<ICategoriaService>();
        _tipoMovimientoService = Substitute.For<ITipoMovimientoService>();
        _empleadoService = Substitute.For<IEmpleadoService>();
        _clienteService = Substitute.For<IClienteService>();
        _obraService = Substitute.For<IObraService>();
        _trabajoService = Substitute.For<ITrabajoService>();
        _facturaService = Substitute.For<IFacturaService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());
        _viewModel = new MovimientoEditViewModel(
            _movimientoService,
            _categoriaService,
            _tipoMovimientoService,
            _empleadoService,
            _clienteService,
            _obraService,
            _trabajoService,
            _facturaService,
            _localizationService);
    }

    [Fact]
    public async Task SaveCommand_ShouldCreate_WhenIdIsEmpty()
    {
        _viewModel.Movimiento.Monto = 100;
        _viewModel.Movimiento.Concepto = "Test";
        _viewModel.Movimiento.Cantidad = 1;
        _viewModel.Movimiento.TipoMovimientoId = Guid.NewGuid();
        _movimientoService.CreateAsync(_viewModel.Movimiento).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _movimientoService.Received(1).CreateAsync(Arg.Any<MovimientoDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public async Task SaveCommand_ShouldUpdate_WhenIdIsNotEmpty()
    {
        _viewModel.Movimiento = new MovimientoDto { Id = Guid.NewGuid(), Monto = 200, Concepto = "Test", Cantidad = 1, TipoMovimientoId = Guid.NewGuid() };
        _movimientoService.UpdateAsync(_viewModel.Movimiento).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _movimientoService.Received(1).UpdateAsync(Arg.Any<MovimientoDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public async Task SaveCommand_ShouldShowError_WhenServiceFails()
    {
        _viewModel.Movimiento.Monto = 100;
        _movimientoService.CreateAsync(_viewModel.Movimiento).Returns(Result.Failure("Validation.Movimiento.ConceptoRequired"));
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        closed.Should().BeFalse();
        _viewModel.ErrorMessage.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public async Task LoadData_ShouldPopulateCollections()
    {
        var cats = new System.Collections.Generic.List<CategoriaDto> { new() { Nombre = "Cat 1" } };
        var tipos = new System.Collections.Generic.List<TipoMovimientoDto> { new() { Nombre = "Tipo 1" } };
        _categoriaService.GetAllAsync().Returns(cats);
        _tipoMovimientoService.GetAllAsync().Returns(tipos);
        _empleadoService.GetAllAsync().Returns(new System.Collections.Generic.List<EmpleadoDto>());
        _clienteService.GetAllAsync().Returns(new System.Collections.Generic.List<ClienteDto>());
        _obraService.GetAllAsync().Returns(new System.Collections.Generic.List<ObraDto>());
        _trabajoService.GetAllAsync().Returns(new System.Collections.Generic.List<TrabajoDto>());
        _facturaService.GetAllAsync().Returns(new System.Collections.Generic.List<FacturaDto>());

        await _viewModel.LoadDataCommand.ExecuteAsync(null);

        _viewModel.Categorias.Should().HaveCount(1);
        _viewModel.TiposMovimiento.Should().HaveCount(1);
    }

    [Fact]
    public async Task LoadData_ShouldSetDefaultTipo_WhenNew()
    {
        var tipoId = Guid.NewGuid();
        var tipos = new System.Collections.Generic.List<TipoMovimientoDto> { new() { Id = tipoId, Nombre = "Tipo 1" } };
        _tipoMovimientoService.GetAllAsync().Returns(tipos);
        _categoriaService.GetAllAsync().Returns(new System.Collections.Generic.List<CategoriaDto>());
        _empleadoService.GetAllAsync().Returns(new System.Collections.Generic.List<EmpleadoDto>());
        _clienteService.GetAllAsync().Returns(new System.Collections.Generic.List<ClienteDto>());
        _obraService.GetAllAsync().Returns(new System.Collections.Generic.List<ObraDto>());
        _trabajoService.GetAllAsync().Returns(new System.Collections.Generic.List<TrabajoDto>());
        _facturaService.GetAllAsync().Returns(new System.Collections.Generic.List<FacturaDto>());

        await _viewModel.LoadDataCommand.ExecuteAsync(null);

        _viewModel.Movimiento.TipoMovimientoId.Should().Be(tipoId);
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
