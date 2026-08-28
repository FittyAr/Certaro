using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class FacturaValidator : AbstractValidator<FacturaDto>
{
    public FacturaValidator()
    {
        RuleFor(x => x.Numero)
            .NotEmpty().WithMessage(ValidationMessages.FacturaNumeroRequired)
            .MaximumLength(50).WithMessage(ValidationMessages.FacturaNumeroMaxLength);

        RuleFor(x => x.ClienteId)
            .NotEmpty().WithMessage(ValidationMessages.FacturaClienteRequired);

        RuleFor(x => x.Subtotal)
            .GreaterThanOrEqualTo(0).WithMessage(ValidationMessages.FacturaSubtotalInvalid);

        RuleFor(x => x.Iva)
            .GreaterThanOrEqualTo(0).WithMessage(ValidationMessages.FacturaIvaInvalid);

        RuleFor(x => x.Total)
            .GreaterThanOrEqualTo(0).WithMessage(ValidationMessages.FacturaTotalInvalid);
    }
}
