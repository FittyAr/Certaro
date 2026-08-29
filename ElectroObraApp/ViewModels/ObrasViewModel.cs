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
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.ViewModels;

public partial class ObrasViewModel : ViewModelBase
{
    private readonly IObraService _obraService;
    private readonly IClienteService _clienteService;
    private readonly IUserSettingsService _settingsService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;

    [ObservableProperty]
    private ObservableCollection<ObraDto> _obras = new();

    [ObservableProperty]
    private int _pageSize;

    [ObservableProperty]
    private int _currentPage = 1;

    [ObservableProperty]
    private ObservableCollection<int> _pageSizeOptions = new() { 10, 30, 50, 100, 0 };

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private ObraEditViewModel? _editViewModel;

    [ObservableProperty]
    private string _filtroNombre = string.Empty;

    [ObservableProperty]
    private Guid? _filtroClienteId;

    [ObservableProperty]
    private int _filtroEstadoIndex;

    [ObservableProperty]
    private EstadoObra? _filtroEstado;

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    public ObrasViewModel(
        IObraService obraService,
        IClienteService clienteService,
        IUserSettingsService settingsService,
        IConfirmDialogService confirmDialogService,
        ILocalizationService localizationService,
        IServiceProvider serviceProvider)
    {
        _obraService = obraService;
        _clienteService = clienteService;
        _settingsService = settingsService;
        _confirmDialogService = confirmDialogService;
        _localizationService = localizationService;
        _serviceProvider = serviceProvider;
        _pageSize = _settingsService.GetPageSize();

        LoadObrasCommand = new AsyncRelayCommand(LoadObrasAsync);
        AddCommand = new AsyncRelayCommand(AddAsync);
        EditCommand = new RelayCommand<ObraDto>(Edit);
        DeleteCommand = new AsyncRelayCommand<ObraDto>(DeleteAsync);
        LimpiarFiltrosCommand = new RelayCommand(LimpiarFiltros);

        _ = LoadInitialDataAsync();
    }

    public IAsyncRelayCommand LoadObrasCommand { get; }
    public IAsyncRelayCommand AddCommand { get; }
    public IRelayCommand<ObraDto> EditCommand { get; }
    public IAsyncRelayCommand<ObraDto> DeleteCommand { get; }
    public IRelayCommand LimpiarFiltrosCommand { get; }

    partial void OnPageSizeChanged(int value)
    {
        _ = _settingsService.SetPageSizeAsync(value);
        _ = LoadObrasAsync();
    }

    partial void OnFiltroNombreChanged(string value) => _ = LoadObrasAsync();
    partial void OnFiltroClienteIdChanged(Guid? value) => _ = LoadObrasAsync();

    partial void OnFiltroEstadoIndexChanged(int value)
    {
        FiltroEstado = value switch
        {
            1 => EstadoObra.Activa,
            2 => EstadoObra.Pausada,
            3 => EstadoObra.Finalizada,
            4 => EstadoObra.Cancelada,
            _ => null
        };
        _ = LoadObrasAsync();
    }

    private async Task LoadInitialDataAsync()
    {
        var cls = await _clienteService.GetAllAsync();
        Clientes = new ObservableCollection<ClienteDto>(cls);
        await LoadObrasAsync();
    }

    private void LimpiarFiltros()
    {
        FiltroNombre = string.Empty;
        FiltroClienteId = null;
        FiltroEstadoIndex = 0;
        FiltroEstado = null;
        _ = LoadObrasAsync();
    }

    public async Task LoadObrasAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var result = await _obraService.GetAllAsync();
            var query = result.AsEnumerable();

            if (!string.IsNullOrWhiteSpace(FiltroNombre))
            {
                query = query.Where(o =>
                    o.Nombre.Contains(FiltroNombre, StringComparison.OrdinalIgnoreCase) ||
                    o.Numero.ToString().Contains(FiltroNombre, StringComparison.OrdinalIgnoreCase) ||
                    (o.Direccion != null && o.Direccion.Contains(FiltroNombre, StringComparison.OrdinalIgnoreCase)) ||
                    (o.Localidad != null && o.Localidad.Contains(FiltroNombre, StringComparison.OrdinalIgnoreCase)));
            }

            if (FiltroClienteId.HasValue)
                query = query.Where(o => o.ClienteId == FiltroClienteId.Value);

            if (FiltroEstado.HasValue)
                query = query.Where(o => o.Estado == FiltroEstado.Value);

            IEnumerable<ObraDto> paginated;
            if (PageSize > 0)
                paginated = query.Skip((CurrentPage - 1) * PageSize).Take(PageSize);
            else
                paginated = query;

            Obras = new ObservableCollection<ObraDto>(paginated);
            IsEmpty = !Obras.Any();
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

    private async Task AddAsync()
    {
        var vm = _serviceProvider.GetRequiredService<ObraEditViewModel>();
        vm.CloseRequest += (s, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadObrasAsync();
        };

        await vm.LoadDataAsync();
        var all = await _obraService.GetAllAsync();
        var nextNumero = all.Any() ? all.Max(o => o.Numero) + 1 : 1;
        vm.Obra = new ObraDto { Numero = nextNumero, Estado = EstadoObra.Activa };
        vm.Title = _localizationService.GetString("Obras.NewTitle");
        EditViewModel = vm;
        IsEditing = true;
    }

    private void Edit(ObraDto? dto)
    {
        if (dto == null) return;

        var vm = _serviceProvider.GetRequiredService<ObraEditViewModel>();
        vm.Obra = dto.Adapt<ObraDto>();
        vm.Title = _localizationService.GetString("Obras.EditTitle");
        vm.CloseRequest += (s, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadObrasAsync();
        };
        _ = vm.LoadDataAsync();
        EditViewModel = vm;
        IsEditing = true;
    }

    private async Task DeleteAsync(ObraDto? dto)
    {
        if (dto == null) return;

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("General.Delete"),
            string.Format(_localizationService.GetString("Obras.DeleteConfirm"), dto.Nombre));

        if (!confirmed) return;

        var result = await _obraService.DeleteAsync(dto.Id);
        if (HandleResult(result, _localizationService))
            await LoadObrasAsync();
    }
}
