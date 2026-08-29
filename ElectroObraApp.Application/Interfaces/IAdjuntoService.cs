using ElectroObraApp.Application.DTOs;

namespace ElectroObraApp.Application.Interfaces;

public interface IAdjuntoService
{
    Task<IReadOnlyList<AdjuntoDto>> GetByEntidadAsync(string entidadTipo, Guid entidadId);
    Task<AdjuntoDto> AddFromFileAsync(string entidadTipo, Guid entidadId, string sourceFilePath);
    Task DeleteAsync(Guid id);
    Task OpenAsync(Guid id);
}
