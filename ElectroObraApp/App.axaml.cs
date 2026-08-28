using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Avalonia.Media;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Composition;
using ElectroObraApp.Infrastructure.Data;
using ElectroObraApp.Infrastructure.Services;
using ElectroObraApp.ViewModels;
using ElectroObraApp.Views;
using ElectroObraApp.Core.Helpers;
using Serilog;
using System.IO;

namespace ElectroObraApp;

public partial class App : Avalonia.Application
{
    public IServiceProvider? Services { get; private set; }
    public IConfiguration? Configuration { get; private set; }

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override async void OnFrameworkInitializationCompleted()
    {
        // 1. Configuración
        var appDataPath = PathHelper.GetAppDataPath();
        var settingsPath = PathHelper.GetSettingsPath();

        if (!File.Exists(settingsPath))
        {
            var baseSettings = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "appsettings.json");
            if (File.Exists(baseSettings))
            {
                File.Copy(baseSettings, settingsPath);
            }
        }

        var environment = Environment.GetEnvironmentVariable("DOTNET_ENVIRONMENT")
            ?? Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT")
            ?? "Production";

        var builder = new ConfigurationBuilder()
            .SetBasePath(appDataPath)
            .AddJsonFile("appsettings.json", optional: false, reloadOnChange: true);

        var environmentSettingsPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, $"appsettings.{environment}.json");
        if (File.Exists(environmentSettingsPath))
        {
            builder.AddJsonFile(environmentSettingsPath, optional: true, reloadOnChange: true);
        }

        Configuration = builder.Build();

        // 2. Logging
        var logDirectory = Path.Combine(appDataPath, "logs");
        SerilogConfiguration.Configure(Configuration, logDirectory);

        // 3. DI Container
        var serviceCollection = new ServiceCollection();
        ConfigureServices(serviceCollection);
        Services = serviceCollection.BuildServiceProvider();

        // 4. Inicialización de Base de Datos
        await InitializeDatabaseAsync();

        // Global Exception Handling
        AppDomain.CurrentDomain.UnhandledException += (sender, e) => 
        {
            Log.Fatal(e.ExceptionObject as Exception, "Error no controlado (AppDomain)");
        };

        TaskScheduler.UnobservedTaskException += (sender, e) =>
        {
            Log.Error(e.Exception, "Error en tarea asíncrona no observada");
            e.SetObserved();
        };

        // 5. Inicialización de UI
        var mainViewModel = Services.GetRequiredService<MainViewModel>();
        
        // Cargar Tema e idioma
        var settings = Services.GetRequiredService<IUserSettingsService>();
        SetTheme(settings.GetTheme());

        var localization = Services.GetRequiredService<ILocalizationService>();
        localization.SetLanguage(settings.GetLanguage());
        localization.LanguageChanged += (_, _) => Markup.LocalizationBindingSource.Instance.Refresh();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.MainWindow = new MainWindow
            {
                DataContext = mainViewModel
            };
        }
        else if (ApplicationLifetime is ISingleViewApplicationLifetime singleViewPlatform)
        {
            singleViewPlatform.MainView = new MainView
            {
                DataContext = mainViewModel
            };
        }

        base.OnFrameworkInitializationCompleted();
    }

    private void ConfigureServices(IServiceCollection services)
    {
        if (ServiceConfigurationHost.ConfigureServices is null)
        {
            throw new InvalidOperationException(
                "ServiceConfigurationHost.ConfigureServices no está registrado. " +
                "El head de la aplicación debe asignarlo antes de iniciar Avalonia.");
        }

        ServiceConfigurationHost.ConfigureServices(services, Configuration!);
    }

    private async Task InitializeDatabaseAsync()
    {
        using var scope = Services!.CreateScope();
        var context = scope.ServiceProvider.GetRequiredService<ApplicationDbContext>();
        var logger = scope.ServiceProvider.GetRequiredService<ILogger<App>>();

        try
        {
            logger.LogInformation("Verificando y aplicando migraciones de base de datos...");
            await context.Database.MigrateAsync();

            var seedService = scope.ServiceProvider.GetRequiredService<IDatabaseSeedService>();
            if (seedService.IsSeedEnabled())
            {
                if (!await context.Movimientos.AnyAsync())
                {
                    logger.LogInformation("Base de datos vacía detectada. Sembrando datos iniciales...");
                    await seedService.SeedAsync();
                }
            }
            logger.LogInformation("Base de datos inicializada correctamente.");
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Error al inicializar la base de datos.");
        }
    }

    public void SetTheme(string themeName)
    {
        if (Resources.TryGetResource("MainBackgroundBrush", null, out var mb) && mb is SolidColorBrush mbBrush &&
            Resources.TryGetResource("PaneBackgroundBrush", null, out var pb) && pb is SolidColorBrush pbBrush &&
            Resources.TryGetResource("AccentBrush", null, out var ab) && ab is SolidColorBrush abBrush &&
            Resources.TryGetResource("SidebarSeparatorBrush", null, out var sb) && sb is SolidColorBrush sbBrush &&
            Resources.TryGetResource("MainForegroundBrush", null, out var mf) && mf is SolidColorBrush mfBrush &&
            Resources.TryGetResource("SecondaryForegroundBrush", null, out var sf) && sf is SolidColorBrush sfBrush)
        {
            switch (themeName)
            {
                case "Media Noche":
                    mbBrush.Color = Color.Parse("#0f0f12");
                    pbBrush.Color = Color.Parse("#1a1a2e");
                    abBrush.Color = Color.Parse("#6c5ce7");
                    sbBrush.Color = Color.Parse("#2e2e4e");
                    break;
                case "Industrial":
                    mbBrush.Color = Color.Parse("#2c3e50");
                    pbBrush.Color = Color.Parse("#34495e");
                    abBrush.Color = Color.Parse("#e67e22");
                    sbBrush.Color = Color.Parse("#465e75");
                    break;
                case "Solar":
                    mbBrush.Color = Color.Parse("#2d2d2d");
                    pbBrush.Color = Color.Parse("#3d3d3d");
                    abBrush.Color = Color.Parse("#f1c40f");
                    sbBrush.Color = Color.Parse("#4d4d4d");
                    break;
                case "Cibernético":
                    mbBrush.Color = Color.Parse("#050505");
                    pbBrush.Color = Color.Parse("#101010");
                    abBrush.Color = Color.Parse("#ff00ff");
                    sbBrush.Color = Color.Parse("#202020");
                    break;
                case "Océano":
                    mbBrush.Color = Color.Parse("#0b132b");
                    pbBrush.Color = Color.Parse("#1c2541");
                    abBrush.Color = Color.Parse("#5bc0be");
                    sbBrush.Color = Color.Parse("#3a506b");
                    break;
                case "Claro":
                    mbBrush.Color = Color.Parse("#f5f5f5");
                    pbBrush.Color = Color.Parse("#ffffff");
                    abBrush.Color = Color.Parse("#094771");
                    sbBrush.Color = Color.Parse("#dddddd");
                    break;
                default: // Oscuro
                    mbBrush.Color = Color.Parse("#1e1e1e");
                    pbBrush.Color = Color.Parse("#252526");
                    abBrush.Color = Color.Parse("#094771");
                    sbBrush.Color = Color.Parse("#3e3e42");
                    break;
            }
        }
    }
}
