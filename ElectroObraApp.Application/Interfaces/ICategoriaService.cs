using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface ICategoriaService
{
    Task<IEnumerable<CategoriaDto>> GetAllAsync();
    Task<CategoriaDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(CategoriaDto dto);
    Task<Result> UpdateAsync(CategoriaDto dto);
    Task<Result> DeleteAsync(Guid id);
}
