using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class MovimientoValidator : AbstractValidator<MovimientoDto>
{
    public MovimientoValidator()
    {
        RuleFor(x => x.Concepto)
            .NotEmpty().WithMessage(ValidationMessages.MovimientoConceptoRequired)
            .MaximumLength(200).WithMessage(ValidationMessages.MovimientoConceptoMaxLength);

        RuleFor(x => x.Monto)
            .GreaterThan(0).WithMessage(ValidationMessages.MovimientoMontoRequired);

        RuleFor(x => x.Cantidad)
            .GreaterThan(0).WithMessage(ValidationMessages.MovimientoCantidadRequired);

        RuleFor(x => x.TipoMovimientoId)
            .NotEmpty().WithMessage(ValidationMessages.MovimientoTipoRequired);
    }
}
