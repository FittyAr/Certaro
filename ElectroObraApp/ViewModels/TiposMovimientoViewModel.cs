using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class TiposMovimientoViewModel : ViewModelBase
{
    private readonly ITipoMovimientoService _tipoMovimientoService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private ObservableCollection<TipoMovimientoDto> _tipos = new();

    [ObservableProperty]
    private TipoMovimientoDto _editItem = new();

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private string _title = string.Empty;

    public TiposMovimientoViewModel(
        ITipoMovimientoService tipoMovimientoService,
        IConfirmDialogService confirmDialogService,
        ILocalizationService localizationService)
    {
        _tipoMovimientoService = tipoMovimientoService;
        _confirmDialogService = confirmDialogService;
        _localizationService = localizationService;
        _title = _localizationService.GetString("MovementTypes.Title");

        LoadCommand = new AsyncRelayCommand(LoadAsync);
        AddCommand = new RelayCommand(Add);
        EditCommand = new RelayCommand<TipoMovimientoDto>(Edit);
        DeleteCommand = new AsyncRelayCommand<TipoMovimientoDto>(DeleteAsync);
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
        _ = LoadAsync();
    }

    public IAsyncRelayCommand LoadCommand { get; }
    public IRelayCommand AddCommand { get; }
    public IRelayCommand<TipoMovimientoDto> EditCommand { get; }
    public IAsyncRelayCommand<TipoMovimientoDto> DeleteCommand { get; }
    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }

    private void Add()
    {
        EditItem = new TipoMovimientoDto();
        IsEditing = true;
    }

    private void Edit(TipoMovimientoDto? dto)
    {
        if (dto is null) return;
        EditItem = new TipoMovimientoDto
        {
            Id = dto.Id,
            Nombre = dto.Nombre,
            Descripcion = dto.Descripcion,
            EsIngreso = dto.EsIngreso,
            EsSistema = dto.EsSistema
        };
        IsEditing = true;
    }

    private void Cancel() => IsEditing = false;

    private async Task SaveAsync()
    {
        var result = EditItem.Id == Guid.Empty
            ? await _tipoMovimientoService.CreateAsync(EditItem)
            : await _tipoMovimientoService.UpdateAsync(EditItem);

        if (HandleResult(result, _localizationService))
        {
            IsEditing = false;
            await LoadAsync();
        }
    }

    private async Task DeleteAsync(TipoMovimientoDto? dto)
    {
        if (dto is null || dto.EsSistema) return;

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("General.Delete"),
            string.Format(_localizationService.GetString("MovementTypes.DeleteConfirm"), dto.Nombre));

        if (!confirmed) return;

        var result = await _tipoMovimientoService.DeleteAsync(dto.Id);
        if (HandleResult(result, _localizationService))
            await LoadAsync();
    }

    public async Task LoadAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var items = await _tipoMovimientoService.GetAllAsync();
            Tipos = new ObservableCollection<TipoMovimientoDto>(items);
            IsEmpty = !Tipos.Any();
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
}
