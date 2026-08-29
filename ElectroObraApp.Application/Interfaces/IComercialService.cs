using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;

namespace ElectroObraApp.Application.Interfaces;

public interface IComercialService
{
    Task<CuentaCorrienteClienteDto> GetCuentaCorrienteClienteAsync(Guid clienteId);

    Task<AntiguedadDeudaDto> GetAntiguedadDeudaAsync(Guid? clienteId = null);

    Task<IReadOnlyList<RentabilidadObraDto>> GetRentabilidadPorObraAsync();
}
