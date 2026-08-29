using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using ElectroObraApp.Core.Enums;
using Mapster;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class TrabajosViewModel : ViewModelBase
{
    private readonly ITrabajoService _trabajoService;
    private readonly IClienteService _clienteService;
    private readonly IUserSettingsService _settingsService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;

    [ObservableProperty]
    private ObservableCollection<TrabajoDto> _trabajos = new();

    [ObservableProperty]
    private int _pageSize;

    [ObservableProperty]
    private int _currentPage = 1;

    [ObservableProperty]
    private ObservableCollection<int> _pageSizeOptions = new() { 10, 30, 50, 100, 0 };

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private TrabajoEditViewModel? _editViewModel;
    [ObservableProperty]
    private string _filtroNombre = string.Empty;

    [ObservableProperty]
    private Guid? _filtroClienteId;

    [ObservableProperty]
    private EstadoTrabajo? _filtroEstado;

    [ObservableProperty]
    private DateTime? _filtroFechaDesde;

    [ObservableProperty]
    private DateTime? _filtroFechaHasta;

    [ObservableProperty]
    private ObservableCollection<ClienteDto> _clientes = new();

    [ObservableProperty]
    private int _filtroEstadoIndex = 0; // 0: Todos, 1: En Curso, 2: Finalizados

    public TrabajosViewModel(
        ITrabajoService trabajoService, 
        IClienteService clienteService, 
        IUserSettingsService settingsService,
        IConfirmDialogService confirmDialogService,
        ILocalizationService localizationService,
        IServiceProvider serviceProvider)
    {
        _trabajoService = trabajoService;
        _clienteService = clienteService;
        _settingsService = settingsService;
        _confirmDialogService = confirmDialogService;
        _localizationService = localizationService;
        _serviceProvider = serviceProvider;
        _pageSize = _settingsService.GetPageSize();

        LoadTrabajosCommand = new AsyncRelayCommand(LoadTrabajosAsync);
        AddCommand = new AsyncRelayCommand(AddAsync);
        EditCommand = new AsyncRelayCommand<TrabajoDto>(EditAsync);
        DeleteCommand = new AsyncRelayCommand<TrabajoDto>(DeleteAsync);
        LimpiarFiltrosCommand = new RelayCommand(LimpiarFiltros);

        _ = LoadInitialDataAsync();
    }

    public IAsyncRelayCommand LoadTrabajosCommand { get; }
    public IAsyncRelayCommand AddCommand { get; }
    public IAsyncRelayCommand<TrabajoDto> EditCommand { get; }
    public IAsyncRelayCommand<TrabajoDto> DeleteCommand { get; }
    public IRelayCommand LimpiarFiltrosCommand { get; }

    private async Task LoadInitialDataAsync()
    {
        var cls = await _clienteService.GetAllAsync();
        Clientes = new ObservableCollection<ClienteDto>(cls);
        await LoadTrabajosAsync();
    }

    partial void OnFiltroNombreChanged(string value) => _ = LoadTrabajosAsync();
    partial void OnFiltroClienteIdChanged(Guid? value) => _ = LoadTrabajosAsync();
    
    partial void OnFiltroEstadoIndexChanged(int value)
    {
        FiltroEstado = value switch {
            1 => EstadoTrabajo.EnProceso,
            2 => EstadoTrabajo.Finalizado,
            3 => EstadoTrabajo.Pausado,
            _ => null
        };
        _ = LoadTrabajosAsync();
    }
    
    partial void OnFiltroFechaDesdeChanged(DateTime? value) => _ = LoadTrabajosAsync();
    partial void OnFiltroFechaHastaChanged(DateTime? value) => _ = LoadTrabajosAsync();

    private void LimpiarFiltros()
    {
        FiltroNombre = string.Empty;
        FiltroClienteId = null;
        FiltroEstadoIndex = 0;
        FiltroEstado = null;
        FiltroFechaDesde = null;
        FiltroFechaHasta = null;
        _ = LoadTrabajosAsync();
    }

    public async Task LoadTrabajosAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var result = await _trabajoService.GetAllAsync();
            var query = result.AsEnumerable();

            if (!string.IsNullOrWhiteSpace(FiltroNombre))
                query = query.Where(t => t.Descripcion.Contains(FiltroNombre, StringComparison.OrdinalIgnoreCase));

            if (FiltroClienteId.HasValue)
                query = query.Where(t => t.ClienteId == FiltroClienteId.Value);

            if (FiltroEstado.HasValue)
                query = query.Where(t => t.Estado == FiltroEstado.Value);

            if (FiltroFechaDesde.HasValue)
                query = query.Where(t => t.FechaInicio.Date >= FiltroFechaDesde.Value.Date);

            if (FiltroFechaHasta.HasValue)
                query = query.Where(t => t.FechaInicio.Date <= FiltroFechaHasta.Value.Date);

            IEnumerable<TrabajoDto> paginated;
            if (PageSize > 0)
                paginated = query.Skip((CurrentPage - 1) * PageSize).Take(PageSize);
            else
                paginated = query;

            Trabajos = new ObservableCollection<TrabajoDto>(paginated);
            IsEmpty = !Trabajos.Any();
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
        var vm = _serviceProvider.GetRequiredService<TrabajoEditViewModel>();
        vm.CloseRequest += (s, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadTrabajosAsync();
        };
        await vm.LoadDataAsync();
        vm.Trabajo = new TrabajoDto { FechaInicio = DateTime.Now };
        EditViewModel = vm;
        IsEditing = true;
    }

    private async Task EditAsync(TrabajoDto? dto)
    {
        if (dto == null) return;
        var vm = _serviceProvider.GetRequiredService<TrabajoEditViewModel>();
        vm.Title = "Editar Trabajo";
        vm.CloseRequest += (s, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadTrabajosAsync();
        };
        await vm.LoadDataAsync();

        // CRÍTICO: Cargar desde DB con Include(OrdenesTrabajo → Items)
        // El DTO del listado solo tiene datos superficiales, sin certificaciones
        var trabajoCompleto = await _trabajoService.GetByIdAsync(dto.Id);
        vm.Trabajo = trabajoCompleto ?? dto;

        EditViewModel = vm;
        IsEditing = true;
    }

    private async Task DeleteAsync(TrabajoDto? dto)
    {
        if (dto == null) return;

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("General.Delete"),
            string.Format(_localizationService.GetString("Jobs.DeleteConfirm"), dto.Descripcion));

        if (!confirmed) return;

        var result = await _trabajoService.DeleteAsync(dto.Id);
        if (HandleResult(result, _localizationService))
            await LoadTrabajosAsync();
    }
}

