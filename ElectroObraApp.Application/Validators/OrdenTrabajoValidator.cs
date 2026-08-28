using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class OrdenTrabajoValidator : AbstractValidator<OrdenTrabajoDto>
{
    public OrdenTrabajoValidator()
    {
        RuleFor(x => x.Titulo)
            .NotEmpty().WithMessage(ValidationMessages.OrdenTrabajoTituloRequired)
            .MaximumLength(200).WithMessage(ValidationMessages.OrdenTrabajoTituloMaxLength);

        RuleForEach(x => x.Items).SetValidator(new OrdenTrabajoItemValidator());
    }
}
