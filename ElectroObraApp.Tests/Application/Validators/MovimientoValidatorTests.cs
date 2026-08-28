using System;
using FluentValidation.TestHelper;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validators;
using Xunit;

namespace ElectroObraApp.Tests.Application.Validators;

public class MovimientoValidatorTests
{
    private readonly MovimientoValidator _validator;

    public MovimientoValidatorTests()
    {
        _validator = new MovimientoValidator();
    }

    [Fact]
    public void Should_HaveError_When_ConceptoIsEmpty()
    {
        var model = new MovimientoDto { Concepto = "" };
        var result = _validator.TestValidate(model);
        result.ShouldHaveValidationErrorFor(x => x.Concepto);
    }

    [Fact]
    public void Should_HaveError_When_MontoIsZero()
    {
        var model = new MovimientoDto { Monto = 0 };
        var result = _validator.TestValidate(model);
        result.ShouldHaveValidationErrorFor(x => x.Monto);
    }

    [Fact]
    public void Should_NotHaveError_When_CategoriaIdIsNull()
    {
        var model = new MovimientoDto 
        { 
            Concepto = "Venta", 
            Monto = 100, 
            Cantidad = 1,
            CategoriaId = null,
            TipoMovimientoId = Guid.NewGuid()
        };
        var result = _validator.TestValidate(model);
        result.ShouldNotHaveValidationErrorFor(x => x.CategoriaId);
    }

    [Fact]
    public void Should_NotHaveError_When_ModelIsValid()
    {
        var model = new MovimientoDto 
        { 
            Concepto = "Venta", 
            Monto = 100, 
            Cantidad = 1,
            CategoriaId = Guid.NewGuid(),
            TipoMovimientoId = Guid.NewGuid()
        };
        var result = _validator.TestValidate(model);
        result.ShouldNotHaveAnyValidationErrors();
    }
}

