using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Globalization;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Media;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.ViewModels;

public partial class AsistenciaDayCellViewModel : ObservableObject
{
    private readonly AsistenciaViewModel _owner;

    public AsistenciaDayCellViewModel(
        AsistenciaViewModel owner,
        Guid empleadoId,
        string empleadoNombre,
        DateTime fecha,
        AsistenciaEmpleadoDto? existing)
    {
        _owner = owner;
        EmpleadoId = empleadoId;
        EmpleadoNombre = empleadoNombre;
        Fecha = fecha.Date;
        AsistenciaId = existing?.Id;
        _hasRecord = existing is not null;
        _tipoJornada = existing?.TipoJornada ?? TipoJornada.Completa;
    }

    public Guid EmpleadoId { get; }

    public string EmpleadoNombre { get; }

    public DateTime Fecha { get; }

    public int DayNumber => Fecha.Day;

    public Guid? AsistenciaId { get; private set; }

    [ObservableProperty]
    private TipoJornada _tipoJornada;

    [ObservableProperty]
    private bool _hasRecord;

    public string DisplaySymbol => HasRecord
        ? _owner.GetTipoJornadaSymbol(TipoJornada)
        : _owner.GetEmptySymbol();

    public string ToolTipText => HasRecord
        ? _owner.GetTipoJornadaLabel(TipoJornada)
        : _owner.GetEmptyLabel();

    public IBrush BackgroundBrush => _owner.GetTipoJornadaBrush(HasRecord ? TipoJornada : null);

    public void ApplyRecord(AsistenciaEmpleadoDto dto)
    {
        AsistenciaId = dto.Id;
        HasRecord = true;
        TipoJornada = dto.TipoJornada;
        NotifyVisualsChanged();
    }

    public void ClearRecord()
    {
        AsistenciaId = null;
        HasRecord = false;
        TipoJornada = TipoJornada.Completa;
        NotifyVisualsChanged();
    }

    partial void OnTipoJornadaChanged(TipoJornada value) => NotifyVisualsChanged();

    partial void OnHasRecordChanged(bool value) => NotifyVisualsChanged();

    private void NotifyVisualsChanged()
    {
        OnPropertyChanged(nameof(DisplaySymbol));
        OnPropertyChanged(nameof(ToolTipText));
        OnPropertyChanged(nameof(BackgroundBrush));
    }
}

public partial class AsistenciaRowViewModel : ObservableObject
{
    public AsistenciaRowViewModel(Guid empleadoId, string empleadoNombre, IEnumerable<AsistenciaDayCellViewModel> days)
    {
        EmpleadoId = empleadoId;
        EmpleadoNombre = empleadoNombre;
        Days = new ObservableCollection<AsistenciaDayCellViewModel>(days);
    }

    public Guid EmpleadoId { get; }

    public string EmpleadoNombre { get; }

    public ObservableCollection<AsistenciaDayCellViewModel> Days { get; }
}

public partial class AsistenciaViewModel : ViewModelBase
{
    private static readonly TipoJornada[] CycleOrder =
    [
        TipoJornada.Completa,
        TipoJornada.Media,
        TipoJornada.Falta,
        TipoJornada.FaltaJustificada,
        TipoJornada.Feriado
    ];

    private readonly IAsistenciaService _asistenciaService;
    private readonly IEmpleadoService _empleadoService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private int _selectedYear;

    [ObservableProperty]
    private int _selectedMonth;

    [ObservableProperty]
    private ObservableCollection<AsistenciaRowViewModel> _rows = new();

    [ObservableProperty]
    private ObservableCollection<int> _dayHeaders = new();

    [ObservableProperty]
    private string _monthTitle = string.Empty;

    public AsistenciaViewModel(
        IAsistenciaService asistenciaService,
        IEmpleadoService empleadoService,
        ILocalizationService localizationService)
    {
        _asistenciaService = asistenciaService;
        _empleadoService = empleadoService;
        _localizationService = localizationService;

        var today = DateTime.Today;
        _selectedYear = today.Year;
        _selectedMonth = today.Month;

        LoadCommand = new AsyncRelayCommand(LoadAsync);
        PreviousMonthCommand = new AsyncRelayCommand(PreviousMonthAsync);
        NextMonthCommand = new AsyncRelayCommand(NextMonthAsync);
        CycleCellCommand = new AsyncRelayCommand<AsistenciaDayCellViewModel>(CycleCellAsync);
    }

    public IAsyncRelayCommand LoadCommand { get; }
    public IAsyncRelayCommand PreviousMonthCommand { get; }
    public IAsyncRelayCommand NextMonthCommand { get; }
    public IAsyncRelayCommand<AsistenciaDayCellViewModel> CycleCellCommand { get; }

    partial void OnSelectedYearChanged(int value) => UpdateMonthTitle();

    partial void OnSelectedMonthChanged(int value) => UpdateMonthTitle();

