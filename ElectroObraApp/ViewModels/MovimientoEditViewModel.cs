using System;
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
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private MovimientoDto _movimiento = new() { Fecha = DateTime.Now, Cantidad = 1 };

    partial void OnMovimientoChanged(MovimientoDto value)
    {
        OnPropertyChanged(nameof(FechaOffset));
    }

    [ObservableProperty]
    private ObservableCollection<CategoriaDto> _categorias = new();

    [ObservableProperty]
    private ObservableCollection<TipoMovimientoDto> _tiposMovimiento = new();

    [ObservableProperty]
    private ObservableCollection<EmpleadoDto> _empleados = new();

    [ObservableProperty]
    private string _title = "Nuevo Movimiento";

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
        ILocalizationService localizationService)
    {
        _movimientoService = movimientoService;
        _categoriaService = categoriaService;
        _tipoMovimientoService = tipoMovimientoService;
        _empleadoService = empleadoService;
        _localizationService = localizationService;
        
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
        LoadDataCommand = new AsyncRelayCommand(LoadDataAsync);
    }

    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }
    public IAsyncRelayCommand LoadDataCommand { get; }

    public async Task LoadDataAsync()
    {
        var cats = await _categoriaService.GetAllAsync();
        var tipos = await _tipoMovimientoService.GetAllAsync();

        Categorias = new ObservableCollection<CategoriaDto>(cats);
        TiposMovimiento = new ObservableCollection<TipoMovimientoDto>(tipos);
        
        var emps = await _empleadoService.GetAllAsync();
        Empleados = new ObservableCollection<EmpleadoDto>(emps);

        if (Movimiento.Id == Guid.Empty && Movimiento.TipoMovimientoId == Guid.Empty && TiposMovimiento.Any())
        {
            Movimiento.TipoMovimientoId = TiposMovimiento.First().Id;
        }
        
        OnPropertyChanged(nameof(Movimiento));
        OnPropertyChanged(nameof(FechaOffset));
    }

    private async Task SaveAsync()
    {
        var result = Movimiento.Id == Guid.Empty
            ? await _movimientoService.CreateAsync(Movimiento)
            : await _movimientoService.UpdateAsync(Movimiento);

        if (HandleResult(result, _localizationService))
        {
            CloseRequest?.Invoke(this, true);
        }
    }

    private void Cancel()
    {
        CloseRequest?.Invoke(this, false);
    }

    public event EventHandler<bool>? CloseRequest;
}
