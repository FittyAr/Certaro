using System;
using ElectroObraApp.Application.Common;

namespace ElectroObraApp.Application.DTOs;

public class ClienteDto : IHasGuidId
{
    public Guid Id { get; set; }
    public string Nombre { get; set; } = string.Empty;
    public string? Cuit { get; set; }
    public string? Direccion { get; set; }
    public string? Telefono { get; set; }
    public string? Email { get; set; }
    public string? CondicionIva { get; set; }
    public System.Collections.ObjectModel.ObservableCollection<ClienteContactoDto> Contactos { get; set; } = new();
}

