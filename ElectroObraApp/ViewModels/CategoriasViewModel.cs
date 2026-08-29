using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class CategoriasViewModel : ViewModelBase
{
    private readonly ICategoriaService _categoriaService;
    private readonly IConfirmDialogService _confirmDialogService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private ObservableCollection<CategoriaDto> _categorias = new();

    [ObservableProperty]
    private CategoriaDto _editItem = new();

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private string _title = string.Empty;

    public CategoriasViewModel(
        ICategoriaService categoriaService,
        IConfirmDialogService confirmDialogService,
        ILocalizationService localizationService)
    {
        _categoriaService = categoriaService;
        _confirmDialogService = confirmDialogService;
        _localizationService = localizationService;
        _title = _localizationService.GetString("Categories.Title");

        LoadCommand = new AsyncRelayCommand(LoadAsync);
        AddCommand = new RelayCommand(Add);
        EditCommand = new RelayCommand<CategoriaDto>(Edit);
        DeleteCommand = new AsyncRelayCommand<CategoriaDto>(DeleteAsync);
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
        _ = LoadAsync();
    }

    public IAsyncRelayCommand LoadCommand { get; }
    public IRelayCommand AddCommand { get; }
    public IRelayCommand<CategoriaDto> EditCommand { get; }
    public IAsyncRelayCommand<CategoriaDto> DeleteCommand { get; }
    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }

    private void Add()
    {
        EditItem = new CategoriaDto();
        IsEditing = true;
    }

    private void Edit(CategoriaDto? dto)
    {
        if (dto is null) return;
        EditItem = new CategoriaDto
        {
            Id = dto.Id,
            Nombre = dto.Nombre,
            Descripcion = dto.Descripcion,
            ColorHex = dto.ColorHex,
            Icono = dto.Icono
        };
        IsEditing = true;
    }

    private void Cancel() => IsEditing = false;

    private async Task SaveAsync()
    {
        var result = EditItem.Id == Guid.Empty
            ? await _categoriaService.CreateAsync(EditItem)
            : await _categoriaService.UpdateAsync(EditItem);

        if (HandleResult(result, _localizationService))
        {
            IsEditing = false;
            await LoadAsync();
        }
    }

    private async Task DeleteAsync(CategoriaDto? dto)
    {
        if (dto is null) return;

        var confirmed = await _confirmDialogService.ConfirmAsync(
            _localizationService.GetString("General.Delete"),
            string.Format(_localizationService.GetString("Categories.DeleteConfirm"), dto.Nombre));

        if (!confirmed) return;

        var result = await _categoriaService.DeleteAsync(dto.Id);
        if (HandleResult(result, _localizationService))
            await LoadAsync();
    }

    public async Task LoadAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var items = await _categoriaService.GetAllAsync();
            Categorias = new ObservableCollection<CategoriaDto>(items);
            IsEmpty = !Categorias.Any();
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
