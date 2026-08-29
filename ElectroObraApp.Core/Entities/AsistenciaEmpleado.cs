using System;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Core.Entities;

public class AsistenciaEmpleado : BaseEntity
{
    public Guid EmpleadoId { get; set; }
    public Empleado Empleado { get; set; } = null!;

    public DateTime Fecha { get; set; }
    public TipoJornada TipoJornada { get; set; } = TipoJornada.Completa;

    public Guid? TrabajoId { get; set; }
    public Trabajo? Trabajo { get; set; }

    public string? Observaciones { get; set; }
}
