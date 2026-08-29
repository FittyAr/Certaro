using System;
using ElectroObraApp.Application.Common;

namespace ElectroObraApp.Application.DTOs;

public class AdjuntoDto : IHasGuidId
{
    public Guid Id { get; set; }
    public string EntidadTipo { get; set; } = string.Empty;
    public Guid EntidadId { get; set; }
    public string NombreArchivo { get; set; } = string.Empty;
    public string RutaRelativa { get; set; } = string.Empty;
    public string Mime { get; set; } = string.Empty;
    public long Tamano { get; set; }
}
