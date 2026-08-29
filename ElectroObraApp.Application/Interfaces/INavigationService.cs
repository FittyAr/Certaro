namespace ElectroObraApp.Application.Interfaces;

public interface INavigationService
{
    string? CurrentRoute { get; }

    bool CanGoBack { get; }

    event EventHandler<NavigationChangedEventArgs>? NavigationChanged;

    void RegisterRoute(string route, Type viewModelType);

    void NavigateTo(string route);

    bool GoBack();
}

public sealed class NavigationChangedEventArgs : EventArgs
{
    public NavigationChangedEventArgs(string route, object? viewModel)
    {
        Route = route;
        ViewModel = viewModel;
    }

    public string Route { get; }

    public object? ViewModel { get; }
}
