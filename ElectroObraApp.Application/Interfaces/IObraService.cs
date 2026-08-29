using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface IObraService
{
    Task<IEnumerable<ObraDto>> GetAllAsync();
    Task<ObraDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(ObraDto dto);
    Task<Result> UpdateAsync(ObraDto dto);
    Task<Result> DeleteAsync(Guid id);
}
