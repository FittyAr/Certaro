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

public class EmpleadosViewModelTests
{
    private readonly IEmpleadoService _empleadoService;
    private readonly IUserSettingsService _settingsService;
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;
    private readonly EmpleadosViewModel _viewModel;

    public EmpleadosViewModelTests()
    {
        _empleadoService = Substitute.For<IEmpleadoService>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _serviceProvider = Substitute.For<IServiceProvider>();

        _settingsService.GetPageSize().Returns(10);
        _empleadoService.GetAllAsync().Returns(new List<EmpleadoDto>());

        _viewModel = new EmpleadosViewModel(
            _empleadoService,
            _settingsService,
            Substitute.For<IConfirmDialogService>(),
            _localizationService,
            _serviceProvider);
    }

    [Fact]
    public async Task LoadEmpleadosAsync_ShouldPopulateEmpleados()
    {
        // Arrange
        var list = new List<EmpleadoDto> { new() { Nombre = "Pablo" } };
        _empleadoService.GetAllAsync().Returns(list);

        // Act
        await _viewModel.LoadEmpleadosAsync();

        // Assert
        _viewModel.Empleados.Should().HaveCount(1);
        _viewModel.Empleados[0].Nombre.Should().Be("Pablo");
    }

    [Fact]
    public void Add_ShouldSetIsEditingAndEditViewModel()
    {
        // Arrange
        var editVm = new EmpleadoEditViewModel(_empleadoService, Substitute.For<ILocalizationService>());
        _serviceProvider.GetService(typeof(EmpleadoEditViewModel)).Returns(editVm);

        // Act
        _viewModel.AddCommand.Execute(null);

        // Assert
        _viewModel.IsEditing.Should().BeTrue();
        _viewModel.EditViewModel.Should().NotBeNull();
    }
}

