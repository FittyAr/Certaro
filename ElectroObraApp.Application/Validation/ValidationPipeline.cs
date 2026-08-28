using System.Linq;
using System.Threading.Tasks;
using ElectroObraApp.Core.Common;
using FluentValidation;

namespace ElectroObraApp.Application.Validation;

public static class ValidationPipeline
{
    public static async Task<Result> ValidateAsync<T>(IValidator<T> validator, T instance)
    {
        var validationResult = await validator.ValidateAsync(instance);
        if (validationResult.IsValid)
            return Result.Success();

        var errors = validationResult.Errors
            .Select(e => e.ErrorMessage)
            .ToList();

        return Result.Failure(errors);
    }
}
