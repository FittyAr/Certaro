using System;
using FluentValidation.TestHelper;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validators;
using Xunit;

namespace ElectroObraApp.Tests.Application.Validators;

public class TrabajoValidatorTests
{
    private readonly TrabajoValidator _validator;

    public TrabajoValidatorTests()
    {
        _validator = new TrabajoValidator();
    }

    [Fact]
    public void Should_HaveError_When_DescripcionIsEmpty()
    {
        var model = new TrabajoDto { Descripcion = "" };
        var result = _validator.TestValidate(model);
        result.ShouldHaveValidationErrorFor(x => x.Descripcion);
    }

    [Fact]
    public void Should_HaveError_When_ObraIdIsEmpty()
    {
        var model = new TrabajoDto { Descripcion = "Trabajo 1", ObraId = Guid.Empty };
        var result = _validator.TestValidate(model);
        result.ShouldHaveValidationErrorFor(x => x.ObraId);
    }

    [Fact]
    public void Should_NotHaveError_When_ModelIsValid()
    {
        var model = new TrabajoDto { Descripcion = "Trabajo 1", ObraId = Guid.NewGuid() };
        var result = _validator.TestValidate(model);
        result.ShouldNotHaveAnyValidationErrors();
    }
}
