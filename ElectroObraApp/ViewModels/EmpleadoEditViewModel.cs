using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public partial class EmpleadoEditViewModel : ViewModelBase
{
    private readonly IEmpleadoService _empleadoService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private EmpleadoDto _empleado = new() { FechaIngreso = DateTime.Now };

    partial void OnEmpleadoChanged(EmpleadoDto value)
    {
        OnPropertyChanged(nameof(FechaIngresoOffset));
    }

    [ObservableProperty]
    private string _title = "Nuevo Empleado";

    public Core.Enums.PaymentFrequency[] PaymentFrequencies => (Core.Enums.PaymentFrequency[])Enum.GetValues(typeof(Core.Enums.PaymentFrequency));

    public DateTimeOffset? FechaIngresoOffset
    {
        get => Empleado.FechaIngreso;
        set
        {
            if (value.HasValue && Empleado.FechaIngreso != value.Value.DateTime)
            {
                Empleado.FechaIngreso = value.Value.DateTime;
                OnPropertyChanged(nameof(FechaIngresoOffset));
            }
        }
    }

    public EmpleadoEditViewModel(IEmpleadoService empleadoService, ILocalizationService localizationService)
    {
        _empleadoService = empleadoService;
        _localizationService = localizationService;
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        CancelCommand = new RelayCommand(Cancel);
    }

    public IAsyncRelayCommand SaveCommand { get; }
    public IRelayCommand CancelCommand { get; }

    private async Task SaveAsync()
    {
        var result = Empleado.Id == Guid.Empty
            ? await _empleadoService.CreateAsync(Empleado)
            : await _empleadoService.UpdateAsync(Empleado);

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
