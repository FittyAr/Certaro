using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class EmpleadoValidator : AbstractValidator<EmpleadoDto>
{
    public EmpleadoValidator()
    {
        RuleFor(x => x.Nombre)
            .NotEmpty().WithMessage(ValidationMessages.EmpleadoNombreRequired)
            .MaximumLength(100).WithMessage(ValidationMessages.EmpleadoNombreMaxLength);

        RuleFor(x => x.Dni)
            .NotEmpty().WithMessage(ValidationMessages.EmpleadoDniRequired)
            .Length(7, 9).WithMessage(ValidationMessages.EmpleadoDniLength);

        RuleFor(x => x.TarifaDiaria)
            .GreaterThanOrEqualTo(0).WithMessage(ValidationMessages.EmpleadoTarifaNegative);
    }
}
