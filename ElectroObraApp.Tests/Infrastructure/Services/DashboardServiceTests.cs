using System;
using System.Threading.Tasks;
using FluentAssertions;
using Microsoft.Extensions.Logging;
using NSubstitute;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Infrastructure.Services;
using ElectroObraApp.Infrastructure.Data;
using Microsoft.EntityFrameworkCore;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Services;

public class DashboardServiceTests
{
    [Fact]
    public async Task CheckDatabaseHealthAsync_WithValidContext_ShouldReturnTrue()
    {
        var options = new DbContextOptionsBuilder<ApplicationDbContext>()
            .UseSqlite($"Data Source=dashboard_test_{Guid.NewGuid()}.db")
            .Options;

        await using var context = new ApplicationDbContext(options);
        await context.Database.EnsureCreatedAsync();

        var service = new DashboardService(context, Substitute.For<ILogger<DashboardService>>());

        var healthy = await service.CheckDatabaseHealthAsync();

        healthy.Should().BeTrue();
    }

    [Fact]
    public async Task GetStatsAsync_ShouldReturnZeroTotalsWhenEmpty()
    {
        var options = new DbContextOptionsBuilder<ApplicationDbContext>()
            .UseSqlite($"Data Source=dashboard_stats_{Guid.NewGuid()}.db")
            .Options;

        await using var context = new ApplicationDbContext(options);
        await context.Database.EnsureCreatedAsync();

        var service = new DashboardService(context, Substitute.For<ILogger<DashboardService>>());

        var stats = await service.GetStatsAsync("Total");

        stats.TotalIngresos.Should().Be(0);
        stats.TotalGastos.Should().Be(0);
        stats.DatabaseHealthy.Should().BeTrue();
    }
}
