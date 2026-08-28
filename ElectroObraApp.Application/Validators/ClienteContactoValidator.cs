using FluentValidation;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Validation;

namespace ElectroObraApp.Application.Validators;

public class ClienteContactoValidator : AbstractValidator<ClienteContactoDto>
{
    public ClienteContactoValidator()
    {
        RuleFor(x => x.Etiqueta)
            .NotEmpty().WithMessage(ValidationMessages.ClienteContactoEtiquetaRequired);

        RuleFor(x => x.Email)
            .EmailAddress().When(x => !string.IsNullOrEmpty(x.Email))
            .WithMessage(ValidationMessages.ClienteContactoEmailInvalid);
    }
}
