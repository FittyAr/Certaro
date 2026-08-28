using System;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

namespace ElectroObraApp.Composition;

/// <summary>
/// Punto de extensión para que cada head (Desktop, Android, Browser, iOS) registre su composition root.
/// </summary>
public static class ServiceConfigurationHost
{
    public static Action<IServiceCollection, IConfiguration>? ConfigureServices { get; set; }
}
