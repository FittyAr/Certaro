using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class TrabajoValidator : AbstractValidator<TrabajoDto>
{
    public TrabajoValidator()
    {
        RuleFor(x => x.Descripcion)
            .NotEmpty().WithMessage(ValidationMessages.TrabajoDescripcionRequired)
            .MaximumLength(200).WithMessage(ValidationMessages.TrabajoDescripcionMaxLength);

        RuleFor(x => x.ObraId)
            .NotEmpty().WithMessage(ValidationMessages.TrabajoObraRequired);

        RuleForEach(x => x.OrdenesTrabajo).SetValidator(new OrdenTrabajoValidator());
    }
}
