using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Avalonia.Styling;
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
            var migrationRunner = scope.ServiceProvider.GetRequiredService<IMigrationRunner>();
            var migrationResult = await migrationRunner.RunPendingMigrationsAsync();

            if (!migrationResult.Success)
            {
                logger.LogCritical("Migración fallida: {Error}", migrationResult.ErrorMessage);
                throw new InvalidOperationException(migrationResult.ErrorMessage ?? "Error desconocido en migración.");
            }

            if (migrationResult.MigrationsApplied)
            {
                logger.LogInformation("Migraciones aplicadas: {Migrations}. Backup: {Backup}",
                    string.Join(", ", migrationResult.AppliedMigrations),
                    migrationResult.BackupPath ?? "N/A");
            }

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
        RequestedThemeVariant = string.IsNullOrWhiteSpace(themeName)
            ? ThemeVariant.Dark
            : ResolveThemeVariant(themeName);
    }

    private static ThemeVariant ResolveThemeVariant(string themeName)
    {
        return themeName.Trim().ToLowerInvariant() switch
        {
            "light" or "claro" => ThemeVariant.Light,
            "dark" or "oscuro" => ThemeVariant.Dark,
            "system" or "default" or "sistema" => ThemeVariant.Default,
            // Legacy decorative theme names map to Dark
            "media noche" or "industrial" or "solar" or "cibernético" or "cibernetico" or "océano" or "oceano" => ThemeVariant.Dark,
            _ => ThemeVariant.Dark
        };
    }
}
