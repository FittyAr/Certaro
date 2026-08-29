using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class EmpleadoSeleccionItem : ObservableObject
{
    public EmpleadoSeleccionItem(EmpleadoDto empleado)
    {
        Empleado = empleado;
    }

    public EmpleadoDto Empleado { get; }

    public Guid Id => Empleado.Id;

    public string Nombre => Empleado.Nombre;

    [ObservableProperty]
    private bool _isSelected;
}

public partial class LiquidacionPreviewItem : ObservableObject
{
    [ObservableProperty]
    private LiquidacionDto _liquidacion;

    public LiquidacionPreviewItem(LiquidacionDto liquidacion)
    {
        _liquidacion = liquidacion;
    }

    public string EmpleadoNombre => Liquidacion.EmpleadoNombre;

    public decimal DiasTrabajados
    {
        get => Liquidacion.DiasTrabajados;
        set
        {
            if (Liquidacion.DiasTrabajados == value) return;
            Liquidacion.DiasTrabajados = value;
            OnPropertyChanged();
        }
    }

    public decimal TotalBruto
    {
        get => Liquidacion.TotalBruto;
        set
        {
            if (Liquidacion.TotalBruto == value) return;
            Liquidacion.TotalBruto = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(TotalNeto));
        }
    }

    public decimal TotalAdelantos => Liquidacion.TotalAdelantos;

    public decimal TotalNeto => Liquidacion.TotalNeto;

    public void ApplyLiquidacion(LiquidacionDto liquidacion)
    {
        Liquidacion = liquidacion;
        OnPropertyChanged(nameof(EmpleadoNombre));
        OnPropertyChanged(nameof(DiasTrabajados));
        OnPropertyChanged(nameof(TotalBruto));
        OnPropertyChanged(nameof(TotalAdelantos));
        OnPropertyChanged(nameof(TotalNeto));
    }
}

public partial class LiquidacionesViewModel : ViewModelBase
{
    private const int WizardStepPeriod = 1;
    private const int WizardStepReview = 2;
    private const int WizardStepConfirm = 3;

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
    private bool _isWizardActive;

    [ObservableProperty]
    private int _wizardStep = WizardStepPeriod;

    [ObservableProperty]
    private DateTime _fechaInicio = DateTime.Today.AddDays(-15);

    [ObservableProperty]
    private DateTime _fechaFin = DateTime.Today;

    [ObservableProperty]
    private ObservableCollection<EmpleadoSeleccionItem> _empleadosSeleccionables = new();

    [ObservableProperty]
    private ObservableCollection<LiquidacionPreviewItem> _previewLiquidaciones = new();

    public bool HasLiquidaciones => Liquidaciones.Count > 0;
    public bool ShowEmptyMessage => !IsLoading && !IsWizardActive && Liquidaciones.Count == 0;
    public bool IsStepPeriod => WizardStep == WizardStepPeriod;
    public bool IsStepReview => WizardStep == WizardStepReview;
    public bool IsStepConfirm => WizardStep == WizardStepConfirm;
    public bool CanGoPrevious => WizardStep > WizardStepPeriod;
    public bool CanGoNext => WizardStep < WizardStepConfirm;
    public bool IsLastStep => WizardStep == WizardStepConfirm;

    public string StepIndicatorText => string.Format(
        _localizationService.GetString("Settlements.Wizard.StepIndicator"),
        WizardStep,
        WizardStepConfirm);

    public string StepTitle => WizardStep switch
    {
        WizardStepPeriod => _localizationService.GetString("Settlements.Wizard.Step1Title"),
        WizardStepReview => _localizationService.GetString("Settlements.Wizard.Step2Title"),
        _ => _localizationService.GetString("Settlements.Wizard.Step3Title")
    };

    public decimal BatchTotalBruto => PreviewLiquidaciones.Sum(x => x.TotalBruto);
    public decimal BatchTotalAdelantos => PreviewLiquidaciones.Sum(x => x.TotalAdelantos);
    public decimal BatchTotalNeto => PreviewLiquidaciones.Sum(x => x.TotalNeto);
    public int SelectedEmployeesCount => EmpleadosSeleccionables.Count(x => x.IsSelected);

    public string SelectedEmployeesText => string.Format(
        _localizationService.GetString("Settlements.Wizard.SelectedCount"),
        SelectedEmployeesCount);

