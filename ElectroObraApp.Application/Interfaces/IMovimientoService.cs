using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface IMovimientoService
{
    Task<IEnumerable<MovimientoDto>> GetAllAsync();
    Task<PagedResult<MovimientoDto>> GetPagedAsync(MovimientoFilterDto filter);
    Task<MovimientoDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(MovimientoDto dto);
    Task<Result> UpdateAsync(MovimientoDto dto);
    Task<Result> DeleteAsync(Guid id);
}
