using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface ITrabajoService
{
    Task<IEnumerable<TrabajoDto>> GetAllAsync();
    Task<TrabajoDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(TrabajoDto dto);
    Task<Result> UpdateAsync(TrabajoDto dto);
    Task<Result> DeleteAsync(Guid id);
}
