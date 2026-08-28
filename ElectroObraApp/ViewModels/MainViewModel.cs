using System;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.Application.Interfaces;
using Microsoft.Extensions.Configuration;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;

namespace ElectroObraApp.ViewModels;

public partial class MainViewModel : ViewModelBase
{
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;

    [ObservableProperty]
    private string _greeting;

    [ObservableProperty]
    private ViewModelBase? _currentPage;

    [ObservableProperty]
    private string _currentSection = "Dashboard";

    [ObservableProperty]
    private bool _isSeedEnabled;

    [ObservableProperty]
    private IImage? _logoImage;

    [ObservableProperty]
    private IImage? _backgroundImage;

    public MainViewModel(ILocalizationService localizationService, IServiceProvider serviceProvider, IDatabaseSeedService seedService, IConfiguration configuration)
    {
        _localizationService = localizationService;
        _serviceProvider = serviceProvider;
        _greeting = configuration["Application:Name"] ?? "ElectroObraApp";
        _isSeedEnabled = seedService.IsSeedEnabled();
        
        var logoPath = configuration.GetValue<string>("Application:Branding:LogoPath", "avares://ElectroObraApp/Assets/Images/electro-obra-logo.svg");
        var backgroundPath = configuration.GetValue<string>("Application:Branding:BackgroundPath", "avares://ElectroObraApp/Assets/Images/electro-obra.svg");
        
        LogoImage = LoadImageFromPath(logoPath);
        BackgroundImage = LoadImageFromPath(backgroundPath);
        
        NavigateToDashboardCommand = new RelayCommand(NavigateToDashboard);
        NavigateToMovimientosCommand = new RelayCommand(NavigateToMovimientos);
        NavigateToClientesCommand = new RelayCommand(NavigateToClientes);
        NavigateToEmpleadosCommand = new RelayCommand(NavigateToEmpleados);
        NavigateToTrabajosCommand = new RelayCommand(NavigateToTrabajos);
        NavigateToLiquidacionesCommand = new RelayCommand(NavigateToLiquidaciones);
        NavigateToFacturasCommand = new RelayCommand(NavigateToFacturas);
        NavigateToSeedCommand = new RelayCommand(NavigateToSeed);
        NavigateToSettingsCommand = new RelayCommand(NavigateToSettings);
        NavigateToCommand = new RelayCommand<string>(NavigateTo);
        
        // Pagina inicial
        NavigateToDashboard();

        // Registro de Mensajería para navegación externa
        WeakReferenceMessenger.Default.Register<string>(this, (r, m) => {
            switch(m)
            {
                case "Dashboard": NavigateToDashboard(); break;
                case "Movimientos": NavigateToMovimientos(); break;
                case "Clientes": NavigateToClientes(); break;
                case "Empleados": NavigateToEmpleados(); break;
                case "Trabajos": NavigateToTrabajos(); break;
                case "Liquidaciones": NavigateToLiquidaciones(); break;
                case "Facturas": NavigateToFacturas(); break;
                case "Configuración": NavigateToSettings(); break;
            }
        });
    }

    public IRelayCommand NavigateToDashboardCommand { get; }
    public IRelayCommand NavigateToMovimientosCommand { get; }
    public IRelayCommand NavigateToClientesCommand { get; }
    public IRelayCommand NavigateToEmpleadosCommand { get; }
    public IRelayCommand NavigateToTrabajosCommand { get; }
    public IRelayCommand NavigateToLiquidacionesCommand { get; }
    public IRelayCommand NavigateToFacturasCommand { get; }
    public IRelayCommand NavigateToSeedCommand { get; }
    public IRelayCommand NavigateToSettingsCommand { get; }
    public IRelayCommand<string> NavigateToCommand { get; }

    private void NavigateToDashboard() 
    { 
        var vm = _serviceProvider.GetRequiredService<DashboardViewModel>(); 
        _ = vm.LoadStatsAsync();
        CurrentPage = vm; 
        CurrentSection = "Dashboard"; 
    }
    private void NavigateToMovimientos() { CurrentPage = _serviceProvider.GetRequiredService<MovimientosViewModel>(); CurrentSection = "Movimientos"; }
    private void NavigateToClientes() { CurrentPage = _serviceProvider.GetRequiredService<ClientesViewModel>(); CurrentSection = "Clientes"; }
    private void NavigateToEmpleados() { CurrentPage = _serviceProvider.GetRequiredService<EmpleadosViewModel>(); CurrentSection = "Empleados"; }
    private void NavigateToTrabajos() { CurrentPage = _serviceProvider.GetRequiredService<TrabajosViewModel>(); CurrentSection = "Trabajos"; }
    private void NavigateToLiquidaciones()
    { 
        var vm = _serviceProvider.GetRequiredService<LiquidacionesViewModel>();
        vm.OnNuevaLiquidacion = () => NavigateTo("LiquidacionEdit");
        _ = vm.LoadAsync();
        CurrentPage = vm;
        CurrentSection = "Liquidaciones";
    }
    private void NavigateToFacturas() { CurrentPage = _serviceProvider.GetRequiredService<FacturasViewModel>(); CurrentSection = "Facturas"; }
    private void NavigateToSeed() { CurrentPage = _serviceProvider.GetRequiredService<SeedViewModel>(); CurrentSection = "Seed"; }
    private void NavigateToSettings() { CurrentPage = _serviceProvider.GetRequiredService<SettingsViewModel>(); CurrentSection = "Configuración"; }

    private void NavigateTo(string? destination)
    {
        if (destination == "LiquidacionEdit")
        {
            var vm = _serviceProvider.GetRequiredService<LiquidacionEditViewModel>();
            vm.CloseRequest += (s, success) => NavigateToLiquidaciones();
            _ = vm.LoadDataAsync(); // Carga de datos asíncrona
            CurrentPage = vm;
            CurrentSection = "Liquidaciones";
        }
    }

    private IImage? LoadImageFromPath(string? path)
    {
        if (string.IsNullOrEmpty(path)) return null;
        try
        {
            var uri = new Uri(path);
            using (var stream = AssetLoader.Open(uri))
            {
                return new Bitmap(stream);
            }
        }
        catch (Exception ex)
        {
            Serilog.Log.Error(ex, "Error al cargar imagen de marca desde {Path}", path);
            return null;
        }
    }
}
