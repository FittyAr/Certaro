using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.ViewModels;

public partial class ObraEditViewModel : ViewModelBase
{
    private readonly IObraService _obraService;
    private readonly IClienteService _clienteService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private ObraDto _obra = new() { Estado = EstadoObra.Activa };

    [ObservableProperty]
    private string _title = string.Empty;

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    public ObservableCollection<EstadoObra> EstadosObra { get; } = new(Enum.GetValues<EstadoObra>());

    public ObraEditViewModel(
        IObraService obraService,
        IClienteService clienteService,
        ILocalizationService localizationService)
    {
        _obraService = obraService;
        _clienteService = clienteService;
        _localizationService = localizationService;
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
    }

    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }

    public async Task LoadDataAsync()
    {
        var list = await _clienteService.GetAllAsync();
        Clientes = new ObservableCollection<ClienteDto>(list);
    }

    private async Task SaveAsync()
    {
        var result = Obra.Id == Guid.Empty
            ? await _obraService.CreateAsync(Obra)
            : await _obraService.UpdateAsync(Obra);

        if (HandleResult(result, _localizationService))
            CloseRequest?.Invoke(this, true);
    }

    private void Cancel()
    {
        CloseRequest?.Invoke(this, false);
    }

    public event EventHandler<bool>? CloseRequest;
}
