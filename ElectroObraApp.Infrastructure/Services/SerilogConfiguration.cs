using System.IO;
using Microsoft.Extensions.Configuration;
using Serilog;
using Serilog.Events;

namespace ElectroObraApp.Infrastructure.Services;

public static class SerilogConfiguration
{
    public static void Configure(IConfiguration configuration, string logDirectory)
    {
        Directory.CreateDirectory(logDirectory);

        var configuredLevel = configuration["Logging:LogLevel:Default"] ?? "Information";
        if (!Enum.TryParse<LogEventLevel>(configuredLevel, true, out var parsedLevel))
        {
            parsedLevel = LogEventLevel.Information;
        }

        Log.Logger = new LoggerConfiguration()
            .MinimumLevel.Is(parsedLevel)
            .MinimumLevel.Override("Microsoft", LogEventLevel.Warning)
            .MinimumLevel.Override("Microsoft.EntityFrameworkCore", LogEventLevel.Information)
            .MinimumLevel.Override("System", LogEventLevel.Warning)
            .Enrich.FromLogContext()
            .Enrich.WithMachineName()
            .Enrich.WithThreadId()
            .WriteTo.Console(
                outputTemplate: "[{Timestamp:HH:mm:ss} {Level:u3}] [{MachineName}/{ThreadId}] {Message:lj}{NewLine}{Exception}")
            .WriteTo.File(
                path: Path.Combine(logDirectory, "log.log"),
                rollingInterval: RollingInterval.Day,
                outputTemplate: "{Timestamp:yyyy-MM-dd HH:mm:ss.fff zzz} [{Level:u3}] [{MachineName}/{ThreadId}] {Message:lj}{NewLine}{Exception}")
            // Future: enable DB audit sink when audit table is available.
            // .WriteTo.Sink(new SerilogDbSink())
            .CreateLogger();
    }
}