    public async Task LoadAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var periodStart = new DateTime(SelectedYear, SelectedMonth, 1);
            var periodEnd = periodStart.AddMonths(1).AddDays(-1);
            var daysInMonth = periodEnd.Day;

            DayHeaders = new ObservableCollection<int>(Enumerable.Range(1, daysInMonth));
            UpdateMonthTitle();

            var empleados = (await _empleadoService.GetAllAsync())
                .Where(e => e.Activo)
                .OrderBy(e => e.Nombre)
                .ToList();

            var asistencias = (await _asistenciaService.GetByPeriodAsync(periodStart, periodEnd))
                .GroupBy(a => a.EmpleadoId)
                .ToDictionary(
                    g => g.Key,
                    g => g.ToDictionary(x => x.Fecha.Date, x => x));

            var rows = new List<AsistenciaRowViewModel>();

            foreach (var empleado in empleados)
            {
                asistencias.TryGetValue(empleado.Id, out var empleadoAsistencias);
                empleadoAsistencias ??= new Dictionary<DateTime, AsistenciaEmpleadoDto>();

                var cells = new List<AsistenciaDayCellViewModel>();
                for (var day = 1; day <= daysInMonth; day++)
                {
                    var fecha = new DateTime(SelectedYear, SelectedMonth, day);
                    empleadoAsistencias.TryGetValue(fecha, out var existing);
                    cells.Add(new AsistenciaDayCellViewModel(this, empleado.Id, empleado.Nombre, fecha, existing));
                }

                rows.Add(new AsistenciaRowViewModel(empleado.Id, empleado.Nombre, cells));
            }

            Rows = new ObservableCollection<AsistenciaRowViewModel>(rows);
            IsEmpty = Rows.Count == 0;
        }
        finally
        {
            IsLoading = false;
        }
    }

    private async Task PreviousMonthAsync()
    {
        var date = new DateTime(SelectedYear, SelectedMonth, 1).AddMonths(-1);
        SelectedYear = date.Year;
        SelectedMonth = date.Month;
        await LoadAsync();
    }

    private async Task NextMonthAsync()
    {
        var date = new DateTime(SelectedYear, SelectedMonth, 1).AddMonths(1);
        SelectedYear = date.Year;
        SelectedMonth = date.Month;
        await LoadAsync();
    }

    private async Task CycleCellAsync(AsistenciaDayCellViewModel? cell)
    {
        if (cell is null) return;

        var nextTipo = cell.HasRecord
            ? GetNextTipoJornada(cell.TipoJornada)
            : TipoJornada.Completa;

        var dto = new AsistenciaEmpleadoDto
        {
            Id = cell.AsistenciaId ?? Guid.Empty,
            EmpleadoId = cell.EmpleadoId,
            EmpleadoNombre = cell.EmpleadoNombre,
            Fecha = cell.Fecha,
            TipoJornada = nextTipo
        };

        var result = await _asistenciaService.UpsertAsync(dto);
        if (!result.IsSuccess || result.Value is null)
        {
            if (!result.IsSuccess)
                HandleResult(result, _localizationService);
            return;
        }

        cell.ApplyRecord(result.Value);
    }

    private static TipoJornada GetNextTipoJornada(TipoJornada current)
    {
        var index = Array.IndexOf(CycleOrder, current);
        if (index < 0 || index >= CycleOrder.Length - 1)
            return CycleOrder[0];

        return CycleOrder[index + 1];
    }

    private void UpdateMonthTitle()
    {
        var culture = CultureInfo.CurrentUICulture;
        MonthTitle = culture.DateTimeFormat.GetMonthName(SelectedMonth) + $" {SelectedYear}";
    }

    public string GetEmptySymbol() => _localizationService.GetString("Attendance.EmptySymbol");

    public string GetEmptyLabel() => _localizationService.GetString("Attendance.EmptyLabel");

    public string GetTipoJornadaSymbol(TipoJornada tipo) =>
        _localizationService.GetString($"Attendance.TipoJornada.{tipo}.Symbol");

    public string GetTipoJornadaLabel(TipoJornada tipo) =>
        _localizationService.GetString($"Attendance.TipoJornada.{tipo}.Label");

    public IBrush GetTipoJornadaBrush(TipoJornada? tipo) =>
        tipo switch
        {
            TipoJornada.Completa => new SolidColorBrush(Color.Parse("#2ecc71")),
            TipoJornada.Media => new SolidColorBrush(Color.Parse("#f1c40f")),
            TipoJornada.Falta => new SolidColorBrush(Color.Parse("#e74c3c")),
            TipoJornada.FaltaJustificada => new SolidColorBrush(Color.Parse("#e67e22")),
            TipoJornada.Feriado => new SolidColorBrush(Color.Parse("#3498db")),
            _ => new SolidColorBrush(Color.Parse("#3a3a3a"))
        };
}
