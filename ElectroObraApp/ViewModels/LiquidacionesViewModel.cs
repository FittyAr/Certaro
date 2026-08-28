using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class LiquidacionesViewModel : ViewModelBase
{
    private readonly ILiquidacionService _liquidacionService;
    private readonly IExportService _exportService;
    private readonly IMovimientoService _movimientoService;
    private readonly IEmpleadoService _empleadoService;
    private readonly IUserSettingsService _settingsService;
    private readonly INotificationService _notificationService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private ObservableCollection<LiquidacionDto> _liquidaciones = new();

    [ObservableProperty]
    private bool _isLoading;

    public bool HasLiquidaciones => Liquidaciones.Count > 0;
    public bool ShowEmptyMessage => !IsLoading && Liquidaciones.Count == 0;

    public LiquidacionesViewModel(
        ILiquidacionService liquidacionService, 
        IExportService exportService,
        IMovimientoService movimientoService,
        IEmpleadoService empleadoService,
        IUserSettingsService settingsService,
        INotificationService notificationService,
        ILocalizationService localizationService)
    {
        _liquidacionService = liquidacionService;
        _exportService = exportService;
        _movimientoService = movimientoService;
        _empleadoService = empleadoService;
        _settingsService = settingsService;
        _notificationService = notificationService;
        _localizationService = localizationService;

        LoadCommand = new AsyncRelayCommand(LoadAsync);
        ExportPdfCommand = new AsyncRelayCommand<LiquidacionDto>(ExportPdfAsync);
        ShareEmailCommand = new AsyncRelayCommand<LiquidacionDto>(ShareEmailAsync);
        ShareWhatsAppCommand = new AsyncRelayCommand<LiquidacionDto>(ShareWhatsAppAsync);
        NuevaLiquidacionCommand = new RelayCommand(NuevaLiquidacion);
    }

    public Action? OnNuevaLiquidacion { get; set; }

    public IAsyncRelayCommand LoadCommand { get; }
    public IAsyncRelayCommand<LiquidacionDto> ExportPdfCommand { get; }
    public IAsyncRelayCommand<LiquidacionDto> ShareEmailCommand { get; }
    public IAsyncRelayCommand<LiquidacionDto> ShareWhatsAppCommand { get; }
    public IRelayCommand NuevaLiquidacionCommand { get; }

    private void NuevaLiquidacion() => OnNuevaLiquidacion?.Invoke();

    public async Task LoadAsync()
    {
        IsLoading = true;
        try
        {
            var list = await _liquidacionService.GetAllAsync();
            Liquidaciones = new ObservableCollection<LiquidacionDto>(list);
            OnPropertyChanged(nameof(HasLiquidaciones));
            OnPropertyChanged(nameof(ShowEmptyMessage));
        }
        finally
        {
            IsLoading = false;
            OnPropertyChanged(nameof(ShowEmptyMessage));
        }
    }

    private async Task ExportPdfAsync(LiquidacionDto? dto)
    {
        var path = await GenerateAndSavePdfAsync(dto);
        if (path != null)
        {
            await _notificationService.ShowInfoAsync(
                _localizationService.GetString("General.Success"),
                string.Format(_localizationService.GetString("Settlements.ExportSuccess"), path));
        }
    }

    private async Task<string?> GenerateAndSavePdfAsync(LiquidacionDto? dto)
    {
        if (dto == null) return null;

        var todosMovimientos = await _movimientoService.GetAllAsync();
        var adelantos = todosMovimientos.Where(m => 
            m.EmpleadoId == dto.EmpleadoId && 
            m.Fecha >= dto.FechaInicio && 
            m.Fecha <= dto.FechaFin &&
            m.TipoMovimientoId == Guid.Parse("00000000-0000-0000-0000-000000000003"));

        var pdf = await _exportService.ExportLiquidacionToPdfAsync(dto, adelantos);
        
        var fileName = $"Reporte_Liquidacion_{dto.EmpleadoNombre}_{dto.FechaFin:yyyyMMdd}.pdf";
        var path = System.IO.Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), fileName);
        await System.IO.File.WriteAllBytesAsync(path, pdf);
        return path;
    }

    private async Task ShareEmailAsync(LiquidacionDto? dto)
    {
        if (dto == null) return;
        
        var empleado = await _empleadoService.GetByIdAsync(dto.EmpleadoId);
        if (empleado == null || string.IsNullOrWhiteSpace(empleado.Email))
        {
            await _notificationService.ShowWarningAsync(
                _localizationService.GetString("General.Error"),
                string.Format(_localizationService.GetString("Settlements.NoEmail"), dto.EmpleadoNombre));
            return;
        }

        try 
        {
            var path = await GenerateAndSavePdfAsync(dto);
            var subject = string.Format(_localizationService.GetString("Settlements.EmailSubject"), dto.FechaInicio, dto.FechaFin);
            Application.Helpers.EmailHelper.OpenEmailClient(empleado.Email, _settingsService, subject);
            if (path != null)
            {
                await _notificationService.ShowInfoAsync(
                    _localizationService.GetString("General.Success"),
                    string.Format(_localizationService.GetString("Settlements.EmailOpened"), empleado.Email));
            }
        }
        catch (Exception ex)
        {
            Serilog.Log.Error(ex, "Error al compartir liquidación por email");
            await _notificationService.ShowErrorAsync(
                _localizationService.GetString("General.Error"),
                _localizationService.GetString("Settlements.EmailError"));
        }
    }

    private async Task ShareWhatsAppAsync(LiquidacionDto? dto)
    {
        if (dto == null) return;
        
        var empleado = await _empleadoService.GetByIdAsync(dto.EmpleadoId);
        if (empleado == null || string.IsNullOrWhiteSpace(empleado.Telefono))
        {
            await _notificationService.ShowWarningAsync(
                _localizationService.GetString("General.Error"),
                string.Format(_localizationService.GetString("Settlements.NoPhone"), dto.EmpleadoNombre));
            return;
        }

        try 
        {
            var path = await GenerateAndSavePdfAsync(dto);
            
            var mensaje = string.Format(
                _localizationService.GetString("Settlements.WhatsAppMessage"),
                empleado.Nombre, dto.FechaInicio, dto.FechaFin);
            var url = $"https://api.whatsapp.com/send?phone={empleado.Telefono}&text={Uri.EscapeDataString(mensaje)}";
            
            System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo(url) { UseShellExecute = true });
            if (path != null)
            {
                await _notificationService.ShowInfoAsync(
                    _localizationService.GetString("General.Success"),
                    string.Format(_localizationService.GetString("Settlements.WhatsAppOpened"), empleado.Telefono));
            }
        }
        catch (Exception ex)
        {
            Serilog.Log.Error(ex, "Error al compartir liquidación por WhatsApp");
            await _notificationService.ShowErrorAsync(
                _localizationService.GetString("General.Error"),
                _localizationService.GetString("Settlements.WhatsAppError"));
        }
    }
}
