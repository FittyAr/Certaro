using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Infrastructure.Services;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Services;

public class ExportServiceTests
{
    private readonly ExportService _service;

    public ExportServiceTests()
    {
        _service = new ExportService(Substitute.For<IUserSettingsService>());
    }

    [Fact]
    public async Task ExportMovimientosToPdfAsync_ShouldReturnNonEmptyByteArray()
    {
        // Arrange
        var data = new List<MovimientoDto>
        {
            new() { Fecha = DateTime.Now, Concepto = "Test", Total = 100, TipoMovimientoNombre = "Ingreso" }
        };

        // Act
        var result = await _service.ExportMovimientosToPdfAsync(data);

        // Assert
        result.Should().NotBeNull();
        result.Length.Should().BeGreaterThan(0);
    }

    [Fact]
    public async Task ExportMovimientosToExcelAsync_ShouldReturnNonEmptyByteArray()
    {
        // Arrange
        var data = new List<MovimientoDto>
        {
            new() { Fecha = DateTime.Now, Concepto = "Test", Total = 100, TipoMovimientoNombre = "Ingreso" }
        };

        // Act
        var result = await _service.ExportMovimientosToExcelAsync(data);

        // Assert
        result.Should().NotBeNull();
        result.Length.Should().BeGreaterThan(0);
    }

    [Fact]
    public async Task ExportMovimientosToCsvAsync_ShouldReturnNonEmptyByteArray()
    {
        // Arrange
        var data = new List<MovimientoDto>
        {
            new() { Fecha = DateTime.Now, Concepto = "Test", Total = 100, TipoMovimientoNombre = "Ingreso" }
        };

        // Act
        var result = await _service.ExportMovimientosToCsvAsync(data);

        // Assert
        result.Should().NotBeNull();
        result.Length.Should().BeGreaterThan(0);
    }

    [Fact]
    public async Task ExportLiquidacionToPdfAsync_ShouldReturnNonEmptyByteArray()
    {
        // Arrange
        var liquidacion = new LiquidacionDto 
        { 
            EmpleadoNombre = "Juan", 
            FechaInicio = DateTime.Now.AddDays(-15), 
            FechaFin = DateTime.Now,
            TotalBruto = 1000,
            TotalAdelantos = 100,
            TotalNeto = 900
        };
        var adelantos = new List<MovimientoDto>();

        // Act
        var result = await _service.ExportLiquidacionToPdfAsync(liquidacion, adelantos);

        // Assert
        result.Should().NotBeNull();
        result.Length.Should().BeGreaterThan(0);
    }

    [Fact]
    public async Task ExportCertificadoToPdfAsync_ShouldReturnNonEmptyByteArray()
    {
        // Arrange
        var certificado = new OrdenTrabajoDto 
        { 
            Titulo = "Cert 1", 
            Fecha = DateTime.Now,
            Items = new System.Collections.ObjectModel.ObservableCollection<OrdenTrabajoItemDto>
            {
                new() { Descripcion = "Item 1", Cantidad = 1, PrecioUnitario = 100 }
            }
        };
        var trabajo = new TrabajoDto { Descripcion = "Obra X" };

        // Act
        var result = await _service.ExportCertificadoToPdfAsync(certificado, trabajo);

        // Assert
        result.Should().NotBeNull();
        result.Length.Should().BeGreaterThan(0);
    }
}

