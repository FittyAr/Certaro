using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.ViewModels;
using Xunit;

namespace ElectroObraApp.Tests.UI.ViewModels;

public class SettingsViewModelTests
{
    private readonly IUserSettingsService _settingsService;
    private readonly IHolidayService _holidayService;
    private readonly ILocalizationService _localizationService;
    private readonly IMigrationRunner _migrationRunner;
    private readonly IBackupService _backupService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly INotificationService _notificationService;
    private readonly IFileSaveDialogService _fileSaveDialogService;

    public SettingsViewModelTests()
    {
        _settingsService = Substitute.For<IUserSettingsService>();
        _holidayService = Substitute.For<IHolidayService>();
        _localizationService = Substitute.For<ILocalizationService>();
        _migrationRunner = Substitute.For<IMigrationRunner>();
        _backupService = Substitute.For<IBackupService>();
        _confirmDialogService = Substitute.For<IConfirmDialogService>();
        _notificationService = Substitute.For<INotificationService>();
        _fileSaveDialogService = Substitute.For<IFileSaveDialogService>();

        _settingsService.GetAppName().Returns("ElectroObraApp");
        _settingsService.GetLogoPath().Returns("logo.png");
        _settingsService.GetBackgroundPath().Returns("background.png");
        _settingsService.GetTheme().Returns("Oscuro");
        _settingsService.GetLanguage().Returns("es");
        _settingsService.GetDashboardPeriod().Returns("Mensual");
        _settingsService.GetPreferredEmailClient().Returns("SystemDefault");
        _settingsService.GetAutoUpdateDollar().Returns(false);
        _settingsService.GetHolidayApiUrl().Returns("https://example.com/feriados/");
        _settingsService.GetDollarApiUrl().Returns("https://example.com/dolares");
        _settingsService.GetDefaultMultiplierSaturday().Returns(1.5m);
        _settingsService.GetDefaultMultiplierSunday().Returns(2.0m);
        _settingsService.GetDefaultMultiplierHoliday().Returns(2.0m);
        _settingsService.GetHolidaysJson().Returns("[]");

        _migrationRunner.GetAppliedMigrationsAsync(Arg.Any<CancellationToken>())
            .Returns(new List<string> { "20260101000000_Initial" });
        _migrationRunner.GetPendingMigrationsAsync(Arg.Any<CancellationToken>())
            .Returns(new List<string>());
        _migrationRunner.GetBackupFilesAsync(Arg.Any<CancellationToken>())
            .Returns(new List<string>());
        _backupService.ListBackupsAsync(Arg.Any<CancellationToken>())
            .Returns(new List<BackupInfo>());

        _localizationService.GetString(Arg.Any<string>()).Returns(call => call.Arg<string>());
    }

    private SettingsViewModel CreateViewModel() =>
        new(
            _settingsService,
            _holidayService,
            _localizationService,
            _migrationRunner,
            _backupService,
            _confirmDialogService,
            _notificationService,
            _fileSaveDialogService);

    [Fact]
    public void Constructor_ShouldLoadSettingsFromService()
    {
        var viewModel = CreateViewModel();

        viewModel.AppName.Should().Be("ElectroObraApp");
        viewModel.SelectedTheme.Should().Be("Oscuro");
        viewModel.SelectedLanguage.Should().Be("es");
        viewModel.MultiplierSaturday.Should().Be(1.5m);
        viewModel.Themes.Should().Equal("Claro", "Oscuro", "Sistema");
    }

    [Theory]
    [InlineData("es", 0)]
    [InlineData("en", 1)]
    public void SelectedLanguageIndex_ShouldMapLanguageCodes(string languageCode, int expectedIndex)
    {
        _settingsService.GetLanguage().Returns(languageCode);
        var viewModel = CreateViewModel();

        viewModel.SelectedLanguageIndex.Should().Be(expectedIndex);
    }

