using System;
using System.Threading.Tasks;
using ElectroObraApp.Application.Common;
using ElectroObraApp.Application.Validation;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using FluentValidation;
using Mapster;
using Microsoft.Extensions.Logging;

namespace ElectroObraApp.Application.Services;

public abstract class BaseCrudService<TEntity, TDto> 
    where TEntity : BaseEntity
    where TDto : class, IHasGuidId
{
    protected IUnitOfWork Uow { get; }
    protected ILogger Logger { get; }
    protected IValidator<TDto> Validator { get; }

    protected BaseCrudService(IUnitOfWork uow, ILogger logger, IValidator<TDto> validator)
    {
        Uow = uow;
        Logger = logger;
        Validator = validator;
    }

    protected IRepository<TEntity> Repository => Uow.Repository<TEntity>();

    protected virtual Task<Result> ValidateAsync(TDto dto) =>
        ValidationPipeline.ValidateAsync(Validator, dto);

    public virtual async Task<Result> CreateAsync(TDto dto)
    {
        var validation = await ValidateAsync(dto);
        if (!validation.IsSuccess)
        {
            Logger.LogWarning("Validación fallida al crear {Entity}: {Errors}",
                typeof(TEntity).Name, string.Join("; ", validation.Errors));
            return validation;
        }

        Logger.LogInformation("Creando {Entity}", typeof(TEntity).Name);
        var entity = dto.Adapt<TEntity>();
        await Repository.AddAsync(entity);

        if (await Uow.SaveChangesAsync() <= 0)
        {
            Logger.LogWarning("No se pudo persistir {Entity}", typeof(TEntity).Name);
            return Result.Failure(ValidationMessages.SaveFailed);
        }

        Logger.LogInformation("{Entity} creado con ID: {Id}", typeof(TEntity).Name, entity.Id);
        return Result.Success();
    }

    public virtual async Task<Result> UpdateAsync(TDto dto)
    {
        var validation = await ValidateAsync(dto);
        if (!validation.IsSuccess)
        {
            Logger.LogWarning("Validación fallida al actualizar {Entity} {Id}: {Errors}",
                typeof(TEntity).Name, dto.Id, string.Join("; ", validation.Errors));
            return validation;
        }

        var entity = await Repository.GetByIdAsync(dto.Id);
        if (entity is null)
        {
            Logger.LogWarning("{Entity} no encontrado: {Id}", typeof(TEntity).Name, dto.Id);
            return Result.Failure(ValidationMessages.EntityNotFound);
        }

        Logger.LogInformation("Actualizando {Entity}: {Id}", typeof(TEntity).Name, dto.Id);
        dto.Adapt(entity);
        entity.UpdatedAt = DateTime.UtcNow;
        Repository.Update(entity);

        if (await Uow.SaveChangesAsync() <= 0)
        {
            Logger.LogWarning("No se pudo actualizar {Entity}: {Id}", typeof(TEntity).Name, dto.Id);
            return Result.Failure(ValidationMessages.SaveFailed);
        }

        return Result.Success();
    }

    public virtual async Task<Result> DeleteAsync(Guid id)
    {
        Logger.LogInformation("Eliminando {Entity}: {Id}", typeof(TEntity).Name, id);
        var entity = await Repository.GetByIdAsync(id);
        if (entity is null)
        {
            Logger.LogWarning("{Entity} no encontrado para eliminar: {Id}", typeof(TEntity).Name, id);
            return Result.Failure(ValidationMessages.EntityNotFound);
        }

        entity.IsDeleted = true;
        entity.DeletedAt = DateTime.UtcNow;
        entity.UpdatedAt = DateTime.UtcNow;
        Repository.Update(entity);

        if (await Uow.SaveChangesAsync() <= 0)
        {
            Logger.LogWarning("No se pudo eliminar {Entity}: {Id}", typeof(TEntity).Name, id);
            return Result.Failure(ValidationMessages.SaveFailed);
        }

        Logger.LogInformation("{Entity} eliminado: {Id}", typeof(TEntity).Name, id);
        return Result.Success();
    }
}
