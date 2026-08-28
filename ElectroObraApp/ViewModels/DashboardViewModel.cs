using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.DTOs;
using LiveChartsCore;
using LiveChartsCore.SkiaSharpView;
using LiveChartsCore.SkiaSharpView.Painting;
using SkiaSharp;
using CommunityToolkit.Mvvm.Messaging;

namespace ElectroObraApp.ViewModels;

public class TopClienteDto
{
    public string Nombre { get; set; } = string.Empty;
    public decimal Total { get; set; }
}

public partial class DashboardViewModel : ViewModelBase
{
    private readonly IDashboardService _dashboardService;
    private readonly IUserSettingsService _settingsService;
    private readonly IDollarService _dollarService;

    [ObservableProperty]
    private string _title = "Dashboard Operativo";

    [ObservableProperty]
    private ObservableCollection<DollarDto> _dollarRates = new();

    [ObservableProperty]
    private bool _showDollarRates = false;

    [ObservableProperty]
    private decimal _totalIngresos;

    [ObservableProperty]
    private decimal _totalGastos;

    [ObservableProperty]
    private decimal _balance;

    [ObservableProperty]
    private decimal _rentabilidad;

    [ObservableProperty]
    private int _clientesActivos;

    [ObservableProperty]
    private int _trabajosPendientes;

    [ObservableProperty]
    private int _liquidacionesPendientesCount;

    [ObservableProperty]
    private string _databaseStatus = "Saludable";

    [ObservableProperty]
    private bool _isPrivacyModeActive;

    partial void OnIsPrivacyModeActiveChanged(bool value)
    {
        _ = _settingsService.SetIsPrivacyModeAsync(value);
        OnPropertyChanged(nameof(DisplayTotalIngresos));
        OnPropertyChanged(nameof(DisplayTotalGastos));
        OnPropertyChanged(nameof(DisplayBalance));
        OnPropertyChanged(nameof(DisplayRentabilidad));
        OnPropertyChanged(nameof(ChartTooltipPosition));
        OnPropertyChanged(nameof(PieTooltipPosition));
    }

    [ObservableProperty]
    private string _currentTimeRange = "Mensual";

    partial void OnCurrentTimeRangeChanged(string value)
    {
        _ = LoadStatsAsync();
    }

    public ObservableCollection<string> TimeRanges { get; } = new() { "Mensual", "Anual", "Total" };
    public ObservableCollection<TopClienteDto> TopClientes { get; } = new();
    public ObservableCollection<MovimientoDto> RecentMovimientos { get; } = new();

    public string DisplayTotalIngresos => IsPrivacyModeActive ? "$ *********" : TotalIngresos.ToString("C");
    public string DisplayTotalGastos => IsPrivacyModeActive ? "$ *********" : TotalGastos.ToString("C");
    public string DisplayBalance => IsPrivacyModeActive ? "$ *********" : Balance.ToString("C");
    public string DisplayRentabilidad => IsPrivacyModeActive ? "** %**" : $"{Rentabilidad:N1} %";

    public LiveChartsCore.Measure.TooltipPosition ChartTooltipPosition => IsPrivacyModeActive
        ? LiveChartsCore.Measure.TooltipPosition.Hidden
        : LiveChartsCore.Measure.TooltipPosition.Bottom;

    public LiveChartsCore.Measure.TooltipPosition PieTooltipPosition => IsPrivacyModeActive
        ? LiveChartsCore.Measure.TooltipPosition.Hidden
        : LiveChartsCore.Measure.TooltipPosition.Right;

    public string LiquidacionesPendientesText => LiquidacionesPendientesCount > 0
        ? $"{LiquidacionesPendientesCount} Liquidaciones Pendientes"
        : "Personal al día";

    public bool ShowLiquidacionesAlert => LiquidacionesPendientesCount > 0;

    public Func<LiveChartsCore.Kernel.ChartPoint, string> PieFormatter =>
        point => $"{point.Context.Series.Name}: {point.Coordinate.PrimaryValue:C}";

    public ObservableCollection<ISeries> Series { get; set; } = new();
    public ObservableCollection<ISeries> CategorySeries { get; set; } = new();

    public Axis[] XAxes { get; set; } =
    {
        new Axis { Labels = new string[] { "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic" } }
    };

