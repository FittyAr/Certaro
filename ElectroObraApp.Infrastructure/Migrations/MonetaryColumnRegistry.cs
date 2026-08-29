namespace ElectroObraApp.Infrastructure.Migrations;

/// <summary>
/// Registry of monetary columns stored as scaled long integers (scale 10_000).
/// </summary>
internal static class MonetaryColumnRegistry
{
    public const int Scale = 10_000;

    public static readonly (string Table, string Column)[] Columns =
    [
        ("Movimientos", "Monto"),
        ("Movimientos", "Cantidad"),
        ("Trabajos", "Presupuesto"),
        ("OrdenTrabajoItems", "PrecioUnitario"),
        ("OrdenTrabajoItems", "PorcentajeAnterior"),
        ("OrdenTrabajoItems", "PorcentajeActual"),
        ("OrdenTrabajoItems", "Cantidad"),
        ("OrdenesTrabajo", "OtrosDescuentos"),
        ("OrdenesTrabajo", "AjusteUocraPorcentaje"),
        ("Liquidaciones", "TotalBruto"),
        ("Liquidaciones", "TotalAdelantos"),
        ("Liquidaciones", "TarifaAplicada"),
        ("Liquidaciones", "MultiplicadorSabado"),
        ("Liquidaciones", "MultiplicadorFeriado"),
        ("Liquidaciones", "MultiplicadorDomingo"),
        ("Liquidaciones", "DiasTrabajados"),
        ("Empleados", "TarifaDiaria"),
        ("Empleados", "SueldoBase"),
        ("Facturas", "Subtotal"),
        ("Facturas", "Iva"),
        ("Facturas", "Total"),
    ];
}
