using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class LiquidacionValidator : AbstractValidator<LiquidacionDto>
{
    public LiquidacionValidator()
    {
        RuleFor(x => x.EmpleadoId)
            .NotEmpty().WithMessage(ValidationMessages.LiquidacionEmpleadoRequired);

        RuleFor(x => x.FechaInicio)
            .LessThanOrEqualTo(x => x.FechaFin).WithMessage(ValidationMessages.LiquidacionFechaInicioInvalid);

        RuleFor(x => x.DiasTrabajados)
            .GreaterThan(0).WithMessage(ValidationMessages.LiquidacionDiasTrabajadosRequired);

        RuleFor(x => x.TarifaAplicada)
            .GreaterThan(0).WithMessage(ValidationMessages.LiquidacionTarifaRequired);
    }
}
