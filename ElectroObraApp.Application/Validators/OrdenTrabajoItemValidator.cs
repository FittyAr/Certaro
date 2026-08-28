using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class OrdenTrabajoItemValidator : AbstractValidator<OrdenTrabajoItemDto>
{
    public OrdenTrabajoItemValidator()
    {
        RuleFor(x => x.Descripcion)
            .NotEmpty().WithMessage(ValidationMessages.OrdenTrabajoItemDescripcionRequired);

        RuleFor(x => x.Cantidad)
            .GreaterThan(0).WithMessage(ValidationMessages.OrdenTrabajoItemCantidadRequired);

        RuleFor(x => x.PrecioUnitario)
            .GreaterThanOrEqualTo(0).WithMessage(ValidationMessages.OrdenTrabajoItemPrecioNegative);

        RuleFor(x => x.PorcentajeActual)
            .InclusiveBetween(0, 100).WithMessage(ValidationMessages.OrdenTrabajoItemPorcentajeInvalid);

        RuleFor(x => x.PorcentajeAnterior)
            .InclusiveBetween(0, 100).WithMessage(ValidationMessages.OrdenTrabajoItemPorcentajeInvalid);
    }
}
