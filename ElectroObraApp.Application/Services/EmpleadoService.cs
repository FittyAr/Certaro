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

public class EmpleadoService : BaseCrudService<Empleado, EmpleadoDto>, IEmpleadoService
{
    public EmpleadoService(IUnitOfWork uow, ILogger<EmpleadoService> logger, IValidator<EmpleadoDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<EmpleadoDto>> GetAllAsync()
    {
        var entities = await Repository.GetAllAsync();
        return entities.Adapt<IEnumerable<EmpleadoDto>>();
    }

    public async Task<EmpleadoDto?> GetByIdAsync(Guid id)
    {
        var entity = await Repository.GetByIdAsync(id);
        return entity?.Adapt<EmpleadoDto>();
    }
}