    public DateTime? FechaInicioOffset
    {
        get => FechaInicio;
        set
        {
            if (value.HasValue && FechaInicio != value.Value)
            {
                FechaInicio = value.Value;
                OnPropertyChanged();
            }
        }
    }

    public DateTime? FechaFinOffset
    {
        get => FechaFin;
        set
        {
            if (value.HasValue && FechaFin != value.Value)
            {
                FechaFin = value.Value;
                OnPropertyChanged();
            }
        }
    }

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
        StartWizardCommand = new AsyncRelayCommand(StartWizardAsync);
        CancelWizardCommand = new RelayCommand(CancelWizard);
        NextStepCommand = new AsyncRelayCommand(NextStepAsync, CanAdvanceWizard);
        PreviousStepCommand = new RelayCommand(PreviousStep, () => CanGoPrevious);
        ConfirmBatchCommand = new AsyncRelayCommand(ConfirmBatchAsync, () => PreviewLiquidaciones.Count > 0);
        SelectAllEmployeesCommand = new RelayCommand(SelectAllEmployees);
        ClearEmployeeSelectionCommand = new RelayCommand(ClearEmployeeSelection);
        RecalculatePreviewCommand = new AsyncRelayCommand(() => RecalculatePreviewAsync());
    }

    public IAsyncRelayCommand LoadCommand { get; }
    public IAsyncRelayCommand<LiquidacionDto> ExportPdfCommand { get; }
    public IAsyncRelayCommand<LiquidacionDto> ShareEmailCommand { get; }
    public IAsyncRelayCommand<LiquidacionDto> ShareWhatsAppCommand { get; }
    public IAsyncRelayCommand StartWizardCommand { get; }
    public IRelayCommand CancelWizardCommand { get; }
    public IAsyncRelayCommand NextStepCommand { get; }
    public IRelayCommand PreviousStepCommand { get; }
    public IAsyncRelayCommand ConfirmBatchCommand { get; }
    public IRelayCommand SelectAllEmployeesCommand { get; }
    public IRelayCommand ClearEmployeeSelectionCommand { get; }
    public IAsyncRelayCommand RecalculatePreviewCommand { get; }

    partial void OnIsWizardActiveChanged(bool value)
    {
        OnPropertyChanged(nameof(ShowEmptyMessage));
    }

    partial void OnWizardStepChanged(int value)
    {
        OnPropertyChanged(nameof(IsStepPeriod));
        OnPropertyChanged(nameof(IsStepReview));
        OnPropertyChanged(nameof(IsStepConfirm));
        OnPropertyChanged(nameof(CanGoPrevious));
        OnPropertyChanged(nameof(CanGoNext));
        OnPropertyChanged(nameof(IsLastStep));
        OnPropertyChanged(nameof(StepIndicatorText));
        OnPropertyChanged(nameof(StepTitle));
        NextStepCommand.NotifyCanExecuteChanged();
        PreviousStepCommand.NotifyCanExecuteChanged();
        ConfirmBatchCommand.NotifyCanExecuteChanged();
    }

    partial void OnPreviewLiquidacionesChanged(ObservableCollection<LiquidacionPreviewItem> value)
    {
        NotifyBatchTotalsChanged();
        ConfirmBatchCommand.NotifyCanExecuteChanged();
    }

    private void NotifyBatchTotalsChanged()
    {
        OnPropertyChanged(nameof(BatchTotalBruto));
        OnPropertyChanged(nameof(BatchTotalAdelantos));
        OnPropertyChanged(nameof(BatchTotalNeto));
    }

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

    private async Task StartWizardAsync()
    {
        ErrorMessage = null;
        WizardStep = WizardStepPeriod;
        FechaInicio = DateTime.Today.AddDays(-15);
        FechaFin = DateTime.Today;
        PreviewLiquidaciones.Clear();

        var empleados = await _empleadoService.GetAllAsync();
        EmpleadosSeleccionables = new ObservableCollection<EmpleadoSeleccionItem>(
            empleados
                .Where(e => e.Activo)
                .OrderBy(e => e.Nombre)
                .Select(e => new EmpleadoSeleccionItem(e)));

        foreach (var item in EmpleadosSeleccionables)
        {
            item.PropertyChanged += (_, args) =>
            {
                if (args.PropertyName == nameof(EmpleadoSeleccionItem.IsSelected))
                {
                    OnPropertyChanged(nameof(SelectedEmployeesCount));
                    OnPropertyChanged(nameof(SelectedEmployeesText));
                    NextStepCommand.NotifyCanExecuteChanged();
                }
            };
        }

        IsWizardActive = true;
        OnPropertyChanged(nameof(SelectedEmployeesCount));
    }

    private void CancelWizard()
    {
        IsWizardActive = false;
        WizardStep = WizardStepPeriod;
        PreviewLiquidaciones.Clear();
        ErrorMessage = null;
        OnPropertyChanged(nameof(ShowEmptyMessage));
    }

    private bool CanAdvanceWizard()
    {
        if (WizardStep == WizardStepPeriod)
            return FechaInicio <= FechaFin && SelectedEmployeesCount > 0;

        if (WizardStep == WizardStepReview)
            return PreviewLiquidaciones.Count > 0;

        return false;
    }

    private async Task NextStepAsync()
    {
        ErrorMessage = null;

        if (WizardStep == WizardStepPeriod)
        {
            if (FechaInicio > FechaFin)
            {
                ErrorMessage = _localizationService.GetString("Settlements.Wizard.InvalidPeriod");
                return;
            }

            if (SelectedEmployeesCount == 0)
            {
                ErrorMessage = _localizationService.GetString("Settlements.Wizard.NoEmployeesSelected");
                return;
            }

            await BuildPreviewAsync();
            WizardStep = WizardStepReview;
            return;
        }

        if (WizardStep == WizardStepReview)
        {
            WizardStep = WizardStepConfirm;
        }
    }

    private void PreviousStep()
    {
        if (WizardStep <= WizardStepPeriod) return;
        WizardStep--;
        ErrorMessage = null;
    }

    private void SelectAllEmployees()
    {
        foreach (var item in EmpleadosSeleccionables)
            item.IsSelected = true;

        OnPropertyChanged(nameof(SelectedEmployeesCount));
        OnPropertyChanged(nameof(SelectedEmployeesText));
        NextStepCommand.NotifyCanExecuteChanged();
    }

    private void ClearEmployeeSelection()
    {
        foreach (var item in EmpleadosSeleccionables)
            item.IsSelected = false;

        OnPropertyChanged(nameof(SelectedEmployeesCount));
        OnPropertyChanged(nameof(SelectedEmployeesText));
        NextStepCommand.NotifyCanExecuteChanged();
    }

    private async Task BuildPreviewAsync()
    {
        IsLoading = true;
        try
        {
            var previews = new List<LiquidacionPreviewItem>();

            foreach (var item in EmpleadosSeleccionables.Where(x => x.IsSelected))
            {
                var sugerencia = await _liquidacionService.SugerirLiquidacionAsync(
                    item.Empleado.Id,
                    FechaInicio,
                    FechaFin,
                    0);

                previews.Add(new LiquidacionPreviewItem(sugerencia));
            }

            PreviewLiquidaciones = new ObservableCollection<LiquidacionPreviewItem>(previews);
            NotifyBatchTotalsChanged();
        }
        finally
        {
            IsLoading = false;
        }
    }

    private async Task RecalculatePreviewAsync(bool showLoading = true)
    {
        if (PreviewLiquidaciones.Count == 0) return;

        if (showLoading) IsLoading = true;
        try
        {
            foreach (var preview in PreviewLiquidaciones)
            {
                var sugerencia = await _liquidacionService.SugerirLiquidacionAsync(
                    preview.Liquidacion.EmpleadoId,
                    FechaInicio,
                    FechaFin,
                    preview.Liquidacion.DiasTrabajados);

                preview.ApplyLiquidacion(sugerencia);
            }

            NotifyBatchTotalsChanged();
        }
        finally
        {
            if (showLoading) IsLoading = false;
        }
    }

    private async Task ConfirmBatchAsync()
    {
        if (PreviewLiquidaciones.Count == 0) return;

        IsLoading = true;
        try
        {
            await RecalculatePreviewAsync(showLoading: false);

            var dtos = PreviewLiquidaciones.Select(x => x.Liquidacion).ToList();
            var result = await _liquidacionService.CreateBatchAsync(dtos);

            if (!result.IsSuccess)
            {
                ErrorMessage = result.Error ?? _localizationService.GetString("General.Error");
                return;
            }

            await _notificationService.ShowInfoAsync(
                _localizationService.GetString("General.Success"),
                string.Format(
                    _localizationService.GetString("Settlements.Wizard.BatchSuccess"),
                    dtos.Count));

            CancelWizard();
            await LoadAsync();
        }
        finally
        {
            IsLoading = false;
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
