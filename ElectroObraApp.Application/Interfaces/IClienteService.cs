using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface IClienteService
{
    Task<IEnumerable<ClienteDto>> GetAllAsync();
    Task<ClienteDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(ClienteDto dto);
    Task<Result> UpdateAsync(ClienteDto dto);
    Task<Result> DeleteAsync(Guid id);
}
