using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;

using CommunityToolkit.Mvvm.Input;

using CommunityToolkit.Mvvm.Messaging;

using ElectroObraApp.Application.Interfaces;

using Microsoft.Extensions.Configuration;

using Avalonia.Media;

using Avalonia.Media.Imaging;

using Avalonia.Platform;



namespace ElectroObraApp.ViewModels;



public partial class MainViewModel : ViewModelBase

{

    private static readonly string[] NavigationShortcuts =

    [

        "dashboard",

        "movimientos",

        "clientes",

        "obras",

        "certificados",

        "facturas",

        "empleados",

        "asistencia",

        "liquidaciones"

    ];



    private readonly ILocalizationService _localizationService;

    private readonly INavigationService _navigationService;



    [ObservableProperty]

    private string _greeting;



    [ObservableProperty]

    private ViewModelBase? _currentPage;



    [ObservableProperty]

    private string? _currentRoute;



    [ObservableProperty]

    private string _currentSection = string.Empty;



    [ObservableProperty]

    private string _breadcrumbText = string.Empty;



    [ObservableProperty]

    private bool _isSeedEnabled;



    [ObservableProperty]

    private bool _isSidebarOpen = true;



    [ObservableProperty]

    private IImage? _logoImage;



    [ObservableProperty]

    private IImage? _backgroundImage;



    [ObservableProperty]

    private bool _isCommandPaletteOpen;



    public CommandPaletteViewModel CommandPalette { get; }



    public MainViewModel(

        ILocalizationService localizationService,

        INavigationService navigationService,

        IDatabaseSeedService seedService,

        IConfiguration configuration,

        CommandPaletteViewModel commandPaletteViewModel)

    {

        _localizationService = localizationService;

        _navigationService = navigationService;

        _greeting = configuration["Application:Name"] ?? "ElectroObraApp";

        _isSeedEnabled = seedService.IsSeedEnabled();

        CommandPalette = commandPaletteViewModel;
        CommandPalette.CloseRequested += () => IsCommandPaletteOpen = false;



        var logoPath = configuration.GetValue<string>("Application:Branding:LogoPath", "avares://ElectroObraApp/Assets/Images/electro-obra-logo.svg");

        var backgroundPath = configuration.GetValue<string>("Application:Branding:BackgroundPath", "avares://ElectroObraApp/Assets/Images/electro-obra.svg");



        LogoImage = LoadImageFromPath(logoPath);

        BackgroundImage = LoadImageFromPath(backgroundPath);



        NavigateToCommand = new RelayCommand<string>(NavigateToRoute);

        GoBackCommand = new RelayCommand(GoBack, () => CanGoBack);

        ToggleSidebarCommand = new RelayCommand(ToggleSidebar);



        _navigationService.NavigationChanged += OnNavigationChanged;

        _navigationService.NavigateTo("dashboard");



        WeakReferenceMessenger.Default.Register<string>(this, (_, message) =>

        {

            var route = MapLegacySectionToRoute(message);

            if (route is not null)

            {

                NavigateToRoute(route);

            }

        });

    }



    public IRelayCommand<string> NavigateToCommand { get; }

    public IRelayCommand GoBackCommand { get; }

    public IRelayCommand ToggleSidebarCommand { get; }



    public bool CanGoBack => _navigationService.CanGoBack;



    [RelayCommand]

    private void OpenCommandPalette()

    {

        CommandPalette.Reset();

        IsCommandPaletteOpen = true;

    }



    [RelayCommand]

    private void CloseCommandPalette()

    {

        IsCommandPaletteOpen = false;

    }



    [RelayCommand]

    private void NavigateShortcut(string? indexText)

    {

        if (!int.TryParse(indexText, out var index) || index < 1 || index > NavigationShortcuts.Length)

        {

            return;

        }



        NavigateToRoute(NavigationShortcuts[index - 1]);

    }



    [RelayCommand]

    private async Task HandleContextNewAsync()

