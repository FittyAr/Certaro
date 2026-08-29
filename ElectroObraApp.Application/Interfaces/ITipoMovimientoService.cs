using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface ITipoMovimientoService
{
    Task<IEnumerable<TipoMovimientoDto>> GetAllAsync();
    Task<TipoMovimientoDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(TipoMovimientoDto dto);
    Task<Result> UpdateAsync(TipoMovimientoDto dto);
    Task<Result> DeleteAsync(Guid id);
}
