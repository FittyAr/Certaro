using System;
using ElectroObraApp.Application.Common;

namespace ElectroObraApp.Application.DTOs;

public class TipoConceptoPagoDto : IHasGuidId
{
    public Guid Id { get; set; }
    public string Nombre { get; set; } = string.Empty;
    public bool EsSistema { get; set; }
}
