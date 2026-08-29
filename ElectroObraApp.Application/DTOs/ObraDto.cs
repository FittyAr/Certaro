using System;
using ElectroObraApp.Application.Common;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Application.DTOs;

public class ObraDto : IHasGuidId
{
    public Guid Id { get; set; }
    public int Numero { get; set; }
    public string Nombre { get; set; } = string.Empty;
    public string? Direccion { get; set; }
    public string? Localidad { get; set; }
    public Guid ClienteId { get; set; }
    public string? ClienteNombre { get; set; }
    public EstadoObra Estado { get; set; }
}
