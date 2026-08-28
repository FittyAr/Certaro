using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.Interfaces;
using System;
using System.Linq;
using System.Collections.Generic;

namespace ElectroObraApp.ViewModels;

public partial class SettingsViewModel : ViewModelBase
{
    private readonly IUserSettingsService _settingsService;
    private readonly IHolidayService _holidayService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private int _selectedCategoryIndex = 0;

    [ObservableProperty]
    private bool _isSaved;

    [ObservableProperty]
    private string _appName;

    [ObservableProperty]
    private string _logoPath;

    [ObservableProperty]
    private string _backgroundPath;

    [ObservableProperty]
    private string _selectedLanguage;

    public int SelectedLanguageIndex
    {
        get => SelectedLanguage == "en" ? 1 : 0;
        set
        {
            var code = value == 1 ? "en" : "es";
            if (SelectedLanguage != code)
            {
                SelectedLanguage = code;
            }
        }
    }

    partial void OnSelectedLanguageChanged(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return;
        }

        _localizationService.SetLanguage(value);
        Markup.LocalizationBindingSource.Instance.Refresh();
        OnPropertyChanged(nameof(SelectedLanguageIndex));
        _ = _settingsService.SetLanguageAsync(value);
    }

    [ObservableProperty]
    private string _selectedTheme;

    partial void OnSelectedThemeChanged(string value)
    {
        if (Avalonia.Application.Current is App app && !string.IsNullOrEmpty(value))
        {
            app.SetTheme(value);
            // Save theme automatically
            _ = _settingsService.SetThemeAsync(value);
        }
    }

    [ObservableProperty]
    private string _selectedDashboardPeriod;

    [ObservableProperty]
    private string _selectedEmailClient;

    [ObservableProperty]
    private bool _autoUpdateDollar;

    [ObservableProperty]
    private string _holidayApiUrl;

    [ObservableProperty]
    private string _dollarApiUrl;

    [ObservableProperty]
    private decimal _multiplierSaturday;

    [ObservableProperty]
    private decimal _multiplierSunday;

    [ObservableProperty]
    private decimal _multiplierHoliday;

    [ObservableProperty]
    private DateTime? _newHolidayDate = DateTime.Now;

    [ObservableProperty]
    private string _newHolidayName = string.Empty;

    public ObservableCollection<HolidayModel> Holidays { get; } = new();

    public ObservableCollection<string> Themes { get; } = new() { "Oscuro", "Media Noche", "Industrial", "Solar", "Cibernético", "Océano", "Claro" };
    public ObservableCollection<string> DashboardPeriods { get; } = new() { "Mensual", "Anual", "Total" };
    public ObservableCollection<string> EmailClients { get; } = new() { "SystemDefault", "Gmail", "Yahoo", "OutlookWeb" };

    public SettingsViewModel(
        IUserSettingsService settingsService,
        IHolidayService holidayService,
        ILocalizationService localizationService)
    {
        _settingsService = settingsService;
        _holidayService = holidayService;
        _localizationService = localizationService;
        
        _appName = _settingsService.GetAppName();
        _logoPath = _settingsService.GetLogoPath();
        _backgroundPath = _settingsService.GetBackgroundPath();
        _selectedTheme = _settingsService.GetTheme();
        _selectedLanguage = _settingsService.GetLanguage();
        _selectedDashboardPeriod = _settingsService.GetDashboardPeriod();
        _selectedEmailClient = _settingsService.GetPreferredEmailClient();
        _autoUpdateDollar = _settingsService.GetAutoUpdateDollar();
        _holidayApiUrl = _settingsService.GetHolidayApiUrl();
        _dollarApiUrl = _settingsService.GetDollarApiUrl();
        _multiplierSaturday = _settingsService.GetDefaultMultiplierSaturday();
        _multiplierSunday = _settingsService.GetDefaultMultiplierSunday();
        _multiplierHoliday = _settingsService.GetDefaultMultiplierHoliday();

        var holidaysJson = _settingsService.GetHolidaysJson();
        try
        {
            var items = System.Text.Json.JsonSerializer.Deserialize<List<HolidayModel>>(holidaysJson);
            if (items != null)
            {
                foreach (var item in items.OrderBy(x => x.Date)) Holidays.Add(item);
            }
        }
        catch 
        { 
            // Fallback for old format (List<DateTime>)
            try
            {
                var dates = System.Text.Json.JsonSerializer.Deserialize<List<DateTime>>(holidaysJson);
                if (dates != null)
                {
                    foreach (var d in dates.OrderBy(x => x)) Holidays.Add(new HolidayModel { Date = d, Name = "Feriado" });
                }
            }
            catch { }
        }
    }

    [RelayCommand]
    public void AddHoliday()
    {
        if (NewHolidayDate.HasValue && !Holidays.Any(h => h.Date == NewHolidayDate.Value.Date))
        {
            Holidays.Add(new HolidayModel 
            { 
                Date = NewHolidayDate.Value.Date, 
                Name = string.IsNullOrWhiteSpace(NewHolidayName) ? "Manual" : NewHolidayName 
            });
            SortHolidays();
            NewHolidayName = string.Empty;
        }
    }

    [RelayCommand]
    public void RemoveHoliday(HolidayModel holiday)
    {
        Holidays.Remove(holiday);
    }

    [RelayCommand]
    public async Task SyncHolidaysAsync()
    {
        var currentYear = DateTime.Now.Year;
        var nextYear = currentYear + 1;

        var currentHolidays = await _holidayService.GetHolidaysAsync(currentYear);
        var nextHolidays = await _holidayService.GetHolidaysAsync(nextYear);

        bool added = false;
        foreach (var h in currentHolidays)
        {
            if (!Holidays.Any(x => x.Date == h.Date))
            {
                Holidays.Add(h);
                added = true;
            }
        }

        foreach (var h in nextHolidays)
        {
            if (!Holidays.Any(x => x.Date == h.Date))
            {
                Holidays.Add(h);
                added = true;
            }
        }

        if (added)
        {
            SortHolidays();
        }
    }

    private void SortHolidays()
    {
        var sorted = Holidays.OrderBy(x => x.Date).ToList();
        Holidays.Clear();
        foreach (var h in sorted) Holidays.Add(h);
    }

    [RelayCommand]
    public async Task ApplyChangesAsync()
    {
        try
        {
            IsSaved = false;

            // Serialize holidays
            var holidayList = Holidays.ToList();
            var json = System.Text.Json.JsonSerializer.Serialize(holidayList);
            
            // Save holidays first to ensure they are captured
            await _settingsService.SetHolidaysJsonAsync(json);
            
            await _settingsService.SetAppNameAsync(AppName);
            await _settingsService.SetLogoPathAsync(LogoPath);
            await _settingsService.SetBackgroundPathAsync(BackgroundPath);
            await _settingsService.SetThemeAsync(SelectedTheme);
            await _settingsService.SetDashboardPeriodAsync(SelectedDashboardPeriod);
            await _settingsService.SetPreferredEmailClientAsync(SelectedEmailClient);
            await _settingsService.SetAutoUpdateDollarAsync(AutoUpdateDollar);
            await _settingsService.SetHolidayApiUrlAsync(HolidayApiUrl);
            await _settingsService.SetDollarApiUrlAsync(DollarApiUrl);
            await _settingsService.SetDefaultMultiplierSaturdayAsync(MultiplierSaturday);
            await _settingsService.SetDefaultMultiplierSundayAsync(MultiplierSunday);
            await _settingsService.SetDefaultMultiplierHolidayAsync(MultiplierHoliday);
            
            if (Avalonia.Application.Current is App app)
            {
                app.SetTheme(SelectedTheme);
            }

            IsSaved = true;
            await Task.Delay(3000);
            IsSaved = false;
        }
        catch (Exception ex)
        {
            Serilog.Log.Error(ex, "Error al guardar la configuración");
        }
    }
}
