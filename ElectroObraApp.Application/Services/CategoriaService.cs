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

public class CategoriaService : BaseCrudService<Categoria, CategoriaDto>, ICategoriaService
{
    public CategoriaService(IUnitOfWork uow, ILogger<CategoriaService> logger, IValidator<CategoriaDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<CategoriaDto>> GetAllAsync()
    {
        var entities = await Repository.GetAllAsync();
        return entities.Adapt<IEnumerable<CategoriaDto>>();
    }

    public async Task<CategoriaDto?> GetByIdAsync(Guid id)
    {
        var entity = await Repository.GetByIdAsync(id);
        return entity?.Adapt<CategoriaDto>();
    }
}
