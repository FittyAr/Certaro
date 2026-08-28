using ElectroObraApp.Application;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Infrastructure;
using ElectroObraApp.Services;
using ElectroObraApp.ViewModels;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Serilog;

namespace ElectroObraApp.Desktop;

public static class ServiceConfiguration
{
    public static void ConfigureServices(IServiceCollection services, IConfiguration configuration)
    {
        services.AddSingleton(configuration);
        services.AddLogging(builder => builder.AddSerilog());
        services.AddApplication();
        services.AddInfrastructure(configuration);

        services.AddScoped<IConfirmDialogService, ConfirmDialogService>();
        services.AddScoped<IFileSaveDialogService, FileSaveDialogService>();
        services.AddSingleton<INotificationService, NotificationService>();

        services.AddTransient<MainViewModel>();
        services.AddSingleton<DashboardViewModel>();
        services.AddTransient<MovimientosViewModel>();
        services.AddTransient<MovimientoEditViewModel>();
        services.AddTransient<ClientesViewModel>();
        services.AddTransient<ClienteEditViewModel>();
        services.AddTransient<EmpleadosViewModel>();
        services.AddTransient<EmpleadoEditViewModel>();
        services.AddTransient<TrabajosViewModel>();
        services.AddTransient<TrabajoEditViewModel>();
        services.AddTransient<FacturasViewModel>();
        services.AddTransient<FacturaEditViewModel>();
        services.AddTransient<LiquidacionesViewModel>();
        services.AddTransient<LiquidacionEditViewModel>();
        services.AddTransient<SeedViewModel>();
        services.AddTransient<SettingsViewModel>();
    }
}
