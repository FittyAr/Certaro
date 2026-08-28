using System;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.ViewModels;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class MainViewModelTests
{
    private readonly ILocalizationService _localizationService;
    private readonly IConfiguration _configuration;
    private readonly IConfirmDialogService _confirmDialogService;

    public MainViewModelTests()
    {
        _localizationService = Substitute.For<ILocalizationService>();
        _configuration = Substitute.For<IConfiguration>();
        _confirmDialogService = Substitute.For<IConfirmDialogService>();
    }

    private static DashboardViewModel CreateDashboardViewModel() =>
        new(
            Substitute.For<IDashboardService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IDollarService>());

    private static TrabajosViewModel CreateTrabajosViewModel(IServiceProvider serviceProvider) =>
        new(
            Substitute.For<ITrabajoService>(),
            Substitute.For<IClienteService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IConfirmDialogService>(),
            Substitute.For<ILocalizationService>(),
            serviceProvider);

    private static ClientesViewModel CreateClientesViewModel(IServiceProvider serviceProvider) =>
        new(
            Substitute.For<IClienteService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IConfirmDialogService>(),
            Substitute.For<ILocalizationService>(),
            serviceProvider);

    private static EmpleadosViewModel CreateEmpleadosViewModel(IServiceProvider serviceProvider) =>
        new(
            Substitute.For<IEmpleadoService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IConfirmDialogService>(),
            Substitute.For<ILocalizationService>(),
            serviceProvider);

    [Fact]
    public void Constructor_ShouldSetGreetingFromLocalizationService()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var dashboardVm = CreateDashboardViewModel();

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        _configuration["Application:Name"].Returns("Proyecto Pablito");
        _localizationService.GetString("General.AppName").Returns("Proyecto Pablito");

        var seedService = Substitute.For<IDatabaseSeedService>();

        var vm = new MainViewModel(_localizationService, serviceProvider, seedService, _configuration);

        vm.Greeting.Should().Be("Proyecto Pablito");
    }

    [Fact]
    public void NavigateToMovimientos_ShouldSetCurrentPage()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var seedService = Substitute.For<IDatabaseSeedService>();
        var movimientoService = Substitute.For<IMovimientoService>();
        var dashboardVm = CreateDashboardViewModel();
        var movimientosVm = new MovimientosViewModel(
            movimientoService,
            Substitute.For<ITipoMovimientoService>(),
            Substitute.For<IExportService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IConfirmDialogService>(),
            Substitute.For<ILocalizationService>(),
            Substitute.For<IFileSaveDialogService>(),
            serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(MovimientosViewModel)).Returns(movimientosVm);
        _localizationService.GetString(Arg.Any<string>()).Returns("Test");

        var vm = new MainViewModel(_localizationService, serviceProvider, seedService, _configuration);

        vm.NavigateToMovimientosCommand.Execute(null);

        vm.CurrentPage.Should().Be(movimientosVm);
    }

    [Fact]
    public void NavigateToTrabajos_ShouldSetCurrentPage()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var seedService = Substitute.For<IDatabaseSeedService>();
        var dashboardVm = CreateDashboardViewModel();
        var trabajosVm = CreateTrabajosViewModel(serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(TrabajosViewModel)).Returns(trabajosVm);
        _localizationService.GetString(Arg.Any<string>()).Returns("Test");

        var vm = new MainViewModel(_localizationService, serviceProvider, seedService, _configuration);

        vm.NavigateToTrabajosCommand.Execute(null);

        vm.CurrentPage.Should().Be(trabajosVm);
    }

    [Fact]
    public void NavigateToClientes_ShouldSetCurrentPage()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var seedService = Substitute.For<IDatabaseSeedService>();
        var dashboardVm = CreateDashboardViewModel();
        var clientesVm = CreateClientesViewModel(serviceProvider);
        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(ClientesViewModel)).Returns(clientesVm);
        _localizationService.GetString(Arg.Any<string>()).Returns("Test");

        var vm = new MainViewModel(_localizationService, serviceProvider, seedService, _configuration);

        vm.NavigateToClientesCommand.Execute(null);

        vm.CurrentPage.Should().Be(clientesVm);
    }

    [Fact]
    public void NavigateToEmpleados_ShouldSetCurrentPage()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var seedService = Substitute.For<IDatabaseSeedService>();
        var dashboardVm = CreateDashboardViewModel();
        var empleadosVm = CreateEmpleadosViewModel(serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(EmpleadosViewModel)).Returns(empleadosVm);
        _localizationService.GetString(Arg.Any<string>()).Returns("Test");

        var vm = new MainViewModel(_localizationService, serviceProvider, seedService, _configuration);

        vm.NavigateToEmpleadosCommand.Execute(null);

        vm.CurrentPage.Should().Be(empleadosVm);
    }
}
