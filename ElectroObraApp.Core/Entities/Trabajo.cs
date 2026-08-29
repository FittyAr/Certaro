using System;
using System.Collections.Generic;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Core.Entities;

public class Trabajo : BaseEntity
{
    public string Descripcion { get; set; } = string.Empty;
    public DateTime FechaInicio { get; set; }
    public DateTime? FechaFin { get; set; }
    public decimal Presupuesto { get; set; }
    public EstadoTrabajo Estado { get; set; } = EstadoTrabajo.Presupuestado;

    public Guid ObraId { get; set; }
    public Obra Obra { get; set; } = null!;

    public ICollection<Movimiento> GastosEIngresos { get; set; } = new List<Movimiento>();
    public ICollection<OrdenTrabajo> OrdenesTrabajo { get; set; } = new List<OrdenTrabajo>();
    public ICollection<AsistenciaEmpleado> Asistencias { get; set; } = new List<AsistenciaEmpleado>();
}
