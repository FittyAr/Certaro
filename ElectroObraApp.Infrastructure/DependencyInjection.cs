using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Infrastructure.Data;
using ElectroObraApp.Infrastructure.Migrations;
using ElectroObraApp.Infrastructure.Repositories;
using ElectroObraApp.Infrastructure.Services;

namespace ElectroObraApp.Infrastructure;

public static class DependencyInjection
{
    public static IServiceCollection AddInfrastructure(this IServiceCollection services, IConfiguration configuration)
    {
        var connectionString = ElectroObraApp.Core.Helpers.PathHelper.GetSqliteConnectionString();
        var httpTimeoutSeconds = configuration.GetValue("Application:HttpTimeoutSeconds", 30);

        services.AddDbContext<ApplicationDbContext>(options =>
            options.UseSqlite(connectionString));

        services.AddScoped<IUnitOfWork, UnitOfWork>();
        services.AddSingleton<ILocalizationService, LocalizationService>();
        services.AddScoped<IExportService, ExportService>();
        services.AddScoped<IDatabaseSeedService, DatabaseSeedService>();
        services.AddScoped<IUserSettingsService, UserSettingsService>();
        services.AddScoped<IDashboardService, DashboardService>();
        services.AddScoped<IComercialService, ComercialService>();
        services.AddScoped<IEmailService, SmtpEmailService>();
        services.AddScoped<DatabaseCensusService>();
        services.AddScoped<IBackupService, BackupService>();
        services.AddScoped<IAdjuntoService, AdjuntoService>();
        services.AddScoped<IMigrationRunner, SafeMigrationRunner>();

        services.AddHttpClient<IHolidayService, HolidayService>()
            .ConfigureHttpClient(client => client.Timeout = TimeSpan.FromSeconds(httpTimeoutSeconds));

        services.AddHttpClient<IDollarService, DollarService>()
            .ConfigureHttpClient(client => client.Timeout = TimeSpan.FromSeconds(httpTimeoutSeconds));

        return services;
    }
}
