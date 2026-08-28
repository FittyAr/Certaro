using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.ViewModels;

public partial class FacturaEditViewModel : ViewModelBase
{
    private readonly IFacturaService _facturaService;
    private readonly IClienteService _clienteService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private FacturaDto _factura = new();

    [ObservableProperty]
    private string _title = "Nueva Factura";

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    public Array EstadosFactura { get; } = Enum.GetValues(typeof(EstadoFactura));

    public FacturaEditViewModel(
        IFacturaService facturaService,
        IClienteService clienteService,
        ILocalizationService localizationService)
    {
        _facturaService = facturaService;
        _clienteService = clienteService;
        _localizationService = localizationService;
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
    }

    public DateTime? FechaOffset
    {
        get => Factura.Fecha;
        set
        {
            if (value.HasValue && Factura.Fecha != value.Value)
            {
                Factura.Fecha = value.Value;
                OnPropertyChanged(nameof(FechaOffset));
            }
        }
    }

    partial void OnFacturaChanged(FacturaDto value)
    {
        OnPropertyChanged(nameof(FechaOffset));
    }

    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }

    public async Task LoadDataAsync()
    {
        var clientes = await _clienteService.GetAllAsync();
        Clientes = new ObservableCollection<ClienteDto>(clientes);
        if (Factura.Id == Guid.Empty && Clientes.Any())
            Factura.ClienteId = Clientes.First().Id;
    }

    private void RecalculateTotal()
    {
        Factura.Total = Factura.Subtotal + Factura.Iva;
    }

    private async Task SaveAsync()
    {
        RecalculateTotal();
        var result = Factura.Id == Guid.Empty
            ? await _facturaService.CreateAsync(Factura)
            : await _facturaService.UpdateAsync(Factura);

        if (HandleResult(result, _localizationService))
            CloseRequest?.Invoke(this, true);
    }

    private void Cancel() => CloseRequest?.Invoke(this, false);

    public event EventHandler<bool>? CloseRequest;
}
