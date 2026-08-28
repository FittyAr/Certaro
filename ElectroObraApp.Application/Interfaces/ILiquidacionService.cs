using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Core.Common;

namespace ElectroObraApp.Application.Interfaces;

public interface ILiquidacionService
{
    Task<IEnumerable<LiquidacionDto>> GetAllAsync();
    Task<LiquidacionDto?> GetByIdAsync(Guid id);
    Task<Result<LiquidacionDto>> CreateAsync(LiquidacionDto dto);
    Task<Result> UpdateAsync(LiquidacionDto dto);
    Task<Result> DeleteAsync(Guid id);
    
    /// <summary>
    /// Calcula una pre-liquidación basada en el empleado y rango de fechas.
    /// Busca adelantos registrados en Movimientos.
    /// </summary>
    Task<LiquidacionDto> SugerirLiquidacionAsync(Guid empleadoId, DateTime inicio, DateTime fin, decimal diasTrabajados);
}
