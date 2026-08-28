using FluentAssertions;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validators;
using ElectroObraApp.Application.Validation;
using Xunit;

namespace ElectroObraApp.Tests.Application.Validators;

public class FacturaValidatorTests
{
    private readonly FacturaValidator _validator = new();

    [Fact]
    public void Validate_WithEmptyNumero_ShouldFail()
    {
        var dto = new FacturaDto { ClienteId = Guid.NewGuid(), Subtotal = 100, Iva = 21, Total = 121 };

        var result = _validator.Validate(dto);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.ErrorMessage == ValidationMessages.FacturaNumeroRequired);
    }

    [Fact]
    public void Validate_WithValidDto_ShouldPass()
    {
        var dto = new FacturaDto
        {
            Numero = "F-100",
            ClienteId = Guid.NewGuid(),
            Subtotal = 100,
            Iva = 21,
            Total = 121
        };

        var result = _validator.Validate(dto);

        result.IsValid.Should().BeTrue();
    }
}
