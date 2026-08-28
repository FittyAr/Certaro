using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class ClienteEditViewModel : ViewModelBase
{
    private readonly IClienteService _clienteService;
    private readonly IUserSettingsService _settingsService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private ClienteDto _cliente = new();

    [ObservableProperty]
    private string _title = "Nuevo Cliente";

    public ClienteEditViewModel(
        IClienteService clienteService,
        IUserSettingsService settingsService,
        ILocalizationService localizationService)
    {
        _clienteService = clienteService;
        _settingsService = settingsService;
        _localizationService = localizationService;
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
        AddContactCommand = new RelayCommand(AddContact);
        RemoveContactCommand = new RelayCommand<ClienteContactoDto>(RemoveContact);
        OpenEmailCommand = new RelayCommand(OpenEmail);
    }

    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }
    public IRelayCommand AddContactCommand { get; }
    public IRelayCommand<ClienteContactoDto> RemoveContactCommand { get; }
    public IRelayCommand OpenEmailCommand { get; }

    private void OpenEmail()
    {
        if (!string.IsNullOrEmpty(Cliente.Email))
        {
            Application.Helpers.EmailHelper.OpenEmailClient(Cliente.Email, _settingsService);
        }
    }

    private void AddContact()
    {
        Cliente.Contactos.Add(new ClienteContactoDto { Etiqueta = "General" });
    }

    private void RemoveContact(ClienteContactoDto? contacto)
    {
        if (contacto != null)
        {
            Cliente.Contactos.Remove(contacto);
        }
    }

    private async Task SaveAsync()
    {
        var result = Cliente.Id == Guid.Empty
            ? await _clienteService.CreateAsync(Cliente)
            : await _clienteService.UpdateAsync(Cliente);

        if (HandleResult(result, _localizationService))
        {
            CloseRequest?.Invoke(this, true);
        }
    }

    private void Cancel()
    {
        CloseRequest?.Invoke(this, false);
    }

    public event EventHandler<bool>? CloseRequest;
}
