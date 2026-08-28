using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class TrabajoEditViewModel : ViewModelBase
{
    private readonly ITrabajoService _trabajoService;
    private readonly IClienteService _clienteService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private TrabajoDto _trabajo = new() { FechaInicio = DateTime.Now };

    partial void OnTrabajoChanged(TrabajoDto value)
    {
        OnPropertyChanged(nameof(FechaInicioOffset));
    }

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    [ObservableProperty]
    private string _title = "Nuevo Trabajo";

    public DateTimeOffset? FechaInicioOffset
    {
        get => new DateTimeOffset(Trabajo.FechaInicio);
        set
        {
            if (value.HasValue && Trabajo.FechaInicio != value.Value.DateTime)
            {
                Trabajo.FechaInicio = value.Value.DateTime;
                OnPropertyChanged(nameof(FechaInicioOffset));
            }
        }
    }

    public TrabajoEditViewModel(
        ITrabajoService trabajoService,
        IClienteService clienteService,
        ILocalizationService localizationService)
    {
        _trabajoService = trabajoService;
        _clienteService = clienteService;
        _localizationService = localizationService;
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
        LoadDataCommand = new AsyncRelayCommand(LoadDataAsync);
        AddOrdenCommand = new RelayCommand(AddOrden);
        RemoveOrdenCommand = new RelayCommand<OrdenTrabajoDto>(RemoveOrden);
        AddItemCommand = new RelayCommand<OrdenTrabajoDto>(AddItem);
        RemoveItemCommand = new RelayCommand<OrdenTrabajoItemDto>(RemoveItem);
    }

    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }
    public IAsyncRelayCommand LoadDataCommand { get; }
    public IRelayCommand AddOrdenCommand { get; }
    public IRelayCommand<OrdenTrabajoDto> RemoveOrdenCommand { get; }
    public IRelayCommand<OrdenTrabajoDto> AddItemCommand { get; }
    public IRelayCommand<OrdenTrabajoItemDto> RemoveItemCommand { get; }

    private void AddOrden()
    {
        Trabajo.OrdenesTrabajo.Add(new OrdenTrabajoDto { Titulo = "Nuevo Certificado", Fecha = DateTime.Now });
        OnPropertyChanged(nameof(Trabajo));
    }

    private void RemoveOrden(OrdenTrabajoDto? orden)
    {
        if (orden != null)
        {
            Trabajo.OrdenesTrabajo.Remove(orden);
            OnPropertyChanged(nameof(Trabajo));
        }
    }

    private void AddItem(OrdenTrabajoDto? orden)
    {
        if (orden != null)
        {
            orden.Items.Add(new OrdenTrabajoItemDto { Descripcion = "Nuevo Item", Cantidad = 1 });
            OnPropertyChanged(nameof(Trabajo));
        }
    }

    private void RemoveItem(OrdenTrabajoItemDto? item)
    {
        if (item != null)
        {
            foreach (var orden in Trabajo.OrdenesTrabajo)
            {
                if (orden.Items.Remove(item)) break;
            }
            OnPropertyChanged(nameof(Trabajo));
        }
    }

    public async Task LoadDataAsync()
    {
        var list = await _clienteService.GetAllAsync();
        Clientes = new ObservableCollection<ClienteDto>(list);
    }

    private async Task SaveAsync()
    {
        var result = Trabajo.Id == Guid.Empty
            ? await _trabajoService.CreateAsync(Trabajo)
            : await _trabajoService.UpdateAsync(Trabajo);

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
