using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Mapster;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using FluentValidation;

namespace ElectroObraApp.Application.Services;

public class ObraService : BaseCrudService<Obra, ObraDto>, IObraService
{
    public ObraService(IUnitOfWork uow, ILogger<ObraService> logger, IValidator<ObraDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<ObraDto>> GetAllAsync()
    {
        var entities = await Uow.Obras.GetAllWithClienteAsync();
        return entities.Adapt<IEnumerable<ObraDto>>();
    }

    public async Task<ObraDto?> GetByIdAsync(Guid id)
    {
        var entity = await Uow.Obras.GetByIdWithClienteAsync(id);
        return entity?.Adapt<ObraDto>();
    }
}
