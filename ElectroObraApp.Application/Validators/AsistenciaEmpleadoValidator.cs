using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class AsistenciaEmpleadoValidator : AbstractValidator<AsistenciaEmpleadoDto>
{
    public AsistenciaEmpleadoValidator()
    {
        RuleFor(x => x.EmpleadoId)
            .NotEmpty().WithMessage(ValidationMessages.AsistenciaEmpleadoRequired);

        RuleFor(x => x.Fecha)
            .NotEmpty().WithMessage(ValidationMessages.AsistenciaFechaRequired);
    }
}
