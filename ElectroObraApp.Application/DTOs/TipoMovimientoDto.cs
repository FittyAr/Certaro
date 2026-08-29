using System;
using ElectroObraApp.Application.Common;

namespace ElectroObraApp.Application.DTOs;

public class TipoMovimientoDto : IHasGuidId
{
    public Guid Id { get; set; }
    public string Nombre { get; set; } = string.Empty;
    public string? Descripcion { get; set; }
    public bool EsIngreso { get; set; }
    public bool EsSistema { get; set; }
}

