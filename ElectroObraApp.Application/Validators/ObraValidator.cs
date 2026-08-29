using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class ObraValidator : AbstractValidator<ObraDto>
{
    public ObraValidator()
    {
        RuleFor(x => x.Nombre)
            .NotEmpty().WithMessage(ValidationMessages.ObraNombreRequired)
            .MaximumLength(200).WithMessage(ValidationMessages.ObraNombreMaxLength);

        RuleFor(x => x.ClienteId)
            .NotEmpty().WithMessage(ValidationMessages.ObraClienteRequired);

        RuleFor(x => x.Numero)
            .GreaterThan(0).WithMessage(ValidationMessages.ObraNumeroRequired);
    }
}
