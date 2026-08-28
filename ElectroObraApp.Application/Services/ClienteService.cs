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
using FluentValidation;

namespace ElectroObraApp.Application.Services;

public class ClienteService : BaseCrudService<Cliente, ClienteDto>, IClienteService
{
    public ClienteService(IUnitOfWork uow, ILogger<ClienteService> logger, IValidator<ClienteDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<ClienteDto>> GetAllAsync()
    {
        var entities = await Uow.Clientes.GetAllWithContactosAsync();
        return entities.Adapt<IEnumerable<ClienteDto>>();
    }

    public async Task<ClienteDto?> GetByIdAsync(Guid id)
    {
        var entity = await Uow.Clientes.GetByIdWithContactosAsync(id);
        return entity?.Adapt<ClienteDto>();
    }

    public override async Task<Result> UpdateAsync(ClienteDto dto)
    {
        var validation = await ValidateAsync(dto);
        if (!validation.IsSuccess)
            return validation;

        Logger.LogInformation("Actualizando cliente: {Id}", dto.Id);
        var entity = dto.Adapt<Cliente>();
        await Uow.Clientes.UpdateWithContactosAsync(entity);

        if (await Uow.SaveChangesAsync() <= 0)
            return Result.Failure(ValidationMessages.SaveFailed);

        return Result.Success();
    }
}
