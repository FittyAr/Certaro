using System;
using System.Collections.ObjectModel;
using System.Threading;
using System.Threading.Tasks;
using Mapster;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class MovimientosViewModel : ViewModelBase
{
    private readonly IMovimientoService _movimientoService;
    private readonly ITipoMovimientoService _tipoMovimientoService;
    private readonly IExportService _exportService;
    private readonly IUserSettingsService _settingsService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly ILocalizationService _localizationService;
    private readonly IFileSaveDialogService _fileSaveDialogService;
    private readonly IServiceProvider _serviceProvider;
    private CancellationTokenSource? _filterDebounceCts;

    [ObservableProperty]
    private ObservableCollection<MovimientoDto> _movimientos = new();

    [ObservableProperty]
    private int _pageSize;

    [ObservableProperty]
    private int _currentPage = 1;

    [ObservableProperty]
    private int _totalPages = 1;

    [ObservableProperty]
    private ObservableCollection<int> _pageSizeOptions = new() { 10, 30, 50, 100, 0 };

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private MovimientoEditViewModel? _editViewModel;

    [ObservableProperty]
    private ObservableCollection<TipoMovimientoDto> _tiposMovimiento = new();

    [ObservableProperty] private string _filtroConcepto = string.Empty;
    [ObservableProperty] private Guid? _filtroTipoId;
    [ObservableProperty] private DateTime? _filtroFechaDesde;
    [ObservableProperty] private DateTime? _filtroFechaHasta;
    [ObservableProperty] private decimal? _filtroMontoMin;
    [ObservableProperty] private decimal? _filtroMontoMax;

    public bool ShowPagination => PageSize > 0;

    public MovimientosViewModel(
        IMovimientoService movimientoService,
        ITipoMovimientoService tipoMovimientoService,
        IExportService exportService,
        IUserSettingsService settingsService,
        IConfirmDialogService confirmDialogService,
        ILocalizationService localizationService,
        IFileSaveDialogService fileSaveDialogService,
        IServiceProvider serviceProvider)
    {
        _movimientoService = movimientoService;
        _tipoMovimientoService = tipoMovimientoService;
        _exportService = exportService;
        _settingsService = settingsService;
        _confirmDialogService = confirmDialogService;
        _localizationService = localizationService;
        _fileSaveDialogService = fileSaveDialogService;
        _serviceProvider = serviceProvider;
        _pageSize = _settingsService.GetPageSize();

        LoadMovimientosCommand = new AsyncRelayCommand(LoadMovimientosAsync);
        AddCommand = new AsyncRelayCommand(OnAddAsync);
        EditCommand = new AsyncRelayCommand<MovimientoDto>(OnEditAsync);
        DeleteCommand = new AsyncRelayCommand<MovimientoDto>(DeleteAsync);
        LimpiarFiltrosCommand = new RelayCommand(LimpiarFiltros);
        PreviousPageCommand = new RelayCommand(GoToPreviousPage, () => CurrentPage > 1);
        NextPageCommand = new RelayCommand(GoToNextPage, () => CurrentPage < TotalPages);

        ExportPdfCommand = new AsyncRelayCommand(ExportPdfAsync);
        ExportExcelCommand = new AsyncRelayCommand(ExportExcelAsync);
        ExportCsvCommand = new AsyncRelayCommand(ExportCsvAsync);
        ExportJsonCommand = new AsyncRelayCommand(ExportJsonAsync);
        ExportWordCommand = new AsyncRelayCommand(ExportWordAsync);

        _ = LoadInitialDataAsync();
    }

    public IAsyncRelayCommand LoadMovimientosCommand { get; }
    public IAsyncRelayCommand AddCommand { get; }
    public IAsyncRelayCommand<MovimientoDto> EditCommand { get; }
    public IAsyncRelayCommand<MovimientoDto> DeleteCommand { get; }
    public IRelayCommand LimpiarFiltrosCommand { get; }
    public IRelayCommand PreviousPageCommand { get; }
    public IRelayCommand NextPageCommand { get; }
    public IAsyncRelayCommand ExportPdfCommand { get; }
    public IAsyncRelayCommand ExportExcelCommand { get; }
    public IAsyncRelayCommand ExportCsvCommand { get; }
    public IAsyncRelayCommand ExportJsonCommand { get; }
    public IAsyncRelayCommand ExportWordCommand { get; }

    partial void OnCurrentPageChanged(int value)
    {
        PreviousPageCommand.NotifyCanExecuteChanged();
        NextPageCommand.NotifyCanExecuteChanged();
    }

    partial void OnTotalPagesChanged(int value)
    {
        PreviousPageCommand.NotifyCanExecuteChanged();
        NextPageCommand.NotifyCanExecuteChanged();
    }

    partial void OnPageSizeChanged(int value)
    {
        OnPropertyChanged(nameof(ShowPagination));
        _ = _settingsService.SetPageSizeAsync(value);
        CurrentPage = 1;
        _ = LoadMovimientosAsync();
    }

    private async Task LoadInitialDataAsync()
    {
        var tipos = await _tipoMovimientoService.GetAllAsync();
        TiposMovimiento = new ObservableCollection<TipoMovimientoDto>(tipos);
        await LoadMovimientosAsync();
    }

    private async Task OnAddAsync()
    {
        var vm = _serviceProvider.GetRequiredService<MovimientoEditViewModel>();
        vm.Title = "Nuevo Movimiento";
        vm.CloseRequest += OnEditFinished;
        await vm.LoadDataAsync();
        vm.Movimiento = new MovimientoDto { Fecha = DateTime.Now };
        EditViewModel = vm;
        IsEditing = true;
    }

    private async Task OnEditAsync(MovimientoDto? dto)
    {
        if (dto == null) return;
        var vm = _serviceProvider.GetRequiredService<MovimientoEditViewModel>();
        vm.Title = "Editar Movimiento";
        vm.CloseRequest += OnEditFinished;
        await vm.LoadDataAsync();
        vm.Movimiento = dto.Adapt<MovimientoDto>();
        EditViewModel = vm;
        IsEditing = true;
    }

    private void OnEditFinished(object? sender, bool saved)
    {
        IsEditing = false;
        EditViewModel = null;
        if (saved) _ = LoadMovimientosAsync();
    }

    private async Task DeleteAsync(MovimientoDto? dto)
    {
        if (dto == null) return;

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("General.Delete"),
            string.Format(_localizationService.GetString("Movements.DeleteConfirm"), dto.Concepto));

        if (!confirmed) return;

        var result = await _movimientoService.DeleteAsync(dto.Id);
        if (HandleResult(result, _localizationService))
            await LoadMovimientosAsync();
    }

    private void LimpiarFiltros()
    {
        FiltroConcepto = string.Empty;
        FiltroTipoId = null;
        FiltroFechaDesde = null;
        FiltroFechaHasta = null;
        FiltroMontoMin = null;
        FiltroMontoMax = null;
        CurrentPage = 1;
        _ = LoadMovimientosAsync();
    }

    private void ScheduleFilterReload()
    {
        _filterDebounceCts?.Cancel();
        _filterDebounceCts = new CancellationTokenSource();
        var token = _filterDebounceCts.Token;
        _ = DebouncedLoadAsync(token);
    }

    private async Task DebouncedLoadAsync(CancellationToken token)
    {
        try
        {
            await Task.Delay(300, token);
            CurrentPage = 1;
            await LoadMovimientosAsync();
        }
        catch (OperationCanceledException)
        {
        }
    }

    partial void OnFiltroConceptoChanged(string value) => ScheduleFilterReload();
    partial void OnFiltroTipoIdChanged(Guid? value) => ScheduleFilterReload();
    partial void OnFiltroFechaDesdeChanged(DateTime? value) => ScheduleFilterReload();
    partial void OnFiltroFechaHastaChanged(DateTime? value) => ScheduleFilterReload();
    partial void OnFiltroMontoMinChanged(decimal? value) => ScheduleFilterReload();
    partial void OnFiltroMontoMaxChanged(decimal? value) => ScheduleFilterReload();

    private async Task ExportPdfAsync()
    {
        var bytes = await _exportService.ExportMovimientosToPdfAsync(Movimientos);
        await SaveFileAsync(bytes, "pdf");
    }

    private async Task ExportExcelAsync()
    {
        var bytes = await _exportService.ExportMovimientosToExcelAsync(Movimientos);
        await SaveFileAsync(bytes, "xlsx");
    }

    private async Task ExportCsvAsync()
    {
        var bytes = await _exportService.ExportMovimientosToCsvAsync(Movimientos);
        await SaveFileAsync(bytes, "csv");
    }

    private async Task ExportJsonAsync()
    {
        var json = System.Text.Json.JsonSerializer.Serialize(Movimientos);
        var bytes = System.Text.Encoding.UTF8.GetBytes(json);
        await SaveFileAsync(bytes, "json");
    }

    private async Task ExportWordAsync()
    {
        var bytes = await _exportService.ExportMovimientosToWordAsync(Movimientos);
        await SaveFileAsync(bytes, "docx");
    }

    private async Task SaveFileAsync(byte[] bytes, string ext)
    {
        await _fileSaveDialogService.SaveFileAsync(
            bytes,
            $"Movimientos_{DateTime.Now:yyyyMMddHHmmss}.{ext}",
            ext);
    }

    private void GoToPreviousPage()
    {
        if (CurrentPage > 1)
        {
            CurrentPage--;
            _ = LoadMovimientosAsync();
        }
    }

    private void GoToNextPage()
    {
        if (CurrentPage < TotalPages)
        {
            CurrentPage++;
            _ = LoadMovimientosAsync();
        }
    }

    public async Task LoadMovimientosAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var filter = new MovimientoFilterDto
            {
                Concepto = string.IsNullOrWhiteSpace(FiltroConcepto) ? null : FiltroConcepto,
                TipoId = FiltroTipoId,
                FechaDesde = FiltroFechaDesde,
                FechaHasta = FiltroFechaHasta,
                MontoMin = FiltroMontoMin,
                MontoMax = FiltroMontoMax,
                PageNumber = CurrentPage,
                PageSize = PageSize
            };

            var result = await _movimientoService.GetPagedAsync(filter);

            Movimientos.Clear();
            foreach (var item in result.Items)
                Movimientos.Add(item);

            TotalPages = Math.Max(1, result.TotalPages);
            IsEmpty = result.TotalCount == 0;
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
            IsEmpty = false;
        }
        finally
        {
            IsLoading = false;
            PreviousPageCommand.NotifyCanExecuteChanged();
            NextPageCommand.NotifyCanExecuteChanged();
        }
    }
}
