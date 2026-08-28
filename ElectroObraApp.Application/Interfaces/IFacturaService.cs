using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface IFacturaService
{
    Task<IEnumerable<FacturaDto>> GetAllAsync();
    Task<FacturaDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(FacturaDto dto);
    Task<Result> UpdateAsync(FacturaDto dto);
    Task<Result> DeleteAsync(Guid id);
}
