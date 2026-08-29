using System;

namespace ElectroObraApp.Core.Entities;

public class Adjunto : BaseEntity
{
    public string EntidadTipo { get; set; } = string.Empty;
    public Guid EntidadId { get; set; }
    public string NombreArchivo { get; set; } = string.Empty;
    public string RutaRelativa { get; set; } = string.Empty;
    public string Mime { get; set; } = string.Empty;
    public long Tamano { get; set; }
}
