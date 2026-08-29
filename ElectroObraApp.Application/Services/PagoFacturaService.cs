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

public class PagoFacturaService : BaseCrudService<PagoFactura, PagoFacturaDto>, IPagoFacturaService
{
    public PagoFacturaService(IUnitOfWork uow, ILogger<PagoFacturaService> logger, IValidator<PagoFacturaDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<PagoFacturaDto>> GetAllAsync()
    {
        var entities = await Repository.GetAllAsync();
        return entities.Adapt<IEnumerable<PagoFacturaDto>>();
    }

    public async Task<PagoFacturaDto?> GetByIdAsync(Guid id)
    {
        var entity = await Repository.GetByIdAsync(id);
        return entity?.Adapt<PagoFacturaDto>();
    }
}
