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

public class TrabajoEditViewModelTests
{
    private readonly ITrabajoService _trabajoService;
    private readonly IObraService _obraService;
    private readonly ILocalizationService _localizationService;
    private readonly TrabajoEditViewModel _viewModel;

    public TrabajoEditViewModelTests()
    {
        _trabajoService = Substitute.For<ITrabajoService>();
        _obraService = Substitute.For<IObraService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());
        _viewModel = new TrabajoEditViewModel(_trabajoService, _obraService, _localizationService);
    }

    [Fact]
    public async Task SaveCommand_ShouldCreate_WhenIdIsEmpty()
    {
        _viewModel.Trabajo.Descripcion = "Nuevo";
        _trabajoService.CreateAsync(_viewModel.Trabajo).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _trabajoService.Received(1).CreateAsync(Arg.Any<TrabajoDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public async Task SaveCommand_ShouldUpdate_WhenIdIsNotEmpty()
    {
        _viewModel.Trabajo = new TrabajoDto { Id = Guid.NewGuid(), Descripcion = "Update" };
        _trabajoService.UpdateAsync(_viewModel.Trabajo).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _trabajoService.Received(1).UpdateAsync(Arg.Any<TrabajoDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public void AddOrden_ShouldAddToList()
    {
        _viewModel.AddOrdenCommand.Execute(null);

        _viewModel.Trabajo.OrdenesTrabajo.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveOrden_ShouldRemoveFromList()
    {
        var orden = new OrdenTrabajoDto { Titulo = "Test" };
        _viewModel.Trabajo.OrdenesTrabajo.Add(orden);

        _viewModel.RemoveOrdenCommand.Execute(orden);

        _viewModel.Trabajo.OrdenesTrabajo.Should().BeEmpty();
    }

    [Fact]
    public void AddItem_ShouldAddToList()
    {
        var orden = new OrdenTrabajoDto { Titulo = "Test" };
        _viewModel.Trabajo.OrdenesTrabajo.Add(orden);

        _viewModel.AddItemCommand.Execute(orden);

        orden.Items.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveItem_ShouldRemoveFromList()
    {
        var orden = new OrdenTrabajoDto { Titulo = "Test" };
        var item = new OrdenTrabajoItemDto { Descripcion = "Item" };
        orden.Items.Add(item);
        _viewModel.Trabajo.OrdenesTrabajo.Add(orden);

        _viewModel.RemoveItemCommand.Execute(item);

        orden.Items.Should().BeEmpty();
    }

    [Fact]
    public async Task LoadData_ShouldPopulateObras()
    {
        var obras = new List<ObraDto> { new() { Nombre = "Obra 1" } };
        _obraService.GetAllAsync().Returns(obras);

        await _viewModel.LoadDataCommand.ExecuteAsync(null);

        _viewModel.Obras.Should().HaveCount(1);
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
