using System;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Services;
using ElectroObraApp.ViewModels;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class MainViewModelTests
{
    private readonly ILocalizationService _localizationService;
    private readonly IConfiguration _configuration;

    public MainViewModelTests()
    {
        _localizationService = Substitute.For<ILocalizationService>();
        _configuration = Substitute.For<IConfiguration>();
    }

    private static CuentaCorrienteViewModel CreateCuentaCorrienteViewModel() =>
        new(
            Substitute.For<IComercialService>(),
            Substitute.For<IClienteService>(),
            Substitute.For<ILocalizationService>());

    private static DashboardViewModel CreateDashboardViewModel() =>
        new(
            Substitute.For<IDashboardService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IDollarService>(),
            Substitute.For<INavigationService>(),
            Substitute.For<ILocalizationService>());

    private static NavigationService CreateNavigationService(IServiceProvider serviceProvider) =>
        new(serviceProvider);

    private static ClientesViewModel CreateClientesViewModel(IServiceProvider serviceProvider) =>
        new(
            Substitute.For<IClienteService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IConfirmDialogService>(),
            Substitute.For<ILocalizationService>(),
            serviceProvider,
            CreateCuentaCorrienteViewModel());

    private static EmpleadosViewModel CreateEmpleadosViewModel(IServiceProvider serviceProvider) =>
        new(
            Substitute.For<IEmpleadoService>(),
            Substitute.For<IUserSettingsService>(),
            Substitute.For<IConfirmDialogService>(),
            Substitute.For<ILocalizationService>(),
            serviceProvider);

    private static CommandPaletteViewModel CreateCommandPaletteViewModel(INavigationService navigationService) =>
        new(navigationService, Substitute.For<ILocalizationService>());

    [Fact]
    public void Constructor_ShouldSetGreetingFromLocalizationService()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var dashboardVm = CreateDashboardViewModel();
        var navigationService = CreateNavigationService(serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(NavigationService)).Returns(navigationService);
        _configuration["Application:Name"].Returns("Proyecto Pablito");
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());

        var seedService = Substitute.For<IDatabaseSeedService>();

        var vm = new MainViewModel(
            _localizationService,
            navigationService,
            seedService,
            _configuration,
            CreateCommandPaletteViewModel(navigationService));

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
        var navigationService = CreateNavigationService(serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(MovimientosViewModel)).Returns(movimientosVm);
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());

        var vm = new MainViewModel(
            _localizationService,
            navigationService,
            seedService,
            _configuration,
            CreateCommandPaletteViewModel(navigationService));

        vm.NavigateToCommand.Execute("movimientos");

        vm.CurrentPage.Should().Be(movimientosVm);
        vm.CurrentRoute.Should().Be("movimientos");
    }

    [Fact]
    public void NavigateToCertificados_ShouldSetCurrentPage()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var seedService = Substitute.For<IDatabaseSeedService>();
        var dashboardVm = CreateDashboardViewModel();
        var certificadosVm = new CertificadosViewModel(
            Substitute.For<ITrabajoService>(),
            Substitute.For<IExportService>(),
            Substitute.For<IFileSaveDialogService>(),
            Substitute.For<INotificationService>(),
            _localizationService);
        var navigationService = CreateNavigationService(serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(CertificadosViewModel)).Returns(certificadosVm);
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());

        var vm = new MainViewModel(
            _localizationService,
            navigationService,
            seedService,
            _configuration,
            CreateCommandPaletteViewModel(navigationService));

        vm.NavigateToCommand.Execute("certificados");

        vm.CurrentPage.Should().Be(certificadosVm);
        vm.CurrentRoute.Should().Be("certificados");
    }

    [Fact]
    public void NavigateToClientes_ShouldSetCurrentPage()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var seedService = Substitute.For<IDatabaseSeedService>();
        var dashboardVm = CreateDashboardViewModel();
        var clientesVm = CreateClientesViewModel(serviceProvider);
        var navigationService = CreateNavigationService(serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(ClientesViewModel)).Returns(clientesVm);
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());

        var vm = new MainViewModel(
            _localizationService,
            navigationService,
            seedService,
            _configuration,
            CreateCommandPaletteViewModel(navigationService));

        vm.NavigateToCommand.Execute("clientes");

        vm.CurrentPage.Should().Be(clientesVm);
    }

    [Fact]
    public void NavigateToEmpleados_ShouldSetCurrentPage()
    {
        var serviceProvider = Substitute.For<IServiceProvider>();
        var seedService = Substitute.For<IDatabaseSeedService>();
        var dashboardVm = CreateDashboardViewModel();
        var empleadosVm = CreateEmpleadosViewModel(serviceProvider);
        var navigationService = CreateNavigationService(serviceProvider);

        serviceProvider.GetService(typeof(DashboardViewModel)).Returns(dashboardVm);
        serviceProvider.GetService(typeof(EmpleadosViewModel)).Returns(empleadosVm);
        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());

        var vm = new MainViewModel(
            _localizationService,
            navigationService,
            seedService,
            _configuration,
            CreateCommandPaletteViewModel(navigationService));

        vm.NavigateToCommand.Execute("empleados");

        vm.CurrentPage.Should().Be(empleadosVm);
    }
}
