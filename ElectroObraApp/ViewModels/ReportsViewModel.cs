using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class ReportsViewModel : ViewModelBase
{
    private readonly IMovimientoService _movimientoService;
    private readonly IExportService _exportService;
    private readonly IFileSaveDialogService _fileSaveDialogService;
    private readonly INotificationService _notificationService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private string _title = string.Empty;

    [ObservableProperty]
    private string _subtitle = string.Empty;

    [ObservableProperty]
    private bool _isExporting;

    public ReportsViewModel(
        IMovimientoService movimientoService,
        IExportService exportService,
        IFileSaveDialogService fileSaveDialogService,
        INotificationService notificationService,
        ILocalizationService localizationService)
    {
        _movimientoService = movimientoService;
        _exportService = exportService;
        _fileSaveDialogService = fileSaveDialogService;
        _notificationService = notificationService;
        _localizationService = localizationService;
        _title = _localizationService.GetString("Reports.Title");
        _subtitle = _localizationService.GetString("Reports.Subtitle");

        ExportMovimientosPdfCommand = new AsyncRelayCommand(() => ExportMovimientosAsync("pdf", _exportService.ExportMovimientosToPdfAsync));
        ExportMovimientosExcelCommand = new AsyncRelayCommand(() => ExportMovimientosAsync("xlsx", _exportService.ExportMovimientosToExcelAsync));
        ExportMovimientosWordCommand = new AsyncRelayCommand(() => ExportMovimientosAsync("docx", _exportService.ExportMovimientosToWordAsync));
        ExportMovimientosCsvCommand = new AsyncRelayCommand(() => ExportMovimientosAsync("csv", _exportService.ExportMovimientosToCsvAsync));
        ExportMovimientosJsonCommand = new AsyncRelayCommand(ExportMovimientosJsonAsync);
    }

    public IAsyncRelayCommand ExportMovimientosPdfCommand { get; }
    public IAsyncRelayCommand ExportMovimientosExcelCommand { get; }
    public IAsyncRelayCommand ExportMovimientosWordCommand { get; }
    public IAsyncRelayCommand ExportMovimientosCsvCommand { get; }
    public IAsyncRelayCommand ExportMovimientosJsonCommand { get; }

    private async Task ExportMovimientosAsync(string extension, Func<System.Collections.Generic.IEnumerable<object>, Task<byte[]>> exporter)
    {
        if (IsExporting) return;

        IsExporting = true;
        ErrorMessage = null;

        try
        {
            var movimientos = await _movimientoService.GetAllAsync();
            var bytes = await exporter(movimientos);
            await _fileSaveDialogService.SaveFileAsync(
                bytes,
                $"Movimientos_{DateTime.Now:yyyyMMddHHmmss}.{extension}",
                extension);
            await _notificationService.ShowInfoAsync(
                _localizationService.GetString("General.Success"),
                _localizationService.GetString("Reports.ExportSuccess"));
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
        }
        finally
        {
            IsExporting = false;
        }
    }

    private async Task ExportMovimientosJsonAsync()
    {
        if (IsExporting) return;

        IsExporting = true;
        ErrorMessage = null;

        try
        {
            var movimientos = await _movimientoService.GetAllAsync();
            var json = System.Text.Json.JsonSerializer.Serialize(movimientos);
            var bytes = System.Text.Encoding.UTF8.GetBytes(json);
            await _fileSaveDialogService.SaveFileAsync(
                bytes,
                $"Movimientos_{DateTime.Now:yyyyMMddHHmmss}.json",
                "json");
            await _notificationService.ShowInfoAsync(
                _localizationService.GetString("General.Success"),
                _localizationService.GetString("Reports.ExportSuccess"));
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
        }
        finally
        {
            IsExporting = false;
        }
    }
}
