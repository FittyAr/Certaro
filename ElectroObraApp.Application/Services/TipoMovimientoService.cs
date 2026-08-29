using System.Collections.Generic;
using System.Threading.Tasks;
using Mapster;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using FluentValidation;

namespace ElectroObraApp.Application.Services;

public class TipoMovimientoService : BaseCrudService<TipoMovimiento, TipoMovimientoDto>, ITipoMovimientoService
{
    public TipoMovimientoService(
        IUnitOfWork uow,
        ILogger<TipoMovimientoService> logger,
        IValidator<TipoMovimientoDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<TipoMovimientoDto>> GetAllAsync()
    {
        var entities = await Repository.GetAllAsync();
        return entities.Adapt<IEnumerable<TipoMovimientoDto>>();
    }

    public async Task<TipoMovimientoDto?> GetByIdAsync(Guid id)
    {
        var entity = await Repository.GetByIdAsync(id);
        return entity?.Adapt<TipoMovimientoDto>();
    }

    public override async Task<Result> UpdateAsync(TipoMovimientoDto dto)
    {
        var existing = await Repository.GetByIdAsync(dto.Id);
        if (existing is null)
            return Result.Failure("El tipo de movimiento no fue encontrado.");

        if (existing.EsSistema)
            dto.EsIngreso = existing.EsIngreso;

        dto.EsSistema = existing.EsSistema;
        return await base.UpdateAsync(dto);
    }

    public override async Task<Result> DeleteAsync(Guid id)
    {
        var existing = await Repository.GetByIdAsync(id);
        if (existing is null)
            return Result.Failure("El tipo de movimiento no fue encontrado.");

        if (existing.EsSistema)
            return Result.Failure("No se pueden eliminar tipos de movimiento del sistema.");

        return await base.DeleteAsync(id);
    }
}
