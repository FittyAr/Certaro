using System.Threading.Tasks;
using FluentAssertions;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;
using ElectroObraApp.Application.Validators;
using Xunit;

namespace ElectroObraApp.Tests.Application.Validation;

public class ValidationPipelineTests
{
    [Fact]
    public async Task ValidateAsync_ShouldReturnSuccess_WhenValid()
    {
        var validator = new CategoriaValidator();
        var dto = new CategoriaDto { Nombre = "Test" };

        var result = await ValidationPipeline.ValidateAsync(validator, dto);

        result.IsSuccess.Should().BeTrue();
    }

    [Fact]
    public async Task ValidateAsync_ShouldReturnFailureWithKeys_WhenInvalid()
    {
        var validator = new CategoriaValidator();
        var dto = new CategoriaDto { Nombre = "" };

        var result = await ValidationPipeline.ValidateAsync(validator, dto);

        result.IsSuccess.Should().BeFalse();
        result.Errors.Should().Contain(ValidationMessages.CategoriaNombreRequired);
    }
}
