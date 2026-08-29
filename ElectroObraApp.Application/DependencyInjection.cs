using FluentValidation;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.Services;
using System.Reflection;

namespace ElectroObraApp.Application;

public static class DependencyInjection
{
    public static IServiceCollection AddApplication(this IServiceCollection services)
    {
        Mappings.MappingConfig.Configure();
        services.AddScoped<IMovimientoService, MovimientoService>();
        services.AddScoped<ICategoriaService, CategoriaService>();
        services.AddScoped<ITipoMovimientoService, TipoMovimientoService>();
        services.AddScoped<IClienteService, ClienteService>();
        services.AddScoped<IEmpleadoService, EmpleadoService>();
        services.AddScoped<ITrabajoService, TrabajoService>();
        services.AddScoped<IObraService, ObraService>();
        services.AddScoped<IAsistenciaService, AsistenciaService>();
        services.AddScoped<IPagoFacturaService, PagoFacturaService>();
        services.AddScoped<ILiquidacionService, LiquidacionService>();
        services.AddScoped<IFacturaService, FacturaService>();
        
        services.AddValidatorsFromAssembly(Assembly.GetExecutingAssembly());
        
        return services;
    }
}

