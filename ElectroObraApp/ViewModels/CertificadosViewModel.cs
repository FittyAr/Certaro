using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class CertificadosViewModel : ViewModelBase
{
    private readonly ITrabajoService _trabajoService;
    private readonly IExportService _exportService;
    private readonly IFileSaveDialogService _fileSaveDialogService;
    private readonly INotificationService _notificationService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private string _title;

    [ObservableProperty]
    private string _subtitle;

    [ObservableProperty]
    private ObservableCollection<CertificadoDocumentItem> _certificados = new();

    [ObservableProperty]
    private CertificadoDocumentItem? _selectedCertificado;

    public bool HasSelection => SelectedCertificado is not null;
    public bool ShowEmptyDetail => !IsLoading && SelectedCertificado is null;

    public CertificadosViewModel(
        ITrabajoService trabajoService,
        IExportService exportService,
        IFileSaveDialogService fileSaveDialogService,
        INotificationService notificationService,
        ILocalizationService localizationService)
    {
        _trabajoService = trabajoService;
        _exportService = exportService;
        _fileSaveDialogService = fileSaveDialogService;
        _notificationService = notificationService;
        _localizationService = localizationService;

        _title = _localizationService.GetString("Certificates.Title");
        _subtitle = _localizationService.GetString("Certificates.Subtitle");

        LoadCommand = new AsyncRelayCommand(LoadAsync);
        ExportPdfCommand = new AsyncRelayCommand(ExportPdfAsync, () => SelectedCertificado is not null);
    }

    public IAsyncRelayCommand LoadCommand { get; }
    public IAsyncRelayCommand ExportPdfCommand { get; }

    partial void OnSelectedCertificadoChanged(CertificadoDocumentItem? value)
    {
        OnPropertyChanged(nameof(HasSelection));
        OnPropertyChanged(nameof(ShowEmptyDetail));
        ExportPdfCommand.NotifyCanExecuteChanged();
    }

    public async Task LoadAsync()
    {
        IsLoading = true;
        ErrorMessage = null;

        try
        {
            var trabajos = await _trabajoService.GetAllAsync();
            var items = trabajos
                .SelectMany(t => t.OrdenesTrabajo.Select(o => new CertificadoDocumentItem
                {
                    Certificado = o,
                    Trabajo = t
                }))
                .OrderByDescending(c => c.Certificado.Fecha)
                .ThenBy(c => c.ListTitle)
                .ToList();

            Certificados = new ObservableCollection<CertificadoDocumentItem>(items);
            SelectedCertificado = Certificados.FirstOrDefault();
            IsEmpty = Certificados.Count == 0;
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
            IsEmpty = false;
        }
        finally
        {
            IsLoading = false;
            OnPropertyChanged(nameof(ShowEmptyDetail));
        }
    }

    private async Task ExportPdfAsync()
    {
        if (SelectedCertificado is null)
            return;

        var certificado = SelectedCertificado.Certificado;
        var trabajo = SelectedCertificado.Trabajo;
        var bytes = await _exportService.ExportCertificadoToPdfAsync(certificado, trabajo);

        var safeTitle = string.Join("_", certificado.Titulo.Split(Path.GetInvalidFileNameChars(), StringSplitOptions.RemoveEmptyEntries));
        if (string.IsNullOrWhiteSpace(safeTitle))
            safeTitle = certificado.NumeroCertificado ?? "Certificado";

        var fileName = $"Certificado_{safeTitle}_{certificado.Fecha:yyyyMMdd}.pdf";
        var saved = await _fileSaveDialogService.SaveFileAsync(bytes, fileName, "pdf");

        if (saved)
        {
            await _notificationService.ShowInfoAsync(
                _localizationService.GetString("General.Success"),
                _localizationService.GetString("Certificates.ExportSuccess"));
        }
    }
}
