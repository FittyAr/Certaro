using Microsoft.Extensions.Configuration;
using ElectroObraApp.Application.Interfaces;
using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading.Tasks;
using System.Globalization;
using Microsoft.Extensions.Logging;

namespace ElectroObraApp.Infrastructure.Services;

public class UserSettingsService : IUserSettingsService
{
    private readonly IConfiguration _configuration;
    private readonly ILogger<UserSettingsService> _logger;
    private readonly string _settingsPath;
    private readonly SemaphoreSlim _fileLock = new(1, 1);

    public UserSettingsService(IConfiguration configuration, ILogger<UserSettingsService> logger, string? settingsPath = null)
    {
        _configuration = configuration;
        _logger = logger;
        _settingsPath = settingsPath ?? ElectroObraApp.Core.Helpers.PathHelper.GetSettingsPath();
        
        // Ensure the directory exists
        var directory = Path.GetDirectoryName(_settingsPath);
        if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
        {
            Directory.CreateDirectory(directory);
        }

        // Ensure the file exists with basic structure if it doesn't
        if (!File.Exists(_settingsPath))
        {
            File.WriteAllText(_settingsPath, "{ \"Application\": { } }");
        }
    }

    public int GetPageSize() => int.TryParse(GetValue("Application:LastPageSize", "30"), out var res) ? res : 30;
    public Task SetPageSizeAsync(int pageSize) => SetValueAsync("Application:LastPageSize", pageSize);

    public string GetTheme() => GetValue("Application:Appearance:Theme", "Dark");
    public Task SetThemeAsync(string theme) => SetValueAsync("Application:Appearance:Theme", theme);

    public string GetLogoPath() => GetValue("Application:Branding:LogoPath", "avares://ElectroObraApp/Assets/Images/electro-obra.png");
    public Task SetLogoPathAsync(string path) => SetValueAsync("Application:Branding:LogoPath", path);

    public string GetBackgroundPath() => GetValue("Application:Branding:BackgroundPath", "avares://ElectroObraApp/Assets/Images/electro-obra3.png");
    public Task SetBackgroundPathAsync(string path) => SetValueAsync("Application:Branding:BackgroundPath", path);

    public string GetAppName() => GetValue("Application:Name", "ElectroObra");
    public Task SetAppNameAsync(string name) => SetValueAsync("Application:Name", name);

    public string GetDashboardPeriod() => GetValue("Application:Dashboard:Period", "Mensual");
    public Task SetDashboardPeriodAsync(string period) => SetValueAsync("Application:Dashboard:Period", period);

    public bool GetIsPrivacyMode() => GetValue("Application:Dashboard:IsPrivacyMode", "false").ToLower() == "true";
    public Task SetIsPrivacyModeAsync(bool isPrivate) => SetValueAsync("Application:Dashboard:IsPrivacyMode", isPrivate);

    public string GetPreferredEmailClient() => GetValue("Application:Email:PreferredClient", "SystemDefault");
    public Task SetPreferredEmailClientAsync(string client) => SetValueAsync("Application:Email:PreferredClient", client);

    public string GetGmailUrl() => GetValue("Application:Email:GmailUrl", "https://mail.google.com/mail/u/0/?view=cm&fs=1&to={email}");
    public string GetOutlookUrl() => GetValue("Application:Email:OutlookUrl", "https://outlook.live.com/mail/0/deeplink/compose?to={email}");
    public string GetYahooUrl() => GetValue("Application:Email:YahooUrl", "https://mail.yahoo.com/d/compose-message?to={email}");

    public decimal GetDefaultMultiplierSaturday() => decimal.TryParse(GetValue("Application:Settlement:MultiplierSaturday", "1.5"), CultureInfo.InvariantCulture, out var result) ? result : 1.5m;
    public Task SetDefaultMultiplierSaturdayAsync(decimal multiplier) => SetValueAsync("Application:Settlement:MultiplierSaturday", multiplier.ToString(CultureInfo.InvariantCulture));

    public decimal GetDefaultMultiplierSunday() => decimal.TryParse(GetValue("Application:Settlement:MultiplierSunday", "2.0"), CultureInfo.InvariantCulture, out var result) ? result : 2.0m;
    public Task SetDefaultMultiplierSundayAsync(decimal multiplier) => SetValueAsync("Application:Settlement:MultiplierSunday", multiplier.ToString(CultureInfo.InvariantCulture));

    public decimal GetDefaultMultiplierHoliday() => decimal.TryParse(GetValue("Application:Settlement:MultiplierHoliday", "2.0"), CultureInfo.InvariantCulture, out var result) ? result : 2.0m;
    public Task SetDefaultMultiplierHolidayAsync(decimal multiplier) => SetValueAsync("Application:Settlement:MultiplierHoliday", multiplier.ToString(CultureInfo.InvariantCulture));

