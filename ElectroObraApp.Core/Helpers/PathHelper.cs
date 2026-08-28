using System;
using System.IO;
using System.Runtime.InteropServices;

namespace ElectroObraApp.Core.Helpers;

public static class PathHelper
{
    private const string AppFolderName = "ElectroObraApp";

    /// <summary>
    /// Resuelve la carpeta base de datos, configuración y logs según la plataforma.
    /// </summary>
    /// <remarks>
    /// <list type="bullet">
    /// <item><description><b>Browser (WASM)</b>: ruta virtual <c>/AppData</c> (almacenamiento en memoria/IndexedDB del host).</description></item>
    /// <item><description><b>Android / iOS</b>: <see cref="Environment.SpecialFolder.LocalApplicationData"/> (sandbox del usuario, recomendado por Apple/Google).</description></item>
    /// <item><description><b>Desktop Windows</b>: <see cref="Environment.SpecialFolder.LocalApplicationData"/> (<c>%LocalAppData%</c>). Se prefiere sobre <see cref="Environment.SpecialFolder.CommonApplicationData"/> porque la BD SQLite y la configuración son por usuario y no requieren permisos elevados.</description></item>
    /// <item><description><b>Desktop Linux / macOS</b>: <see cref="Environment.SpecialFolder.LocalApplicationData"/> (<c>~/.local/share</c> / <c>~/Library/Application Support</c>).</description></item>
    /// </list>
    /// </remarks>
    public static string GetAppDataPath()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Create("BROWSER")))
        {
            return "/AppData";
        }

        if (OperatingSystem.IsAndroid() || OperatingSystem.IsIOS())
        {
            return EnsureDirectory(GetSpecialFolderPath(Environment.SpecialFolder.LocalApplicationData));
        }

        // Desktop (Windows, Linux, macOS): datos por usuario en LocalApplicationData.
        return EnsureDirectory(GetSpecialFolderPath(Environment.SpecialFolder.LocalApplicationData));
    }

    public static string GetSettingsPath() => Path.Combine(GetAppDataPath(), "appsettings.json");

    public static string GetDatabasePath() => Path.Combine(GetAppDataPath(), "ElectroObraApp.db");

    public static string GetSqliteConnectionString() => $"Data Source={GetDatabasePath()}";

    private static string GetSpecialFolderPath(Environment.SpecialFolder folder)
    {
        var basePath = Environment.GetFolderPath(folder);
        if (string.IsNullOrEmpty(basePath))
        {
            basePath = AppDomain.CurrentDomain.BaseDirectory;
        }

        return Path.Combine(basePath, AppFolderName);
    }

    private static string EnsureDirectory(string path)
    {
        if (!Directory.Exists(path))
        {
            try
            {
                Directory.CreateDirectory(path);
            }
            catch
            {
                return AppDomain.CurrentDomain.BaseDirectory;
            }
        }

        return path;
    }
}
