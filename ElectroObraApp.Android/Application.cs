using Android.App;
using Android.Runtime;
using Avalonia;
using Avalonia.Android;
using ElectroObraApp.Composition;
using ElectroObraApp.Desktop;

namespace ElectroObraApp.Android
{
    [Application]
    public class Application : AvaloniaAndroidApplication<App>
    {
        static Application()
        {
            ServiceConfigurationHost.ConfigureServices = ServiceConfiguration.ConfigureServices;
        }
        protected Application(nint javaReference, JniHandleOwnership transfer) : base(javaReference, transfer)
        {
        }

        protected override AppBuilder CustomizeAppBuilder(AppBuilder builder)
        {
            return base.CustomizeAppBuilder(builder)
            .WithInterFont();
        }
    }
}

