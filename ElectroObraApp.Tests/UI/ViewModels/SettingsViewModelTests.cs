using System;
using System.Collections.Generic;
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

    public SettingsViewModelTests()
    {
        _settingsService = Substitute.For<IUserSettingsService>();
        _holidayService = Substitute.For<IHolidayService>();
        _localizationService = Substitute.For<ILocalizationService>();

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
    }

    private SettingsViewModel CreateViewModel() =>
        new(_settingsService, _holidayService, _localizationService);

    [Fact]
    public void Constructor_ShouldLoadSettingsFromService()
    {
        // Act
        var viewModel = CreateViewModel();

        // Assert
        viewModel.AppName.Should().Be("ElectroObraApp");
        viewModel.SelectedTheme.Should().Be("Oscuro");
        viewModel.SelectedLanguage.Should().Be("es");
        viewModel.MultiplierSaturday.Should().Be(1.5m);
    }

    [Theory]
    [InlineData("es", 0)]
    [InlineData("en", 1)]
    public void SelectedLanguageIndex_ShouldMapLanguageCodes(string languageCode, int expectedIndex)
    {
        // Arrange
        _settingsService.GetLanguage().Returns(languageCode);
        var viewModel = CreateViewModel();

        // Act & Assert
        viewModel.SelectedLanguageIndex.Should().Be(expectedIndex);
    }

    [Fact]
    public void AddHoliday_ShouldAppendHoliday_WhenDateIsUnique()
    {
        // Arrange
        var viewModel = CreateViewModel();
        viewModel.NewHolidayDate = new DateTime(2026, 12, 25);
        viewModel.NewHolidayName = "Navidad";

        // Act
        viewModel.AddHolidayCommand.Execute(null);

        // Assert
        viewModel.Holidays.Should().ContainSingle(h =>
            h.Date == new DateTime(2026, 12, 25) &&
            h.Name == "Navidad");
    }

    [Fact]
    public void AddHoliday_ShouldNotDuplicateExistingDate()
    {
        // Arrange
        var viewModel = CreateViewModel();
        viewModel.NewHolidayDate = new DateTime(2026, 1, 1);
        viewModel.AddHolidayCommand.Execute(null);
        viewModel.NewHolidayName = "Duplicado";

        // Act
        viewModel.AddHolidayCommand.Execute(null);

        // Assert
        viewModel.Holidays.Should().HaveCount(1);
    }

    [Fact]
    public void RemoveHoliday_ShouldRemoveSelectedHoliday()
    {
        // Arrange
        var viewModel = CreateViewModel();
        var holiday = new HolidayModel { Date = new DateTime(2026, 3, 24), Name = "Día Nacional de la Memoria" };
        viewModel.Holidays.Add(holiday);

        // Act
        viewModel.RemoveHolidayCommand.Execute(holiday);

        // Assert
        viewModel.Holidays.Should().BeEmpty();
    }

    [Fact]
    public async Task SyncHolidaysAsync_ShouldMergeRemoteHolidays()
    {
        // Arrange
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

        // Act
        await viewModel.SyncHolidaysAsync();

        // Assert
        viewModel.Holidays.Should().HaveCount(2);
        await _holidayService.Received(1).GetHolidaysAsync(currentYear);
        await _holidayService.Received(1).GetHolidaysAsync(currentYear + 1);
    }

    [Fact]
    public async Task ApplyChangesAsync_ShouldPersistSettings()
    {
        // Arrange
        var viewModel = CreateViewModel();
        viewModel.AppName = "ElectroObra QA";
        viewModel.MultiplierSaturday = 1.75m;
        viewModel.MultiplierSunday = 2.25m;
        viewModel.MultiplierHoliday = 2.5m;

        // Act
        await viewModel.ApplyChangesCommand.ExecuteAsync(null);

        // Assert
        await _settingsService.Received(1).SetAppNameAsync("ElectroObra QA");
        await _settingsService.Received(1).SetDefaultMultiplierSaturdayAsync(1.75m);
        await _settingsService.Received(1).SetDefaultMultiplierSundayAsync(2.25m);
        await _settingsService.Received(1).SetDefaultMultiplierHolidayAsync(2.5m);
    }
}
