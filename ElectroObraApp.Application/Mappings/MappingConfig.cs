using Mapster;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Entities;

namespace ElectroObraApp.Application.Mappings;

public class MappingConfig
{
    public static void Configure()
    {
        // CONFIGURACIÓN ENTIDAD -> DTO
        TypeAdapterConfig<Movimiento, MovimientoDto>.NewConfig()
            .Map(dest => dest.TipoMovimientoNombre, src => src.TipoMovimiento!.Nombre)
            .Map(dest => dest.TipoMovimientoSuma, src => src.TipoMovimiento!.EsIngreso)
            .Map(dest => dest.EsIngreso, src => src.TipoMovimiento.EsIngreso)
            .Map(dest => dest.CategoriaNombre, src => src.Categoria != null ? src.Categoria.Nombre : null)
            .Map(dest => dest.ClienteNombre, src => src.Cliente != null ? src.Cliente.Nombre : null);

        TypeAdapterConfig<Trabajo, TrabajoDto>.NewConfig()
            .Map(dest => dest.ClienteNombre, src => src.Cliente!.Nombre)
            .Map(dest => dest.OrdenesTrabajo, src => src.OrdenesTrabajo);

        TypeAdapterConfig<Liquidacion, LiquidacionDto>.NewConfig()
            .Map(dest => dest.EmpleadoNombre, src => src.Empleado!.Nombre);

        TypeAdapterConfig<Factura, FacturaDto>.NewConfig()
            .Map(dest => dest.ClienteNombre, src => src.Cliente != null ? src.Cliente.Nombre : null);

        TypeAdapterConfig<Cliente, ClienteDto>.NewConfig()
            .Map(dest => dest.Contactos, src => src.Contactos);

        TypeAdapterConfig<OrdenTrabajo, OrdenTrabajoDto>.NewConfig()
            .Map(dest => dest.Items, src => src.Items);

        // CONFIGURACIÓN DTO -> ENTIDAD (EVITA STACKOVERFLOW Y PROBLEMAS DE TRACKING)
        TypeAdapterConfig<TrabajoDto, Trabajo>.NewConfig()
            .Ignore(dest => dest.Cliente!)
            .Map(dest => dest.OrdenesTrabajo, src => src.OrdenesTrabajo);

        TypeAdapterConfig<OrdenTrabajoDto, OrdenTrabajo>.NewConfig()
            .Ignore(dest => dest.Trabajo!)
            .Map(dest => dest.Items, src => src.Items);

        TypeAdapterConfig<OrdenTrabajoItemDto, OrdenTrabajoItem>.NewConfig()
            .Ignore(dest => dest.OrdenTrabajo!);

        TypeAdapterConfig<ClienteDto, Cliente>.NewConfig()
            .Map(dest => dest.Contactos, src => src.Contactos)
            .Ignore(dest => dest.Movimientos);

        TypeAdapterConfig<ClienteContactoDto, ClienteContacto>.NewConfig()
            .Ignore(dest => dest.Cliente!);
            
        TypeAdapterConfig<MovimientoDto, Movimiento>.NewConfig()
            .Ignore(dest => dest.TipoMovimiento)
            .Ignore(dest => dest.Categoria!)
            .Ignore(dest => dest.Empleado!)
            .Ignore(dest => dest.Cliente!)
            .Ignore(dest => dest.Trabajo!);
            
        TypeAdapterConfig<EmpleadoDto, Empleado>.NewConfig()
            .Ignore(dest => dest.AdelantosYPagos)
            .Ignore(dest => dest.Liquidaciones);

        TypeAdapterConfig<LiquidacionDto, Liquidacion>.NewConfig()
            .Ignore(dest => dest.Empleado!);

        TypeAdapterConfig<FacturaDto, Factura>.NewConfig()
            .Ignore(dest => dest.Cliente!)
            .Ignore(dest => dest.Movimientos);
    }
}