    {

        if (IsCommandPaletteOpen)

        {

            return;

        }



        switch (CurrentPage)

        {

            case MovimientosViewModel vm when vm.AddCommand.CanExecute(null):

                await vm.AddCommand.ExecuteAsync(null);

                break;

            case ClientesViewModel vm when vm.AddCommand.CanExecute(null):

                vm.AddCommand.Execute(null);

                break;

            case EmpleadosViewModel vm when vm.AddCommand.CanExecute(null):

                vm.AddCommand.Execute(null);

                break;

            case FacturasViewModel vm when vm.AddCommand.CanExecute(null):

                vm.AddCommand.Execute(null);

                break;

            case TrabajosViewModel vm when vm.AddCommand.CanExecute(null):

                await vm.AddCommand.ExecuteAsync(null);

                break;

            case LiquidacionesViewModel vm when vm.StartWizardCommand.CanExecute(null):

                await vm.StartWizardCommand.ExecuteAsync(null);

                break;

        }

    }



    [RelayCommand]

    private async Task HandleContextSaveAsync()

    {

        if (IsCommandPaletteOpen)

        {

            return;

        }



        if (await TrySaveActiveEditAsync())

        {

            return;

        }



        if (CurrentPage is SettingsViewModel settings && settings.ApplyChangesCommand.CanExecute(null))

        {

            await settings.ApplyChangesCommand.ExecuteAsync(null);

        }

    }



    [RelayCommand]

    private async Task HandleRefreshAsync()

    {

        if (IsCommandPaletteOpen)

        {

            return;

        }



        switch (CurrentPage)

        {

            case DashboardViewModel vm when vm.LoadStatsCommand.CanExecute(null):

                await vm.LoadStatsCommand.ExecuteAsync(null);

                break;

            case MovimientosViewModel vm when vm.LoadMovimientosCommand.CanExecute(null):

                await vm.LoadMovimientosCommand.ExecuteAsync(null);

                break;

            case ClientesViewModel vm when vm.LoadClientesCommand.CanExecute(null):

                await vm.LoadClientesCommand.ExecuteAsync(null);

                break;

            case EmpleadosViewModel vm when vm.LoadEmpleadosCommand.CanExecute(null):

                await vm.LoadEmpleadosCommand.ExecuteAsync(null);

                break;

            case FacturasViewModel vm when vm.LoadFacturasCommand.CanExecute(null):

                await vm.LoadFacturasCommand.ExecuteAsync(null);

                break;

            case TrabajosViewModel vm when vm.LoadTrabajosCommand.CanExecute(null):

                await vm.LoadTrabajosCommand.ExecuteAsync(null);

                break;

            case LiquidacionesViewModel vm when vm.LoadCommand.CanExecute(null):

                await vm.LoadCommand.ExecuteAsync(null);

                break;

            case CertificadosViewModel vm when vm.LoadCommand.CanExecute(null):

                await vm.LoadCommand.ExecuteAsync(null);

                break;

            case SettingsViewModel vm when vm.RefreshMigrationStatusCommand.CanExecute(null):

                await vm.RefreshMigrationStatusCommand.ExecuteAsync(null);

                break;

        }

    }



    [RelayCommand]

    private void HandleEscape()

    {

        if (IsCommandPaletteOpen)

        {

            IsCommandPaletteOpen = false;

            return;

        }



        if (TryCancelActiveEdit())

        {

            return;

        }



        if (IsSidebarOpen)

        {

            IsSidebarOpen = false;

            return;

        }



        if (CanGoBack)

        {

            GoBack();

        }

    }



    private void NavigateToRoute(string? route)

    {

        if (string.IsNullOrWhiteSpace(route))

        {

            return;

        }



        _navigationService.NavigateTo(route);

    }



    private void GoBack()

    {

        _navigationService.GoBack();

    }



    private void ToggleSidebar()

    {

        IsSidebarOpen = !IsSidebarOpen;

    }



    private void OnNavigationChanged(object? sender, NavigationChangedEventArgs e)

    {

        CurrentPage = e.ViewModel as ViewModelBase;

        CurrentRoute = e.Route;

        CurrentSection = GetSectionTitle(e.Route);

        BreadcrumbText = string.Format(

            _localizationService.GetString("Navigation.Breadcrumb"),

            _localizationService.GetString("General.AppName"),

            CurrentSection);

        OnPropertyChanged(nameof(CanGoBack));

        GoBackCommand.NotifyCanExecuteChanged();

    }



    private async Task<bool> TrySaveActiveEditAsync()

