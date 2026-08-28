using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FluentAssertions;
using Microsoft.Extensions.DependencyInjection;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.ViewModels;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class ClientesViewModelTests
{
    private readonly IClienteService _clienteService;
    private readonly IUserSettingsService _settingsService;
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;
    private readonly ClientesViewModel _viewModel;

    public ClientesViewModelTests()
    {
        _clienteService = Substitute.For<IClienteService>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _serviceProvider = Substitute.For<IServiceProvider>();

        _settingsService.GetPageSize().Returns(10);
        _clienteService.GetAllAsync().Returns(new List<ClienteDto>());

        _viewModel = new ClientesViewModel(
            _clienteService,
            _settingsService,
            Substitute.For<IConfirmDialogService>(),
            _localizationService,
            _serviceProvider);
    }

    [Fact]
    public async Task LoadClientesAsync_ShouldPopulateClientes()
    {
        // Arrange
        var list = new List<ClienteDto> { new() { Nombre = "Test" } };
        _clienteService.GetAllAsync().Returns(list);

        // Act
        await _viewModel.LoadClientesAsync();

        // Assert
        _viewModel.Clientes.Should().HaveCount(1);
        _viewModel.Clientes[0].Nombre.Should().Be("Test");
    }

    [Fact]
    public void Add_ShouldSetIsEditingAndEditViewModel()
    {
        // Arrange
        var editVm = new ClienteEditViewModel(_clienteService, _settingsService, Substitute.For<ILocalizationService>());
        _serviceProvider.GetService(typeof(ClienteEditViewModel)).Returns(editVm);

        // Act
        _viewModel.AddCommand.Execute(null);

        // Assert
        _viewModel.IsEditing.Should().BeTrue();
        _viewModel.EditViewModel.Should().NotBeNull();
    }
}

