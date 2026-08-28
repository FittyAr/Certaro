using System.Runtime.Versioning;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Browser;
using ElectroObraApp;
using ElectroObraApp.Composition;
using ElectroObraApp.Desktop;

internal sealed partial class Program
{
    static Program()
    {
        ServiceConfigurationHost.ConfigureServices = ServiceConfiguration.ConfigureServices;
    }

    private static Task Main(string[] args) => BuildAvaloniaApp()
            .WithInterFont()
#if DEBUG
            .WithDeveloperTools()
#endif
            .StartBrowserAppAsync("out");

    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>();
}
