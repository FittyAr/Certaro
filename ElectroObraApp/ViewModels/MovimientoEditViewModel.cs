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

public partial class MovimientoEditViewModel : ViewModelBase
{
    private readonly IMovimientoService _movimientoService;
    private readonly ICategoriaService _categoriaService;
    private readonly ITipoMovimientoService _tipoMovimientoService;
    private readonly IEmpleadoService _empleadoService;
    private readonly IClienteService _clienteService;
    private readonly IObraService _obraService;
    private readonly ITrabajoService _trabajoService;
    private readonly IFacturaService _facturaService;
    private readonly ILocalizationService _localizationService;

    private List<ObraDto> _allObras = new();
    private List<TrabajoDto> _allTrabajos = new();
    private List<FacturaDto> _allFacturas = new();

    [ObservableProperty]
    private MovimientoDto _movimiento = new() { Fecha = DateTime.Now, Cantidad = 1 };

    [ObservableProperty]
    private ObservableCollection<CategoriaDto> _categorias = new();

    [ObservableProperty]
    private ObservableCollection<TipoMovimientoDto> _tiposMovimiento = new();

    [ObservableProperty]
    private ObservableCollection<EmpleadoDto> _empleados = new();

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    [ObservableProperty]
    private ObservableCollection<ObraDto> _obras = new();

    [ObservableProperty]
    private ObservableCollection<TrabajoDto> _trabajos = new();

    [ObservableProperty]
    private ObservableCollection<FacturaDto> _facturas = new();

    [ObservableProperty]
    private Guid? _selectedObraId;

    [ObservableProperty]
    private string _title = "Nuevo Movimiento";

    public bool ShowAttachments => Movimiento.Id != Guid.Empty;

    public Guid? SelectedClienteId
    {
        get => Movimiento.ClienteId;
        set
        {
            if (Movimiento.ClienteId == value)
                return;

            Movimiento.ClienteId = value;
            OnPropertyChanged(nameof(SelectedClienteId));
            SelectedObraId = null;
            Movimiento.TrabajoId = null;
            Movimiento.FacturaId = null;
            RefreshObras();
            RefreshTrabajos();
            RefreshFacturas();
        }
    }

    public DateTimeOffset? FechaOffset
    {
        get => Movimiento.Fecha;
        set
        {
            if (value.HasValue && Movimiento.Fecha != value.Value.DateTime)
            {
                Movimiento.Fecha = value.Value.DateTime;
                OnPropertyChanged(nameof(FechaOffset));
            }
        }
    }

    public MovimientoEditViewModel(
        IMovimientoService movimientoService,
        ICategoriaService categoriaService,
        ITipoMovimientoService tipoMovimientoService,
        IEmpleadoService empleadoService,
        IClienteService clienteService,
        IObraService obraService,
        ITrabajoService trabajoService,
        IFacturaService facturaService,
        ILocalizationService localizationService)
    {
        _movimientoService = movimientoService;
        _categoriaService = categoriaService;
        _tipoMovimientoService = tipoMovimientoService;
        _empleadoService = empleadoService;
        _clienteService = clienteService;
        _obraService = obraService;
        _trabajoService = trabajoService;
        _facturaService = facturaService;
        _localizationService = localizationService;

        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
        LoadDataCommand = new AsyncRelayCommand(LoadDataAsync);
    }

    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }
    public IAsyncRelayCommand LoadDataCommand { get; }

    partial void OnMovimientoChanged(MovimientoDto value)
    {
        OnPropertyChanged(nameof(FechaOffset));
        OnPropertyChanged(nameof(SelectedClienteId));
        OnPropertyChanged(nameof(ShowAttachments));
    }

    partial void OnSelectedObraIdChanged(Guid? value)
    {
        Movimiento.TrabajoId = null;
        RefreshTrabajos();
    }

    public async Task LoadDataAsync()
    {
        var cats = await _categoriaService.GetAllAsync();
        var tipos = await _tipoMovimientoService.GetAllAsync();
        var emps = await _empleadoService.GetAllAsync();
        var clientes = await _clienteService.GetAllAsync();
        _allObras = (await _obraService.GetAllAsync()).ToList();
        _allTrabajos = (await _trabajoService.GetAllAsync()).ToList();
        _allFacturas = (await _facturaService.GetAllAsync()).ToList();

        Categorias = new ObservableCollection<CategoriaDto>(cats);
        TiposMovimiento = new ObservableCollection<TipoMovimientoDto>(tipos);
        Empleados = new ObservableCollection<EmpleadoDto>(emps);
        Clientes = new ObservableCollection<ClienteDto>(clientes);

        if (Movimiento.Id == Guid.Empty && Movimiento.TipoMovimientoId == Guid.Empty && TiposMovimiento.Any())
            Movimiento.TipoMovimientoId = TiposMovimiento.First().Id;

        if (Movimiento.TrabajoId.HasValue)
        {
            var trabajo = _allTrabajos.FirstOrDefault(t => t.Id == Movimiento.TrabajoId);
            SelectedObraId = trabajo?.ObraId;
        }

        RefreshObras();
        RefreshTrabajos();
        RefreshFacturas();
        OnPropertyChanged(nameof(Movimiento));
        OnPropertyChanged(nameof(FechaOffset));
        OnPropertyChanged(nameof(SelectedClienteId));
    }

    private void RefreshObras()
    {
        var query = _allObras.AsEnumerable();
        if (Movimiento.ClienteId.HasValue)
            query = query.Where(o => o.ClienteId == Movimiento.ClienteId);

        Obras = new ObservableCollection<ObraDto>(query);
        if (SelectedObraId.HasValue && Obras.All(o => o.Id != SelectedObraId))
            SelectedObraId = null;
    }

    private void RefreshTrabajos()
    {
        var query = _allTrabajos.AsEnumerable();
        if (Movimiento.ClienteId.HasValue)
            query = query.Where(t => t.ClienteId == Movimiento.ClienteId);
        if (SelectedObraId.HasValue)
            query = query.Where(t => t.ObraId == SelectedObraId);

        Trabajos = new ObservableCollection<TrabajoDto>(query);
        if (Movimiento.TrabajoId.HasValue && Trabajos.All(t => t.Id != Movimiento.TrabajoId))
            Movimiento.TrabajoId = null;
    }

    private void RefreshFacturas()
    {
        var query = _allFacturas.AsEnumerable();
        if (Movimiento.ClienteId.HasValue)
            query = query.Where(f => f.ClienteId == Movimiento.ClienteId);

        Facturas = new ObservableCollection<FacturaDto>(query);
        if (Movimiento.FacturaId.HasValue && Facturas.All(f => f.Id != Movimiento.FacturaId))
            Movimiento.FacturaId = null;
    }

    private async Task SaveAsync()
    {
        var result = Movimiento.Id == Guid.Empty
            ? await _movimientoService.CreateAsync(Movimiento)
            : await _movimientoService.UpdateAsync(Movimiento);

        if (HandleResult(result, _localizationService))
            CloseRequest?.Invoke(this, true);
    }

    private void Cancel() => CloseRequest?.Invoke(this, false);

    public event EventHandler<bool>? CloseRequest;
}
