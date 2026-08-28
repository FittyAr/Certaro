using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class ClienteValidator : AbstractValidator<ClienteDto>
{
    public ClienteValidator()
    {
        RuleFor(x => x.Nombre)
            .NotEmpty().WithMessage(ValidationMessages.ClienteNombreRequired)
            .MaximumLength(100).WithMessage(ValidationMessages.ClienteNombreMaxLength);

        RuleFor(x => x.Email)
            .EmailAddress().When(x => !string.IsNullOrEmpty(x.Email))
            .WithMessage(ValidationMessages.ClienteEmailInvalid);

        RuleFor(x => x.Cuit)
            .Matches(@"^\d{2}-\d{8}-\d{1}$").When(x => !string.IsNullOrEmpty(x.Cuit))
            .WithMessage(ValidationMessages.ClienteCuitInvalid);

        RuleForEach(x => x.Contactos).SetValidator(new ClienteContactoValidator());
    }
}
