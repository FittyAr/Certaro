using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class CuentaCorrienteViewModel : ViewModelBase
{
    private readonly IComercialService _comercialService;
    private readonly IClienteService _clienteService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    [ObservableProperty]
    private ClienteDto? _selectedCliente;

    [ObservableProperty]
    private decimal _totalDeuda;

    [ObservableProperty]
    private decimal _bucket0To30;

    [ObservableProperty]
    private decimal _bucket31To60;

    [ObservableProperty]
    private decimal _bucket61To90;

    [ObservableProperty]
    private decimal _bucketOver90;

    [ObservableProperty]
    private ObservableCollection<CuentaCorrienteItemDto> _items = new();

    public string DisplayTotalDeuda => TotalDeuda.ToString("C");
    public string DisplayBucket0To30 => Bucket0To30.ToString("C");
    public string DisplayBucket31To60 => Bucket31To60.ToString("C");
    public string DisplayBucket61To90 => Bucket61To90.ToString("C");
    public string DisplayBucketOver90 => BucketOver90.ToString("C");

    public CuentaCorrienteViewModel(
        IComercialService comercialService,
        IClienteService clienteService,
        ILocalizationService localizationService)
    {
        _comercialService = comercialService;
        _clienteService = clienteService;
        _localizationService = localizationService;

        LoadCommand = new AsyncRelayCommand(LoadAsync);
        _ = InitializeAsync();
    }

    public IAsyncRelayCommand LoadCommand { get; }

    partial void OnSelectedClienteChanged(ClienteDto? value) => _ = LoadAsync();

    private async Task InitializeAsync()
    {
        IsLoading = true;
        try
        {
            var clientes = await _clienteService.GetAllAsync();
            Clientes = new ObservableCollection<ClienteDto>(clientes.OrderBy(c => c.Nombre));
            SelectedCliente = Clientes.FirstOrDefault();
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
        }
        finally
        {
            IsLoading = false;
        }
    }

    public async Task LoadAsync()
    {
        if (SelectedCliente is null)
        {
            Items.Clear();
            ResetAging();
            IsEmpty = true;
            return;
        }

        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var cuenta = await _comercialService.GetCuentaCorrienteClienteAsync(SelectedCliente.Id);
            var aging = await _comercialService.GetAntiguedadDeudaAsync(SelectedCliente.Id);

            TotalDeuda = cuenta.TotalDeuda;
            Bucket0To30 = aging.Bucket0To30;
            Bucket31To60 = aging.Bucket31To60;
            Bucket61To90 = aging.Bucket61To90;
            BucketOver90 = aging.BucketOver90;

            Items = new ObservableCollection<CuentaCorrienteItemDto>(cuenta.Items);
            IsEmpty = !Items.Any() && TotalDeuda == 0;

            NotifyDisplayPropertiesChanged();
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
            IsEmpty = false;
        }
        finally
        {
            IsLoading = false;
        }
    }

    private void ResetAging()
    {
        TotalDeuda = 0;
        Bucket0To30 = 0;
        Bucket31To60 = 0;
        Bucket61To90 = 0;
        BucketOver90 = 0;
        NotifyDisplayPropertiesChanged();
    }

    private void NotifyDisplayPropertiesChanged()
    {
        OnPropertyChanged(nameof(DisplayTotalDeuda));
        OnPropertyChanged(nameof(DisplayBucket0To30));
        OnPropertyChanged(nameof(DisplayBucket31To60));
        OnPropertyChanged(nameof(DisplayBucket61To90));
        OnPropertyChanged(nameof(DisplayBucketOver90));
    }
}
