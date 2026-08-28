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

public class FacturaService : BaseCrudService<Factura, FacturaDto>, IFacturaService
{
    public FacturaService(IUnitOfWork uow, ILogger<FacturaService> logger, IValidator<FacturaDto> validator)
        : base(uow, logger, validator)
    {
    }

    public async Task<IEnumerable<FacturaDto>> GetAllAsync()
    {
        var entities = await Uow.Facturas.GetAllWithClienteAsync();
        return entities.Adapt<IEnumerable<FacturaDto>>();
    }

    public async Task<FacturaDto?> GetByIdAsync(Guid id)
    {
        var entity = await Uow.Facturas.GetByIdWithClienteAsync(id);
        return entity?.Adapt<FacturaDto>();
    }

    public override async Task<Result> CreateAsync(FacturaDto dto)
    {
        dto.Total = dto.Subtotal + dto.Iva;
        return await base.CreateAsync(dto);
    }

    public override async Task<Result> UpdateAsync(FacturaDto dto)
    {
        dto.Total = dto.Subtotal + dto.Iva;
        return await base.UpdateAsync(dto);
    }
}
