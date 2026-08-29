using System;
using System.Collections.Generic;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Serilog;

namespace ElectroObraApp.Services;

public sealed class NavigationService : INavigationService
{
    private readonly IServiceProvider _serviceProvider;
    private readonly Dictionary<string, Type> _routes = new(StringComparer.OrdinalIgnoreCase);
    private readonly Stack<string> _history = new();

    public NavigationService(IServiceProvider serviceProvider)
    {
        _serviceProvider = serviceProvider;
        RegisterDefaultRoutes();
    }

    public string? CurrentRoute { get; private set; }

    public bool CanGoBack => _history.Count > 0;

    public event EventHandler<NavigationChangedEventArgs>? NavigationChanged;

    public void RegisterRoute(string route, Type viewModelType)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(route);
        ArgumentNullException.ThrowIfNull(viewModelType);

        _routes[route] = viewModelType;
    }

    public void NavigateTo(string route)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(route);

        if (!_routes.TryGetValue(route, out var viewModelType))
        {
            Log.Warning("Ruta de navegación no registrada: {Route}", route);
            return;
        }

        if (CurrentRoute is not null &&
            !string.Equals(CurrentRoute, route, StringComparison.OrdinalIgnoreCase))
        {
            _history.Push(CurrentRoute);
        }

        var viewModel = _serviceProvider.GetRequiredService(viewModelType);
        InitializeViewModel(viewModel, route);

        CurrentRoute = route;
        NavigationChanged?.Invoke(this, new NavigationChangedEventArgs(route, viewModel));
    }

    public bool GoBack()
    {
        if (_history.Count == 0)
        {
            return false;
        }

        var previousRoute = _history.Pop();
        NavigateToWithoutHistory(previousRoute);
        return true;
    }

    private void RegisterDefaultRoutes()
    {
        RegisterRoute("dashboard", typeof(DashboardViewModel));
        RegisterRoute("movimientos", typeof(MovimientosViewModel));
        RegisterRoute("clientes", typeof(ClientesViewModel));
        RegisterRoute("obras", typeof(ObrasViewModel));
        RegisterRoute("certificados", typeof(CertificadosViewModel));
        RegisterRoute("facturas", typeof(FacturasViewModel));
        RegisterRoute("empleados", typeof(EmpleadosViewModel));
        RegisterRoute("asistencia", typeof(AsistenciaViewModel));
        RegisterRoute("liquidaciones", typeof(LiquidacionesViewModel));
        RegisterRoute("configuracion", typeof(SettingsViewModel));
        RegisterRoute("categorias", typeof(CategoriasViewModel));
        RegisterRoute("tipos-movimiento", typeof(TiposMovimientoViewModel));
        RegisterRoute("reportes", typeof(ReportsViewModel));
        RegisterRoute("seed", typeof(SeedViewModel));
        RegisterRoute("liquidacion-edit", typeof(LiquidacionEditViewModel));
    }

    private void NavigateToWithoutHistory(string route)
    {
        if (!_routes.TryGetValue(route, out var viewModelType))
        {
            Log.Warning("Ruta de navegación no registrada: {Route}", route);
            return;
        }

        var viewModel = _serviceProvider.GetRequiredService(viewModelType);
        InitializeViewModel(viewModel, route);

        CurrentRoute = route;
        NavigationChanged?.Invoke(this, new NavigationChangedEventArgs(route, viewModel));
    }

    private void InitializeViewModel(object viewModel, string route)
    {
        switch (route)
        {
            case "dashboard" when viewModel is DashboardViewModel dashboard:
                _ = dashboard.LoadStatsAsync();
                break;
            case "liquidaciones" when viewModel is LiquidacionesViewModel liquidaciones:
                _ = liquidaciones.LoadAsync();
                break;
            case "asistencia" when viewModel is AsistenciaViewModel asistencia:
                _ = asistencia.LoadAsync();
                break;
            case "liquidacion-edit" when viewModel is LiquidacionEditViewModel edit:
                edit.CloseRequest += (_, _) => GoBack();
                _ = edit.LoadDataAsync();
                break;
            case "certificados" when viewModel is CertificadosViewModel certificados:
                _ = certificados.LoadAsync();
                break;
        }
    }
}
