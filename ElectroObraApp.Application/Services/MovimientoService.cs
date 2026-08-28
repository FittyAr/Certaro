using System.Collections.Generic;
using System.Threading.Tasks;
using Mapster;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.Validation;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Core.Specifications;
using FluentValidation;

namespace ElectroObraApp.Application.Services;

public class MovimientoService : BaseCrudService<Movimiento, MovimientoDto>, IMovimientoService
{
    public MovimientoService(IUnitOfWork uow, ILogger<MovimientoService> logger, IValidator<MovimientoDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<MovimientoDto>> GetAllAsync()
    {
        var entities = await Uow.Movimientos.GetAllWithIncludesAsync();
        return entities.Adapt<IEnumerable<MovimientoDto>>();
    }

    public async Task<PagedResult<MovimientoDto>> GetPagedAsync(MovimientoFilterDto filter)
    {
        var spec = new MovimientosPagedSpecification(
            filter.Concepto,
            filter.TipoId,
            filter.FechaDesde,
            filter.FechaHasta,
            filter.MontoMin,
            filter.MontoMax,
            filter.PageNumber,
            filter.PageSize);

        var paged = await Uow.Movimientos.GetPagedAsync(spec);

        return new PagedResult<MovimientoDto>
        {
            Items = paged.Items.Adapt<IReadOnlyList<MovimientoDto>>(),
            TotalCount = paged.TotalCount,
            PageNumber = paged.PageNumber,
            PageSize = paged.PageSize
        };
    }

    public async Task<MovimientoDto?> GetByIdAsync(Guid id)
    {
        var entity = await Uow.Movimientos.GetByIdWithIncludesAsync(id);
        return entity?.Adapt<MovimientoDto>();
    }

    public override async Task<Result> CreateAsync(MovimientoDto dto)
    {
        Logger.LogInformation("Iniciando creación de movimiento: {Concepto} por {Monto}", dto.Concepto, dto.Monto);
        return await base.CreateAsync(dto);
    }
}
