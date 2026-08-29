using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Mapster;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.Validation;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using FluentValidation;

namespace ElectroObraApp.Application.Services;

public class AsistenciaService : BaseCrudService<AsistenciaEmpleado, AsistenciaEmpleadoDto>, IAsistenciaService
{
    public AsistenciaService(IUnitOfWork uow, ILogger<AsistenciaService> logger, IValidator<AsistenciaEmpleadoDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<AsistenciaEmpleadoDto>> GetAllAsync()
    {
        var entities = await Repository.GetAllAsync();
        return entities.Adapt<IEnumerable<AsistenciaEmpleadoDto>>();
    }

    public async Task<IEnumerable<AsistenciaEmpleadoDto>> GetByPeriodAsync(DateTime inicio, DateTime fin)
    {
        var entities = await Repository.FindAsync(a =>
            a.Fecha >= inicio.Date &&
            a.Fecha <= fin.Date);

        return entities.Adapt<IEnumerable<AsistenciaEmpleadoDto>>();
    }

    public async Task<AsistenciaEmpleadoDto?> GetByIdAsync(Guid id)
    {
        var entity = await Repository.GetByIdAsync(id);
        return entity?.Adapt<AsistenciaEmpleadoDto>();
    }

    public async Task<Result<AsistenciaEmpleadoDto>> UpsertAsync(AsistenciaEmpleadoDto dto)
    {
        var existing = (await Repository.FindAsync(a =>
            a.EmpleadoId == dto.EmpleadoId &&
            a.Fecha.Date == dto.Fecha.Date)).FirstOrDefault();

        if (existing is null)
        {
            var createResult = await CreateAsync(dto);
            if (!createResult.IsSuccess)
                return Result<AsistenciaEmpleadoDto>.Failure(createResult.Errors);

            var created = (await Repository.FindAsync(a =>
                a.EmpleadoId == dto.EmpleadoId &&
                a.Fecha.Date == dto.Fecha.Date)).FirstOrDefault();

            return created is null
                ? Result<AsistenciaEmpleadoDto>.Failure(ValidationMessages.SaveFailed)
                : Result<AsistenciaEmpleadoDto>.Success(created.Adapt<AsistenciaEmpleadoDto>());
        }

        dto.Id = existing.Id;
        var updateResult = await UpdateAsync(dto);
        if (!updateResult.IsSuccess)
            return Result<AsistenciaEmpleadoDto>.Failure(updateResult.Errors);

        existing = await Repository.GetByIdAsync(existing.Id);
        return existing is null
            ? Result<AsistenciaEmpleadoDto>.Failure(ValidationMessages.EntityNotFound)
            : Result<AsistenciaEmpleadoDto>.Success(existing.Adapt<AsistenciaEmpleadoDto>());
    }
}
