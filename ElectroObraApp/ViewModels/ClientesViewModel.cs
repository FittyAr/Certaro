using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Mapster;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class ClientesViewModel : ViewModelBase
{
    private readonly IClienteService _clienteService;
    private readonly IUserSettingsService _settingsService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    [ObservableProperty]
    private int _pageSize;

    [ObservableProperty]
    private int _currentPage = 1;

    [ObservableProperty]
    private ObservableCollection<int> _pageSizeOptions = new() { 10, 30, 50, 100, 0 };

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private ClienteEditViewModel? _editViewModel;
    
    [ObservableProperty]
    private string _filtroNombre = string.Empty;

    public ClientesViewModel(
        IClienteService clienteService,
        IUserSettingsService settingsService,
        IConfirmDialogService confirmDialogService,
        ILocalizationService localizationService,
        IServiceProvider serviceProvider)
    {
        _clienteService = clienteService;
        _settingsService = settingsService;
        _confirmDialogService = confirmDialogService;
        _localizationService = localizationService;
        _serviceProvider = serviceProvider;
        _pageSize = _settingsService.GetPageSize();

        LoadClientesCommand = new AsyncRelayCommand(LoadClientesAsync);
        AddCommand = new RelayCommand(Add);
        EditCommand = new RelayCommand<ClienteDto>(Edit);
        DeleteCommand = new AsyncRelayCommand<ClienteDto>(DeleteAsync);
        LimpiarFiltrosCommand = new RelayCommand(LimpiarFiltros);
        OpenEmailCommand = new RelayCommand<string>(OpenEmail);

        _ = LoadClientesAsync();
    }

    public IAsyncRelayCommand LoadClientesCommand { get; }
    public IRelayCommand AddCommand { get; }
    public IRelayCommand<ClienteDto> EditCommand { get; }
    public IAsyncRelayCommand<ClienteDto> DeleteCommand { get; }
    public IRelayCommand LimpiarFiltrosCommand { get; }
    public IRelayCommand<string> OpenEmailCommand { get; }

    private void OpenEmail(string? email)
    {
        if (!string.IsNullOrEmpty(email))
        {
            Application.Helpers.EmailHelper.OpenEmailClient(email, _settingsService);
        }
    }

    partial void OnPageSizeChanged(int value)
    {
        _ = _settingsService.SetPageSizeAsync(value);
        _ = LoadClientesAsync();
    }

    partial void OnFiltroNombreChanged(string value) => _ = LoadClientesAsync();

    private void LimpiarFiltros()
    {
        FiltroNombre = string.Empty;
        _ = LoadClientesAsync();
    }

    public async Task LoadClientesAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var result = await _clienteService.GetAllAsync();
            var query = result.AsEnumerable();

            if (!string.IsNullOrWhiteSpace(FiltroNombre))
            {
                query = query.Where(c => c.Nombre.Contains(FiltroNombre, StringComparison.OrdinalIgnoreCase) ||
                                        (c.Cuit != null && c.Cuit.Contains(FiltroNombre)));
            }

            IEnumerable<ClienteDto> paginated;
            if (PageSize > 0)
                paginated = query.Skip((CurrentPage - 1) * PageSize).Take(PageSize);
            else
                paginated = query;

            Clientes = new ObservableCollection<ClienteDto>(paginated);
            IsEmpty = !Clientes.Any();
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

    private void Add()
    {
        var vm = _serviceProvider.GetRequiredService<ClienteEditViewModel>();
        vm.CloseRequest += (s, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadClientesAsync();
        };
        EditViewModel = vm;
        IsEditing = true;
    }

    private void Edit(ClienteDto? dto)
    {
        if (dto == null) return;
        var vm = _serviceProvider.GetRequiredService<ClienteEditViewModel>();
        vm.Cliente = dto.Adapt<ClienteDto>();
        vm.Title = "Editar Cliente";
        vm.CloseRequest += (s, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadClientesAsync();
        };
        EditViewModel = vm;
        IsEditing = true;
    }

    private async Task DeleteAsync(ClienteDto? dto)
    {
        if (dto == null) return;

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("General.Delete"),
            string.Format(_localizationService.GetString("Clients.DeleteConfirm"), dto.Nombre));

        if (!confirmed) return;

        var result = await _clienteService.DeleteAsync(dto.Id);
        if (HandleResult(result, _localizationService))
            await LoadClientesAsync();
    }
}

