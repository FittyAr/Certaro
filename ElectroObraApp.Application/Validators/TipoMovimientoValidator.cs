using FluentValidation;
using ElectroObraApp.Application.DTOs;

namespace ElectroObraApp.Application.Validators;

public class TipoMovimientoValidator : AbstractValidator<TipoMovimientoDto>
{
    public TipoMovimientoValidator()
    {
        RuleFor(x => x.Nombre)
            .NotEmpty()
            .MaximumLength(100);
    }
}
