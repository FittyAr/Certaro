using System.Threading.Tasks;

namespace ElectroObraApp.Application.Interfaces;

public interface IUserSettingsService
{
    int GetPageSize();
    Task SetPageSizeAsync(int pageSize);
    
    string GetTheme();
    Task SetThemeAsync(string theme);
    
    string GetLogoPath();
    Task SetLogoPathAsync(string path);
    
    string GetBackgroundPath();
    Task SetBackgroundPathAsync(string path);
    
    string GetAppName();
    Task SetAppNameAsync(string name);

    string GetDashboardPeriod();
    Task SetDashboardPeriodAsync(string period);

    bool GetIsPrivacyMode();
    Task SetIsPrivacyModeAsync(bool isPrivate);

    string GetPreferredEmailClient();
    Task SetPreferredEmailClientAsync(string client);
    string GetGmailUrl();
    string GetOutlookUrl();
    string GetYahooUrl();

    decimal GetDefaultMultiplierSaturday();
    Task SetDefaultMultiplierSaturdayAsync(decimal multiplier);

    decimal GetDefaultMultiplierSunday();
    Task SetDefaultMultiplierSundayAsync(decimal multiplier);

    decimal GetDefaultMultiplierHoliday();
    Task SetDefaultMultiplierHolidayAsync(decimal multiplier);

    string GetHolidaysJson();
    Task SetHolidaysJsonAsync(string holidaysJson);

    bool GetDefaultIncludeSaturday();
    Task SetDefaultIncludeSaturdayAsync(bool value);

    bool GetDefaultIncludeSunday();
    Task SetDefaultIncludeSundayAsync(bool value);

    bool GetDefaultIncludeHoliday();
    Task SetDefaultIncludeHolidayAsync(bool value);

    string GetHolidayApiUrl();
    Task SetHolidayApiUrlAsync(string url);

    bool GetAutoUpdateDollar();
    Task SetAutoUpdateDollarAsync(bool value);

    string GetDollarApiUrl();
    Task SetDollarApiUrlAsync(string url);

    string GetLanguage();
    Task SetLanguageAsync(string languageCode);
}

