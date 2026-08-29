using System;
using System.Collections.Generic;
using ElectroObraApp.Core.Enums;

namespace ElectroObraApp.Core.Entities;

public class Empleado : BaseEntity
{
    public string Nombre { get; set; } = string.Empty;
    public string? Dni { get; set; }
    public string? Cargo { get; set; }
    /// <summary>
    /// Salario base según la frecuencia de pago.
    /// </summary>
    public decimal SueldoBase { get; set; }

    /// <summary>
    /// Frecuencia con la que se percibe el sueldo base.
    /// </summary>
    public PaymentFrequency PagoFrecuencia { get; set; } = PaymentFrequency.Mensual;

    /// <summary>
    /// Valor calculado para un día de trabajo. 
    /// Por defecto: SueldoBase / 30 si es mensual, SueldoBase / 15 si es quincenal, etc.
    /// Este valor es el que se utiliza para las liquidaciones por defecto.
    /// </summary>
    public decimal TarifaDiaria { get; set; }
    
    public string? Email { get; set; }
    public string? Telefono { get; set; }
    
    public DateTime FechaIngreso { get; set; }
    public bool Activo { get; set; } = true;

    public ICollection<Movimiento> AdelantosYPagos { get; set; } = new List<Movimiento>();
    public ICollection<Liquidacion> Liquidaciones { get; set; } = new List<Liquidacion>();
    public ICollection<AsistenciaEmpleado> Asistencias { get; set; } = new List<AsistenciaEmpleado>();
}

