using System;
using System.Collections.Generic;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Core.Entities;

public class Obra : BaseEntity
{
    public int Numero { get; set; }
    public string Nombre { get; set; } = string.Empty;
    public string? Direccion { get; set; }
    public string? Localidad { get; set; }
    public Guid ClienteId { get; set; }
    public Cliente Cliente { get; set; } = null!;
    public EstadoObra Estado { get; set; } = EstadoObra.Activa;

    public ICollection<Trabajo> Trabajos { get; set; } = new List<Trabajo>();
}