    public string GetHolidaysJson() => GetValue("Application:Settlement:Holidays", "[]");
    public Task SetHolidaysJsonAsync(string holidaysJson) => SetValueAsync("Application:Settlement:Holidays", holidaysJson);

    public bool GetDefaultIncludeSaturday() => GetValue("Application:Settlement:IncludeSaturday", "false").ToLower() == "true";
    public Task SetDefaultIncludeSaturdayAsync(bool value) => SetValueAsync("Application:Settlement:IncludeSaturday", value);

    public bool GetDefaultIncludeSunday() => GetValue("Application:Settlement:IncludeSunday", "false").ToLower() == "true";
    public Task SetDefaultIncludeSundayAsync(bool value) => SetValueAsync("Application:Settlement:IncludeSunday", value);

    public bool GetDefaultIncludeHoliday() => GetValue("Application:Settlement:IncludeHoliday", "false").ToLower() == "true";
    public Task SetDefaultIncludeHolidayAsync(bool value) => SetValueAsync("Application:Settlement:IncludeHoliday", value);

    public string GetHolidayApiUrl() => GetValue("Application:Settlement:HolidayApiUrl", "https://api.argentinadatos.com/v1/feriados/");
    public Task SetHolidayApiUrlAsync(string url) => SetValueAsync("Application:Settlement:HolidayApiUrl", url);

    public bool GetAutoUpdateDollar() => GetValue("Application:Dollar:AutoUpdate", "true").ToLower() == "true";
    public Task SetAutoUpdateDollarAsync(bool value) => SetValueAsync("Application:Dollar:AutoUpdate", value);

    public string GetDollarApiUrl() => GetValue("Application:Settlement:DollarApiUrl", "https://dolarapi.com/v1/dolares");
    public Task SetDollarApiUrlAsync(string url) => SetValueAsync("Application:Settlement:DollarApiUrl", url);

    public string GetLanguage() => GetValue("Application:Language", "es");
    public Task SetLanguageAsync(string languageCode) => SetValueAsync("Application:Language", languageCode);

    private string GetValue(string key, string defaultValue)
    {
        try
        {
            if (!File.Exists(_settingsPath)) return defaultValue;
            var jsonString = File.ReadAllText(_settingsPath);
            if (string.IsNullOrWhiteSpace(jsonString)) return defaultValue;

            var root = JsonNode.Parse(jsonString);
            if (root == null) return defaultValue;

            var parts = key.Split(':');
            JsonNode? currentNode = root;

            foreach (var part in parts)
            {
                if (currentNode is JsonObject obj)
                {
                    currentNode = obj[part];
                }
                else
                {
                    return defaultValue;
                }
            }

            if (currentNode == null) return defaultValue;

            if (currentNode is JsonValue jValue)
            {
                if (jValue.TryGetValue<string>(out var s)) return s;
                return jValue.ToString();
            }

            return currentNode.ToJsonString();
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading setting {Key}", key);
            return defaultValue;
        }
    }

    private async Task SetValueAsync(string keyPath, object? value)
    {
        await _fileLock.WaitAsync();
        try
        {
            JsonObject root;
            if (File.Exists(_settingsPath))
            {
                var jsonString = await File.ReadAllTextAsync(_settingsPath);
                try
                {
                    root = JsonNode.Parse(jsonString) as JsonObject ?? new JsonObject();
                }
                catch
                {
                    root = new JsonObject();
                }
            }
            else
            {
                root = new JsonObject();
            }

            var parts = keyPath.Split(':');
            JsonObject current = root;

            for (int i = 0; i < parts.Length - 1; i++)
            {
                var part = parts[i];
                if (current[part] is not JsonObject next)
                {
                    next = new JsonObject();
                    current[part] = next;
                }
                current = next;
            }

            var lastPart = parts[^1];
            
            if (value is string s && (s.TrimStart().StartsWith("[") || s.TrimStart().StartsWith("{")))
            {
                try
                {
                    var node = JsonNode.Parse(s);
                    if (node != null)
                    {
                        current[lastPart] = node;
                        goto Save;
                    }
                }
                catch { /* Not valid JSON, fall through */ }
            }

            current[lastPart] = value == null ? null : JsonValue.Create(value);

        Save:
            var directory = Path.GetDirectoryName(_settingsPath);
            if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
            {
                Directory.CreateDirectory(directory);
            }

            var options = new JsonSerializerOptions { WriteIndented = true };
            using (var stream = File.Create(_settingsPath))
            {
                await JsonSerializer.SerializeAsync(stream, root, options);
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error saving setting {KeyPath}", keyPath);
            throw;
        }
        finally
        {
            _fileLock.Release();
        }
    }
}