    public DashboardViewModel(
        IDashboardService dashboardService,
        IUserSettingsService settingsService,
        IDollarService dollarService)
    {
        _dashboardService = dashboardService;
        _settingsService = settingsService;
        _dollarService = dollarService;

        CurrentTimeRange = settingsService.GetDashboardPeriod();
        IsPrivacyModeActive = settingsService.GetIsPrivacyMode();

        LoadStatsCommand = new AsyncRelayCommand(LoadStatsAsync);
        NavigateToAlertCommand = new RelayCommand<string>(NavigateToAlert);
        TogglePrivacyModeCommand = new RelayCommand(() => IsPrivacyModeActive = !IsPrivacyModeActive);
        _ = LoadStatsAsync();
        _ = LoadDollarRatesAsync();
    }

    private async Task LoadDollarRatesAsync()
    {
        if (!_settingsService.GetAutoUpdateDollar()) return;

        var rates = await _dollarService.GetDollarRatesAsync();
        DollarRates.Clear();
        foreach (var rate in rates.Where(r => r.Casa == "oficial" || r.Casa == "blue"))
            DollarRates.Add(rate);
        ShowDollarRates = DollarRates.Any();
    }

    public IAsyncRelayCommand LoadStatsCommand { get; }
    public IRelayCommand TogglePrivacyModeCommand { get; }
    public IRelayCommand<string> NavigateToAlertCommand { get; }

    private void NavigateToAlert(string? destination)
    {
        if (string.IsNullOrEmpty(destination)) return;
        WeakReferenceMessenger.Default.Send(destination);
    }

    public async Task LoadStatsAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var stats = await _dashboardService.GetStatsAsync(CurrentTimeRange);

            TotalIngresos = stats.TotalIngresos;
            TotalGastos = stats.TotalGastos;
            Balance = stats.Balance;
            Rentabilidad = stats.Rentabilidad;
            ClientesActivos = stats.ClientesActivos;
            TrabajosPendientes = stats.TrabajosPendientes;
            LiquidacionesPendientesCount = stats.LiquidacionesPendientes;
            DatabaseStatus = stats.DatabaseStatus;

            OnPropertyChanged(nameof(LiquidacionesPendientesText));
            OnPropertyChanged(nameof(ShowLiquidacionesAlert));
            OnPropertyChanged(nameof(DisplayRentabilidad));

            TopClientes.Clear();
            foreach (var t in stats.TopClientes)
                TopClientes.Add(new TopClienteDto { Nombre = t.Nombre, Total = t.Total });

            RecentMovimientos.Clear();
            foreach (var r in stats.RecentMovimientos)
                RecentMovimientos.Add(r);

            Series.Clear();
            Series.Add(new ColumnSeries<double>
            {
                Name = "Ingresos",
                Values = stats.MonthlyIncome,
                Stroke = new SolidColorPaint(SKColors.LightGreen) { StrokeThickness = 2 },
                Fill = new SolidColorPaint(SKColors.LightGreen.WithAlpha(100)),
                YToolTipLabelFormatter = point => $"Ingresos: {point.Coordinate.PrimaryValue:C}"
            });
            Series.Add(new ColumnSeries<double>
            {
                Name = "Gastos",
                Values = stats.MonthlyExpenses,
                Stroke = new SolidColorPaint(SKColors.Salmon) { StrokeThickness = 2 },
                Fill = new SolidColorPaint(SKColors.Salmon.WithAlpha(100)),
                YToolTipLabelFormatter = point => $"Gastos: {point.Coordinate.PrimaryValue:C}"
            });

            CategorySeries.Clear();
            foreach (var cat in stats.CategoryExpenses)
            {
                CategorySeries.Add(new PieSeries<double>
                {
                    Name = cat.Name,
                    Values = new double[] { cat.Value },
                    ToolTipLabelFormatter = point => $"{point.Coordinate.PrimaryValue:C}",
                    DataLabelsFormatter = point => $"{point.Coordinate.PrimaryValue:C}"
                });
            }

            IsEmpty = stats.TotalIngresos == 0 && stats.TotalGastos == 0 && !TopClientes.Any();
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
            IsEmpty = false;
        }
        finally
        {
            IsLoading = false;
        }
    }
}
