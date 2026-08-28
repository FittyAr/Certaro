using CommunityToolkit.Mvvm.ComponentModel;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.Validation;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.ViewModels;

public abstract class ViewModelBase : ObservableObject
{
    private string? _errorMessage;
    private bool _isLoading;
    private bool _isEmpty;

    public string? ErrorMessage
    {
        get => _errorMessage;
        set
        {
            if (SetProperty(ref _errorMessage, value))
                OnPropertyChanged(nameof(HasError));
        }
    }

    public bool IsLoading
    {
        get => _isLoading;
        set => SetProperty(ref _isLoading, value);
    }

    public bool IsEmpty
    {
        get => _isEmpty;
        set => SetProperty(ref _isEmpty, value);
    }

    public bool HasError => !string.IsNullOrEmpty(ErrorMessage);

    protected bool HandleResult(Result result, ILocalizationService localization)
    {
        if (result.IsSuccess)
        {
            ErrorMessage = null;
            return true;
        }

        ErrorMessage = result.ToDisplayMessage(localization);
        return false;
    }
}
