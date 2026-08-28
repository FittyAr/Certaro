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

public partial class FacturasViewModel : ViewModelBase
{
    private readonly IFacturaService _facturaService;
    private readonly IUserSettingsService _settingsService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;

    [ObservableProperty]
    private ObservableCollection<FacturaDto> _facturas = new();

    [ObservableProperty]
    private int _pageSize;

    [ObservableProperty]
    private int _currentPage = 1;

    [ObservableProperty]
    private ObservableCollection<int> _pageSizeOptions = new() { 10, 30, 50, 100, 0 };

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private FacturaEditViewModel? _editViewModel;

    [ObservableProperty]
    private string _filtroNumero = string.Empty;

    public FacturasViewModel(
        IFacturaService facturaService,
        IUserSettingsService settingsService,
        IConfirmDialogService confirmDialogService,
        ILocalizationService localizationService,
        IServiceProvider serviceProvider)
    {
        _facturaService = facturaService;
        _settingsService = settingsService;
        _confirmDialogService = confirmDialogService;
        _localizationService = localizationService;
        _serviceProvider = serviceProvider;
        _pageSize = _settingsService.GetPageSize();

        LoadFacturasCommand = new AsyncRelayCommand(LoadFacturasAsync);
        AddCommand = new RelayCommand(Add);
        EditCommand = new RelayCommand<FacturaDto>(Edit);
        DeleteCommand = new AsyncRelayCommand<FacturaDto>(DeleteAsync);
        LimpiarFiltrosCommand = new RelayCommand(LimpiarFiltros);

        _ = LoadFacturasAsync();
    }

    public IAsyncRelayCommand LoadFacturasCommand { get; }
    public IRelayCommand AddCommand { get; }
    public IRelayCommand<FacturaDto> EditCommand { get; }
    public IAsyncRelayCommand<FacturaDto> DeleteCommand { get; }
    public IRelayCommand LimpiarFiltrosCommand { get; }

    partial void OnPageSizeChanged(int value)
    {
        _ = _settingsService.SetPageSizeAsync(value);
        _ = LoadFacturasAsync();
    }

    partial void OnFiltroNumeroChanged(string value) => _ = LoadFacturasAsync();

    private void LimpiarFiltros()
    {
        FiltroNumero = string.Empty;
        _ = LoadFacturasAsync();
    }

    public async Task LoadFacturasAsync()
    {
        var result = await _facturaService.GetAllAsync();
        var query = result.AsEnumerable();

        if (!string.IsNullOrWhiteSpace(FiltroNumero))
        {
            query = query.Where(f =>
                f.Numero.Contains(FiltroNumero, StringComparison.OrdinalIgnoreCase) ||
                (f.ClienteNombre != null && f.ClienteNombre.Contains(FiltroNumero, StringComparison.OrdinalIgnoreCase)));
        }

        IEnumerable<FacturaDto> paginated = PageSize > 0
            ? query.Skip((CurrentPage - 1) * PageSize).Take(PageSize)
            : query;

        Facturas = new ObservableCollection<FacturaDto>(paginated);
    }

    private void Add()
    {
        var vm = _serviceProvider.GetRequiredService<FacturaEditViewModel>();
        vm.CloseRequest += (_, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadFacturasAsync();
        };
        _ = vm.LoadDataAsync();
        EditViewModel = vm;
        IsEditing = true;
    }

    private void Edit(FacturaDto? dto)
    {
        if (dto == null) return;
        var vm = _serviceProvider.GetRequiredService<FacturaEditViewModel>();
        vm.Factura = dto.Adapt<FacturaDto>();
        vm.Title = _localizationService.GetString("Invoices.EditTitle");
        vm.CloseRequest += (_, success) =>
        {
            IsEditing = false;
            EditViewModel = null;
            if (success) _ = LoadFacturasAsync();
        };
        _ = vm.LoadDataAsync();
        EditViewModel = vm;
        IsEditing = true;
    }

    private async Task DeleteAsync(FacturaDto? dto)
    {
        if (dto == null) return;

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("General.Delete"),
            string.Format(_localizationService.GetString("Invoices.DeleteConfirm"), dto.Numero));

        if (!confirmed) return;

        var result = await _facturaService.DeleteAsync(dto.Id);
        if (HandleResult(result, _localizationService))
            await LoadFacturasAsync();
    }
}
