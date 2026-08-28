using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.ViewModels;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class DashboardViewModelTests
{
    private readonly IDashboardService _dashboardService;
    private readonly IUserSettingsService _settingsService;
    private readonly IDollarService _dollarService;

    public DashboardViewModelTests()
    {
        _dashboardService = Substitute.For<IDashboardService>();
        _settingsService = Substitute.For<IUserSettingsService>();
        _dollarService = Substitute.For<IDollarService>();

        _settingsService.GetDashboardPeriod().Returns("Mes");
        _settingsService.GetIsPrivacyMode().Returns(false);
        _settingsService.GetAutoUpdateDollar().Returns(false);
    }

    private DashboardViewModel CreateDashboardViewModel() =>
        new(_dashboardService, _settingsService, _dollarService);

    [Fact]
    public async Task Constructor_ShouldCalculateTotals()
    {
        _dashboardService.GetStatsAsync(Arg.Any<string>()).Returns(new DashboardStatsDto
        {
            TotalIngresos = 100,
            TotalGastos = 40
        });

        var vm = CreateDashboardViewModel();
        await vm.LoadStatsCommand.ExecuteAsync(null);

        vm.TotalIngresos.Should().Be(100);
        vm.TotalGastos.Should().Be(40);
        vm.Balance.Should().Be(60);
    }

    [Fact]
    public async Task LoadStats_ShouldHandleZeroMovements()
    {
        _dashboardService.GetStatsAsync(Arg.Any<string>()).Returns(new DashboardStatsDto());

        var vm = CreateDashboardViewModel();
        await vm.LoadStatsCommand.ExecuteAsync(null);

        vm.TotalIngresos.Should().Be(0);
        vm.TotalGastos.Should().Be(0);
        vm.Balance.Should().Be(0);
    }

    [Fact]
    public async Task LoadStats_ShouldHandleOnlyIncome()
    {
        _dashboardService.GetStatsAsync(Arg.Any<string>()).Returns(new DashboardStatsDto
        {
            TotalIngresos = 200
        });

        var vm = CreateDashboardViewModel();
        await vm.LoadStatsCommand.ExecuteAsync(null);

        vm.TotalIngresos.Should().Be(200);
        vm.TotalGastos.Should().Be(0);
        vm.Balance.Should().Be(200);
    }
}
