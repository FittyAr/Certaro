using System.Collections.Generic;

namespace ElectroObraApp.Core.Entities;

public class TipoConceptoPago : BaseEntity
{
    public string Nombre { get; set; } = string.Empty;
    public bool EsSistema { get; set; }

    public ICollection<Movimiento> Movimientos { get; set; } = new List<Movimiento>();
}
