using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Text.Json;
using Avalonia.Platform;
using ElectroObraApp.Application.Interfaces;
using Microsoft.Extensions.Configuration;

namespace ElectroObraApp.Infrastructure.Services;

public class LocalizationService : ILocalizationService
{
    private const string ResourceBaseUri = "avares://ElectroObraApp/Assets/i18n";
    private const string UiAssemblyName = "ElectroObraApp";

    private readonly IConfiguration _configuration;
    private Dictionary<string, JsonElement> _translations = new();
    private string _currentLanguage;

    public LocalizationService(IConfiguration configuration, IUserSettingsService? settingsService = null)
    {
        _configuration = configuration;
        _currentLanguage = settingsService?.GetLanguage()
            ?? configuration["Application:DefaultLanguage"]
            ?? "es";
        LoadTranslations();
    }

    public string CurrentLanguage => _currentLanguage;

    public event EventHandler? LanguageChanged;

    public string GetString(string key)
    {
        var parts = key.Split('.');
        JsonElement current = default;
        var found = false;

        if (_translations.TryGetValue(parts[0], out var firstPart))
        {
            current = firstPart;
            found = true;
            for (var i = 1; i < parts.Length; i++)
            {
                if (current.TryGetProperty(parts[i], out var next))
                {
                    current = next;
                }
                else
                {
                    found = false;
                    break;
                }
            }
        }

        if (!found)
        {
            return key;
        }

        return current.ValueKind switch
        {
            JsonValueKind.String => current.GetString() ?? key,
            _ => current.ToString()
        };
    }

    public void SetLanguage(string languageCode)
    {
        if (string.IsNullOrWhiteSpace(languageCode) || _currentLanguage == languageCode)
        {
            return;
        }

        _currentLanguage = languageCode;
        LoadTranslations();
        LanguageChanged?.Invoke(this, EventArgs.Empty);
    }

    private void LoadTranslations()
    {
        try
        {
            var json = TryLoadFromAvares()
                ?? TryLoadFromAssembly()
                ?? TryLoadFromFileSystem();

            if (string.IsNullOrWhiteSpace(json))
            {
                _translations = new Dictionary<string, JsonElement>();
                return;
            }

            _translations = JsonSerializer.Deserialize<Dictionary<string, JsonElement>>(json) ?? new();
        }
        catch
        {
            _translations = new Dictionary<string, JsonElement>();
        }
    }

    private string? TryLoadFromAvares()
    {
        try
        {
            var uri = new Uri($"{ResourceBaseUri}/{_currentLanguage}.json");
            using var stream = AssetLoader.Open(uri);
            using var reader = new StreamReader(stream);
            return reader.ReadToEnd();
        }
        catch
        {
            return null;
        }
    }

    private string? TryLoadFromAssembly()
    {
        try
        {
            var assembly = FindUiAssembly();
            if (assembly is null)
            {
                return null;
            }

            var resourcePath = $"Assets/i18n/{_currentLanguage}.json".Replace('/', '.');
            var resourceName = assembly
                .GetManifestResourceNames()
                .FirstOrDefault(name => name.EndsWith(resourcePath, StringComparison.OrdinalIgnoreCase));

            if (resourceName is null)
            {
                return null;
            }

            using var stream = assembly.GetManifestResourceStream(resourceName);
            if (stream is null)
            {
                return null;
            }

            using var reader = new StreamReader(stream);
            return reader.ReadToEnd();
        }
        catch
        {
            return null;
        }
    }

    private string? TryLoadFromFileSystem()
    {
        try
        {
            var path = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "i18n", $"{_currentLanguage}.json");
            if (!File.Exists(path))
            {
                return null;
            }

            return File.ReadAllText(path);
        }
        catch
        {
            return null;
        }
    }

    private static Assembly? FindUiAssembly()
    {
        foreach (var assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            if (string.Equals(assembly.GetName().Name, UiAssemblyName, StringComparison.OrdinalIgnoreCase))
            {
                return assembly;
            }
        }

        try
        {
            return Assembly.Load(UiAssemblyName);
        }
        catch
        {
            return null;
        }
    }
}
