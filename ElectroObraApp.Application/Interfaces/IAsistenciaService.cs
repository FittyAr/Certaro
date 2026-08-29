using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface IAsistenciaService
{
    Task<IEnumerable<AsistenciaEmpleadoDto>> GetAllAsync();
    Task<IEnumerable<AsistenciaEmpleadoDto>> GetByPeriodAsync(DateTime inicio, DateTime fin);
    Task<AsistenciaEmpleadoDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(AsistenciaEmpleadoDto dto);
    Task<Result> UpdateAsync(AsistenciaEmpleadoDto dto);
    Task<Result<AsistenciaEmpleadoDto>> UpsertAsync(AsistenciaEmpleadoDto dto);
    Task<Result> DeleteAsync(Guid id);
}
