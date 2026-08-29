using System.Collections.ObjectModel;
using System.IO;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.Interfaces;
using System;
using System.Linq;
using System.Collections.Generic;

namespace ElectroObraApp.ViewModels;

public sealed class BackupListItem
{
    public required BackupInfo Info { get; init; }
    public string FileName => Path.GetFileName(Info.FilePath);
    public string SizeDisplay => FormatBytes(Info.SizeBytes);
    public string DateDisplay => Info.CreatedAt.ToLocalTime().ToString("g");

    private static string FormatBytes(long bytes) =>
        bytes switch
        {
            < 1024 => $"{bytes} B",
            < 1024 * 1024 => $"{bytes / 1024.0:0.#} KB",
            _ => $"{bytes / (1024.0 * 1024.0):0.#} MB"
        };
}

public partial class SettingsViewModel : ViewModelBase
{
    private readonly IUserSettingsService _settingsService;
    private readonly IHolidayService _holidayService;
    private readonly ILocalizationService _localizationService;
    private readonly IMigrationRunner _migrationRunner;
    private readonly IBackupService _backupService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly INotificationService _notificationService;
    private readonly IFileSaveDialogService _fileSaveDialogService;

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
    private string _selectedTheme;

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

    [ObservableProperty]
    private bool _isMigrationLoading;

    [ObservableProperty]
    [NotifyCanExecuteChangedFor(nameof(RestoreBackupCommand))]
    private BackupListItem? _selectedBackup;

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

    partial void OnSelectedThemeChanged(string value)
    {
        if (Avalonia.Application.Current is App app && !string.IsNullOrEmpty(value))
        {
            app.SetTheme(value);
            _ = _settingsService.SetThemeAsync(value);
        }
    }

    public ObservableCollection<HolidayModel> Holidays { get; } = new();
    public ObservableCollection<string> AppliedMigrations { get; } = new();
    public ObservableCollection<string> PendingMigrations { get; } = new();
    public ObservableCollection<BackupListItem> Backups { get; } = new();

    public ObservableCollection<string> Themes { get; } = new() { "Claro", "Oscuro", "Sistema" };
    public ObservableCollection<string> DashboardPeriods { get; } = new() { "Mensual", "Anual", "Total" };
    public ObservableCollection<string> EmailClients { get; } = new() { "SystemDefault", "Gmail", "Yahoo", "OutlookWeb" };

    public SettingsViewModel(
        IUserSettingsService settingsService,
        IHolidayService holidayService,
        ILocalizationService localizationService,
        IMigrationRunner migrationRunner,
        IBackupService backupService,
        IConfirmDialogService confirmDialogService,
        INotificationService notificationService,
        IFileSaveDialogService fileSaveDialogService)
    {
        _settingsService = settingsService;
        _holidayService = holidayService;
        _localizationService = localizationService;
        _migrationRunner = migrationRunner;
        _backupService = backupService;
        _confirmDialogService = confirmDialogService;
        _notificationService = notificationService;
        _fileSaveDialogService = fileSaveDialogService;

        _appName = _settingsService.GetAppName();
        _logoPath = _settingsService.GetLogoPath();
        _backgroundPath = _settingsService.GetBackgroundPath();
        _selectedTheme = MapLegacyTheme(_settingsService.GetTheme());
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

        _ = RefreshMigrationStatusAsync();
    }

    [RelayCommand]
    public async Task RefreshMigrationStatusAsync()
    {
        try
        {
            IsMigrationLoading = true;
            ErrorMessage = null;

            var applied = await _migrationRunner.GetAppliedMigrationsAsync();
            var pending = await _migrationRunner.GetPendingMigrationsAsync();
            var backupFiles = await _migrationRunner.GetBackupFilesAsync();
            var backups = await _backupService.ListBackupsAsync();

            AppliedMigrations.Clear();
            foreach (var migration in applied)
            {
                AppliedMigrations.Add(migration);
            }

            PendingMigrations.Clear();
            foreach (var migration in pending)
            {
                PendingMigrations.Add(migration);
            }

            Backups.Clear();
            var backupItems = backups.Count > 0
                ? backups
                : backupFiles.Select(path => new BackupInfo
                {
                    FilePath = path,
                    CreatedAt = File.GetCreationTimeUtc(path),
                    SizeBytes = new FileInfo(path).Length
                }).ToList();

            foreach (var backup in backupItems.OrderByDescending(b => b.CreatedAt))
            {
                Backups.Add(new BackupListItem { Info = backup });
            }

            SelectedBackup = Backups.FirstOrDefault();
        }
        catch (Exception ex)
        {
            Serilog.Log.Error(ex, "Error al cargar estado de migraciones");
            ErrorMessage = ex.Message;
        }
        finally
        {
            IsMigrationLoading = false;
        }
    }

    private bool CanRestoreBackup => SelectedBackup is not null;

    [RelayCommand(CanExecute = nameof(CanRestoreBackup))]
    public async Task RestoreBackupAsync()
    {
        if (SelectedBackup is null)
        {
            return;
        }

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("Settings.Migration.RestoreTitle"),
            _localizationService.GetString("Settings.Migration.RestoreConfirm"));

        if (!confirmed)
        {
            return;
        }

        try
        {
            IsMigrationLoading = true;
            await _backupService.RestoreFromBackupAsync(SelectedBackup.Info.FilePath);
            await _notificationService.ShowInfoAsync(
                _localizationService.GetString("General.Success"),
                _localizationService.GetString("Settings.Migration.RestoreSuccess"));
            await RefreshMigrationStatusAsync();
        }
        catch (Exception ex)
        {
            Serilog.Log.Error(ex, "Error al restaurar backup");
            await _notificationService.ShowErrorAsync(
                _localizationService.GetString("General.Error"),
                ex.Message);
        }
        finally
        {
            IsMigrationLoading = false;
        }
    }

    [RelayCommand]
    public async Task ExportDatabaseJsonAsync()
    {
        try
        {
            IsMigrationLoading = true;
            var tempPath = Path.Combine(Path.GetTempPath(), $"electroobra_export_{DateTime.UtcNow:yyyyMMdd_HHmmss}.json");
            var result = await _backupService.ExportToJsonAsync(tempPath);

            if (!result.Success || !File.Exists(tempPath))
            {
                await _notificationService.ShowErrorAsync(
                    _localizationService.GetString("General.Error"),
                    result.ErrorMessage ?? _localizationService.GetString("Settings.Migration.ExportFailed"));
                return;
            }

            var bytes = await File.ReadAllBytesAsync(tempPath);
            File.Delete(tempPath);

            var saved = await _fileSaveDialogService.SaveFileAsync(bytes, "electroobra_export", "json");
            if (saved)
            {
                await _notificationService.ShowInfoAsync(
                    _localizationService.GetString("General.Success"),
                    _localizationService.GetString("Settings.Migration.ExportSuccess"));
            }
        }
        catch (Exception ex)
        {
            Serilog.Log.Error(ex, "Error al exportar base de datos");
            await _notificationService.ShowErrorAsync(
                _localizationService.GetString("General.Error"),
                ex.Message);
        }
        finally
        {
            IsMigrationLoading = false;
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

        var added = false;
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

            var holidayList = Holidays.ToList();
            var json = System.Text.Json.JsonSerializer.Serialize(holidayList);

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

    private static string MapLegacyTheme(string theme) =>
        theme.Trim().ToLowerInvariant() switch
        {
            "claro" or "light" => "Claro",
            "oscuro" or "dark" => "Oscuro",
            "sistema" or "system" or "default" => "Sistema",
            _ => "Oscuro"
        };
}