    {

        switch (CurrentPage)

        {

            case MovimientosViewModel { IsEditing: true, EditViewModel: MovimientoEditViewModel edit } when edit.SaveCommand.CanExecute(null):

                await edit.SaveCommand.ExecuteAsync(null);

                return true;

            case ClientesViewModel { IsEditing: true, EditViewModel: ClienteEditViewModel edit } when edit.SaveCommand.CanExecute(null):

                await edit.SaveCommand.ExecuteAsync(null);

                return true;

            case EmpleadosViewModel { IsEditing: true, EditViewModel: EmpleadoEditViewModel edit } when edit.SaveCommand.CanExecute(null):

                await edit.SaveCommand.ExecuteAsync(null);

                return true;

            case FacturasViewModel { IsEditing: true, EditViewModel: FacturaEditViewModel edit } when edit.SaveCommand.CanExecute(null):

                await edit.SaveCommand.ExecuteAsync(null);

                return true;

            case TrabajosViewModel { IsEditing: true, EditViewModel: TrabajoEditViewModel edit } when edit.SaveCommand.CanExecute(null):

                await edit.SaveCommand.ExecuteAsync(null);

                return true;

            case LiquidacionEditViewModel edit when edit.SaveCommand.CanExecute(null):

                await edit.SaveCommand.ExecuteAsync(null);

                return true;

            default:

                return false;

        }

    }



    private bool TryCancelActiveEdit()

    {

        switch (CurrentPage)

        {

            case MovimientosViewModel { IsEditing: true, EditViewModel: MovimientoEditViewModel edit } when edit.CancelCommand.CanExecute(null):

                edit.CancelCommand.Execute(null);

                return true;

            case ClientesViewModel { IsEditing: true, EditViewModel: ClienteEditViewModel edit } when edit.CancelCommand.CanExecute(null):

                edit.CancelCommand.Execute(null);

                return true;

            case EmpleadosViewModel { IsEditing: true, EditViewModel: EmpleadoEditViewModel edit } when edit.CancelCommand.CanExecute(null):

                edit.CancelCommand.Execute(null);

                return true;

            case FacturasViewModel { IsEditing: true, EditViewModel: FacturaEditViewModel edit } when edit.CancelCommand.CanExecute(null):

                edit.CancelCommand.Execute(null);

                return true;

            case TrabajosViewModel { IsEditing: true, EditViewModel: TrabajoEditViewModel edit } when edit.CancelCommand.CanExecute(null):

                edit.CancelCommand.Execute(null);

                return true;

            case LiquidacionEditViewModel edit when edit.CancelCommand.CanExecute(null):

                edit.CancelCommand.Execute(null);

                return true;

            default:

                return false;

        }

    }



    private string GetSectionTitle(string route) =>

        _localizationService.GetString($"Menu.{GetMenuKey(route)}");



    private static string GetMenuKey(string route) => route switch

    {

        "dashboard" => "Dashboard",

        "movimientos" => "Movimientos",

        "clientes" => "Clientes",

        "obras" => "Obras",

        "certificados" => "Certificados",

        "facturas" => "Facturas",

        "empleados" => "Empleados",

        "asistencia" => "Asistencia",

        "liquidaciones" => "Liquidaciones",

        "configuracion" => "Settings",

        "categorias" => "Categories",

        "tipos-movimiento" => "MovementTypes",

        "reportes" => "Reports",

        "seed" => "Seed",

        "liquidacion-edit" => "Liquidaciones",

        _ => "Dashboard"

    };



    private static string? MapLegacySectionToRoute(string section) => section switch

    {

        "Dashboard" => "dashboard",

        "Movimientos" => "movimientos",

        "Clientes" => "clientes",

        "Empleados" => "empleados",

        "Trabajos" => "certificados",

        "Liquidaciones" => "liquidaciones",

        "Facturas" => "facturas",

        "Configuración" => "configuracion",

        "Seed" => "seed",

        _ => null

    };



    private IImage? LoadImageFromPath(string? path)

    {

        if (string.IsNullOrEmpty(path)) return null;

        try

        {

            var uri = new Uri(path);

            using var stream = AssetLoader.Open(uri);

            return new Bitmap(stream);

        }

        catch (Exception ex)

        {

            Serilog.Log.Error(ex, "Error al cargar imagen de marca desde {Path}", path);

            return null;

        }

    }

}


