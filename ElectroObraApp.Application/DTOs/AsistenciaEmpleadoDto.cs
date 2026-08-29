using System;
using ElectroObraApp.Application.Common;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Application.DTOs;

public class AsistenciaEmpleadoDto : IHasGuidId
{
    public Guid Id { get; set; }
    public Guid EmpleadoId { get; set; }
    public string? EmpleadoNombre { get; set; }
    public DateTime Fecha { get; set; }
    public TipoJornada TipoJornada { get; set; } = TipoJornada.Completa;
    public Guid? TrabajoId { get; set; }
    public string? TrabajoDescripcion { get; set; }
    public string? Observaciones { get; set; }
}
