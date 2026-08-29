using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class PagoFacturaValidator : AbstractValidator<PagoFacturaDto>
{
    public PagoFacturaValidator()
    {
        RuleFor(x => x.FacturaId)
            .NotEmpty().WithMessage(ValidationMessages.PagoFacturaFacturaRequired);

        RuleFor(x => x.Monto)
            .GreaterThan(0).WithMessage(ValidationMessages.PagoFacturaMontoRequired);

        RuleFor(x => x.MedioPago)
            .NotEmpty().WithMessage(ValidationMessages.PagoFacturaMedioPagoRequired)
            .MaximumLength(100).WithMessage(ValidationMessages.PagoFacturaMedioPagoMaxLength);
    }
}
