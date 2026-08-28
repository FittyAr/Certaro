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

public class ClienteEditViewModelTests
{
    private readonly IClienteService _clienteService;
    private readonly IUserSettingsService _settingsService;
    private readonly ILocalizationService _localizationService;
    private readonly ClienteEditViewModel _viewModel;

    public ClienteEditViewModelTests()
    {
        _clienteService = Substitute.For<IClienteService>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());
        _viewModel = new ClienteEditViewModel(_clienteService, _settingsService, _localizationService);
    }

    [Fact]
    public async Task SaveCommand_ShouldCreateClient_WhenIdIsEmpty()
    {
        _viewModel.Cliente.Nombre = "Nuevo";
        _clienteService.CreateAsync(_viewModel.Cliente).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _clienteService.Received(1).CreateAsync(Arg.Any<ClienteDto>());
        closed.Should().BeTrue();
    }

    [Fact]
    public void AddContact_ShouldAddToList()
    {
        _viewModel.AddContactCommand.Execute(null);

        _viewModel.Cliente.Contactos.Should().HaveCount(1);
        _viewModel.Cliente.Contactos[0].Etiqueta.Should().Be("General");
    }

    [Fact]
    public void RemoveContact_ShouldRemoveFromList()
    {
        var contacto = new ClienteContactoDto { Etiqueta = "Test" };
        _viewModel.Cliente.Contactos.Add(contacto);

        _viewModel.RemoveContactCommand.Execute(contacto);

        _viewModel.Cliente.Contactos.Should().BeEmpty();
    }

    [Fact]
    public async Task SaveCommand_ShouldUpdateClient_WhenIdIsNotEmpty()
    {
        _viewModel.Cliente = new ClienteDto { Id = Guid.NewGuid(), Nombre = "Update" };
        _clienteService.UpdateAsync(_viewModel.Cliente).Returns(Result.Success());
        bool closed = false;
        _viewModel.CloseRequest += (s, success) => closed = success;

        await _viewModel.SaveCommand.ExecuteAsync(null);

        await _clienteService.Received(1).UpdateAsync(Arg.Any<ClienteDto>());
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