    [Theory]
    [InlineData("Media Noche", "Oscuro")]
    [InlineData("Claro", "Claro")]
    [InlineData("Sistema", "Sistema")]
    public void Constructor_ShouldMapLegacyThemes(string storedTheme, string expectedTheme)
    {
        _settingsService.GetTheme().Returns(storedTheme);
        var viewModel = CreateViewModel();

        viewModel.SelectedTheme.Should().Be(expectedTheme);
    }

    [Fact]
    public void AddHoliday_ShouldAppendHoliday_WhenDateIsUnique()
    {
        var viewModel = CreateViewModel();
        viewModel.NewHolidayDate = new DateTime(2026, 12, 25);
        viewModel.NewHolidayName = "Navidad";

        viewModel.AddHolidayCommand.Execute(null);

        viewModel.Holidays.Should().ContainSingle(h =>
            h.Date == new DateTime(2026, 12, 25) &&
            h.Name == "Navidad");
    }

    [Fact]
    public void AddHoliday_ShouldNotDuplicateExistingDate()
    {
        var viewModel = CreateViewModel();
        viewModel.NewHolidayDate = new DateTime(2026, 1, 1);
        viewModel.AddHolidayCommand.Execute(null);
        viewModel.NewHolidayName = "Duplicado";

        viewModel.AddHolidayCommand.Execute(null);

        viewModel.Holidays.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveHoliday_ShouldRemoveSelectedHoliday()
    {
        var viewModel = CreateViewModel();
        var holiday = new HolidayModel { Date = new DateTime(2026, 3, 24), Name = "Día Nacional de la Memoria" };
        viewModel.Holidays.Add(holiday);

        viewModel.RemoveHolidayCommand.Execute(holiday);

        viewModel.Holidays.Should().BeEmpty();
    }

    [Fact]
    public async Task SyncHolidaysAsync_ShouldMergeRemoteHolidays()
    {
        var viewModel = CreateViewModel();
        var currentYear = DateTime.Now.Year;
        _holidayService.GetHolidaysAsync(currentYear).Returns(new List<HolidayModel>
        {
            new() { Date = new DateTime(currentYear, 1, 1), Name = "Año Nuevo" }
        });
        _holidayService.GetHolidaysAsync(currentYear + 1).Returns(new List<HolidayModel>
        {
            new() { Date = new DateTime(currentYear + 1, 1, 1), Name = "Año Nuevo" }
        });

        await viewModel.SyncHolidaysAsync();

        viewModel.Holidays.Should().HaveCount(2);
        await _holidayService.Received(1).GetHolidaysAsync(currentYear);
        await _holidayService.Received(1).GetHolidaysAsync(currentYear + 1);
    }

    [Fact]
    public async Task ApplyChangesAsync_ShouldPersistSettings()
    {
        var viewModel = CreateViewModel();
        viewModel.AppName = "ElectroObra QA";
        viewModel.MultiplierSaturday = 1.75m;
        viewModel.MultiplierSunday = 2.25m;
        viewModel.MultiplierHoliday = 2.5m;

        await viewModel.ApplyChangesCommand.ExecuteAsync(null);

        await _settingsService.Received(1).SetAppNameAsync("ElectroObra QA");
        await _settingsService.Received(1).SetDefaultMultiplierSaturdayAsync(1.75m);
        await _settingsService.Received(1).SetDefaultMultiplierSundayAsync(2.25m);
        await _settingsService.Received(1).SetDefaultMultiplierHolidayAsync(2.5m);
    }

    [Fact]
    public async Task RefreshMigrationStatusAsync_ShouldLoadMigrationAndBackupData()
    {
        var viewModel = CreateViewModel();

        await viewModel.RefreshMigrationStatusCommand.ExecuteAsync(null);

        viewModel.AppliedMigrations.Should().Contain("20260101000000_Initial");
        viewModel.PendingMigrations.Should().BeEmpty();
        await _migrationRunner.Received().GetAppliedMigrationsAsync(Arg.Any<CancellationToken>());
        await _backupService.Received().ListBackupsAsync(Arg.Any<CancellationToken>());
    }
}
