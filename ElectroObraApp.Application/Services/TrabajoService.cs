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

public class TrabajoService : BaseCrudService<Trabajo, TrabajoDto>, ITrabajoService
{
    public TrabajoService(IUnitOfWork uow, ILogger<TrabajoService> logger, IValidator<TrabajoDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<TrabajoDto>> GetAllAsync()
    {
        var entities = await Uow.Trabajos.GetAllWithDeepLoadAsync();
        return entities.Adapt<IEnumerable<TrabajoDto>>();
    }

    public async Task<TrabajoDto?> GetByIdAsync(Guid id)
    {
        var entity = await Uow.Trabajos.GetByIdWithDeepLoadAsync(id);
        return entity?.Adapt<TrabajoDto>();
    }
}
