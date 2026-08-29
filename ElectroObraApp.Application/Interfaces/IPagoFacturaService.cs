using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface IPagoFacturaService
{
    Task<IEnumerable<PagoFacturaDto>> GetAllAsync();
    Task<PagoFacturaDto?> GetByIdAsync(Guid id);
    Task<Result> CreateAsync(PagoFacturaDto dto);
    Task<Result> UpdateAsync(PagoFacturaDto dto);
    Task<Result> DeleteAsync(Guid id);
}
