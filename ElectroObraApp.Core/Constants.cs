using System;

namespace ElectroObraApp.Core;

public static class Constants
{
    public static class TiposMovimiento
    {
        public static readonly Guid Ingreso = Guid.Parse("00000000-0000-0000-0000-000000000001");
        public static readonly Guid Gasto = Guid.Parse("00000000-0000-0000-0000-000000000002");
        public static readonly Guid Adelanto = Guid.Parse("00000000-0000-0000-0000-000000000003");
        public static readonly Guid Ajuste = Guid.Parse("00000000-0000-0000-0000-000000000004");
    }

    public static class EntidadesAdjunto
    {
        public const string Obra = "Obra";
        public const string Trabajo = "Trabajo";
        public const string Factura = "Factura";
        public const string Movimiento = "Movimiento";
        public const string Empleado = "Empleado";
    }
}
