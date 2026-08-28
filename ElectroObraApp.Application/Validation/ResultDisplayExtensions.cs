using System.Linq;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Validation;

public static class ResultDisplayExtensions
{
    public static string ToDisplayMessage(this Result result, ILocalizationService localization) =>
        string.Join("\n", result.Errors.Select(e => localization.GetString(e)));
}
