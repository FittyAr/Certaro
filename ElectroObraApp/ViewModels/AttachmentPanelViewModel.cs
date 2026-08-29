using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class AttachmentPanelViewModel : ViewModelBase
{
    private readonly IAdjuntoService _adjuntoService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private string _entidadTipo = string.Empty;

    [ObservableProperty]
    private Guid _entidadId;

    [ObservableProperty]
    private ObservableCollection<AdjuntoDto> _adjuntos = new();

    [ObservableProperty]
    private bool _canManage;

    public AttachmentPanelViewModel(
        IAdjuntoService adjuntoService,
        ILocalizationService localizationService)
    {
        _adjuntoService = adjuntoService;
        _localizationService = localizationService;

        AddFilesCommand = new AsyncRelayCommand<IEnumerable<string>>(AddFilesAsync);
        DeleteCommand = new AsyncRelayCommand<AdjuntoDto>(DeleteAsync);
        OpenCommand = new AsyncRelayCommand<AdjuntoDto>(OpenAsync);
    }

    public IAsyncRelayCommand<IEnumerable<string>> AddFilesCommand { get; }
    public IAsyncRelayCommand<AdjuntoDto> DeleteCommand { get; }
    public IAsyncRelayCommand<AdjuntoDto> OpenCommand { get; }

    partial void OnEntidadTipoChanged(string value) => UpdateCanManage();

    partial void OnEntidadIdChanged(Guid value) => UpdateCanManage();

    private void UpdateCanManage() =>
        CanManage = !string.IsNullOrWhiteSpace(EntidadTipo) && EntidadId != Guid.Empty;

    public async Task LoadAsync()
    {
        if (!CanManage)
        {
            Adjuntos.Clear();
            IsEmpty = true;
            return;
        }

        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var items = await _adjuntoService.GetByEntidadAsync(EntidadTipo, EntidadId);
            Adjuntos = new ObservableCollection<AdjuntoDto>(items);
            IsEmpty = Adjuntos.Count == 0;
        }
        catch (Exception)
        {
            ErrorMessage = _localizationService.GetString("Attachments.ErrorLoad");
            IsEmpty = true;
        }
        finally
        {
            IsLoading = false;
        }
    }

    private async Task AddFilesAsync(IEnumerable<string>? filePaths)
    {
        if (!CanManage || filePaths == null)
            return;

        ErrorMessage = null;

        foreach (var path in filePaths.Where(p => !string.IsNullOrWhiteSpace(p)))
        {
            try
            {
                var added = await _adjuntoService.AddFromFileAsync(EntidadTipo, EntidadId, path);
                Adjuntos.Add(added);
                IsEmpty = false;
            }
            catch (Exception)
            {
                ErrorMessage = _localizationService.GetString("Attachments.ErrorAdd");
            }
        }
    }

    private async Task DeleteAsync(AdjuntoDto? adjunto)
    {
        if (adjunto == null)
            return;

        ErrorMessage = null;

        try
        {
            await _adjuntoService.DeleteAsync(adjunto.Id);
            Adjuntos.Remove(adjunto);
            IsEmpty = Adjuntos.Count == 0;
        }
        catch (Exception)
        {
            ErrorMessage = _localizationService.GetString("Attachments.ErrorDelete");
        }
    }

    private async Task OpenAsync(AdjuntoDto? adjunto)
    {
        if (adjunto == null)
            return;

        ErrorMessage = null;

        try
        {
            await _adjuntoService.OpenAsync(adjunto.Id);
        }
        catch (Exception)
        {
            ErrorMessage = _localizationService.GetString("Attachments.ErrorOpen");
        }
    }
}
