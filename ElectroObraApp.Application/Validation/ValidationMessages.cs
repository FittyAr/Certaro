namespace ElectroObraApp.Application.Validation;

public static class ValidationMessages
{
    public const string EntityNotFound = "Validation.Common.EntityNotFound";
    public const string SaveFailed = "Validation.Common.SaveFailed";

    public const string CategoriaNombreRequired = "Validation.Categoria.NombreRequired";
    public const string CategoriaNombreMaxLength = "Validation.Categoria.NombreMaxLength";

    public const string ClienteNombreRequired = "Validation.Cliente.NombreRequired";
    public const string ClienteNombreMaxLength = "Validation.Cliente.NombreMaxLength";
    public const string ClienteEmailInvalid = "Validation.Cliente.EmailInvalid";
    public const string ClienteCuitInvalid = "Validation.Cliente.CuitInvalid";
    public const string ClienteContactoEmailInvalid = "Validation.Cliente.ContactoEmailInvalid";
    public const string ClienteContactoEtiquetaRequired = "Validation.Cliente.ContactoEtiquetaRequired";

    public const string EmpleadoNombreRequired = "Validation.Empleado.NombreRequired";
    public const string EmpleadoNombreMaxLength = "Validation.Empleado.NombreMaxLength";
    public const string EmpleadoDniRequired = "Validation.Empleado.DniRequired";
    public const string EmpleadoDniLength = "Validation.Empleado.DniLength";
    public const string EmpleadoTarifaNegative = "Validation.Empleado.TarifaNegative";

    public const string LiquidacionEmpleadoRequired = "Validation.Liquidacion.EmpleadoRequired";
    public const string LiquidacionFechaInicioInvalid = "Validation.Liquidacion.FechaInicioInvalid";
    public const string LiquidacionDiasTrabajadosRequired = "Validation.Liquidacion.DiasTrabajadosRequired";
    public const string LiquidacionTarifaRequired = "Validation.Liquidacion.TarifaRequired";

    public const string MovimientoConceptoRequired = "Validation.Movimiento.ConceptoRequired";
    public const string MovimientoConceptoMaxLength = "Validation.Movimiento.ConceptoMaxLength";
    public const string MovimientoMontoRequired = "Validation.Movimiento.MontoRequired";
    public const string MovimientoCantidadRequired = "Validation.Movimiento.CantidadRequired";
    public const string MovimientoTipoRequired = "Validation.Movimiento.TipoRequired";

    public const string TrabajoDescripcionRequired = "Validation.Trabajo.DescripcionRequired";
    public const string TrabajoDescripcionMaxLength = "Validation.Trabajo.DescripcionMaxLength";
    public const string TrabajoClienteRequired = "Validation.Trabajo.ClienteRequired";

    public const string OrdenTrabajoTituloRequired = "Validation.OrdenTrabajo.TituloRequired";
    public const string OrdenTrabajoTituloMaxLength = "Validation.OrdenTrabajo.TituloMaxLength";

    public const string OrdenTrabajoItemDescripcionRequired = "Validation.OrdenTrabajoItem.DescripcionRequired";
    public const string OrdenTrabajoItemCantidadRequired = "Validation.OrdenTrabajoItem.CantidadRequired";
    public const string OrdenTrabajoItemPrecioNegative = "Validation.OrdenTrabajoItem.PrecioNegative";
    public const string OrdenTrabajoItemPorcentajeInvalid = "Validation.OrdenTrabajoItem.PorcentajeInvalid";

    public const string FacturaNumeroRequired = "Validation.Factura.NumeroRequired";
    public const string FacturaNumeroMaxLength = "Validation.Factura.NumeroMaxLength";
    public const string FacturaClienteRequired = "Validation.Factura.ClienteRequired";
    public const string FacturaSubtotalInvalid = "Validation.Factura.SubtotalInvalid";
    public const string FacturaIvaInvalid = "Validation.Factura.IvaInvalid";
    public const string FacturaTotalInvalid = "Validation.Factura.TotalInvalid";
}
