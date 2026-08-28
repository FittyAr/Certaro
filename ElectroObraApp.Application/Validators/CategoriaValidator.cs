using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class CategoriaValidator : AbstractValidator<CategoriaDto>
{
    public CategoriaValidator()
    {
        RuleFor(x => x.Nombre)
            .NotEmpty().WithMessage(ValidationMessages.CategoriaNombreRequired)
            .MaximumLength(100).WithMessage(ValidationMessages.CategoriaNombreMaxLength);
    }
}
