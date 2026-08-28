using FluentAssertions;
using ElectroObraApp.Core.Entities;
using Xunit;

namespace ElectroObraApp.Tests.Core.Entities;

public class OrdenTrabajoItemTests
{
    [Fact]
    public void PorcentajeAcumulado_ShouldBeSumOfAnteriorAndActual()
    {
        var item = new OrdenTrabajoItem { PorcentajeAnterior = 20, PorcentajeActual = 10 };

        item.PorcentajeAcumulado.Should().Be(30);
    }

    [Fact]
    public void CalculateSubtotals_ShouldCalculateBasedOnPercentage()
    {
        var item = new OrdenTrabajoItem 
        { 
            Cantidad = 100, 
            PrecioUnitario = 10, 
            PorcentajeActual = 50 
        };

        var (subtotalActual, subtotalAcumulado) = item.CalculateSubtotals();

        subtotalActual.Should().Be(500);
        subtotalAcumulado.Should().Be(500);
    }

    [Fact]
    public void CalculateSubtotals_ShouldCalculateBasedOnAccumulatedPercentage()
    {
        var item = new OrdenTrabajoItem 
        { 
            Cantidad = 100, 
            PrecioUnitario = 10, 
            PorcentajeAnterior = 20,
            PorcentajeActual = 30 
        };

        var (subtotalActual, subtotalAcumulado) = item.CalculateSubtotals();

        subtotalActual.Should().Be(300);
        subtotalAcumulado.Should().Be(500);
    }
}
