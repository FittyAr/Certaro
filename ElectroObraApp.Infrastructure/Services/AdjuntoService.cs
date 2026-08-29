using System.Diagnostics;
using System.Runtime.InteropServices;
using Mapster;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Helpers;
using ElectroObraApp.Core.Interfaces;

namespace ElectroObraApp.Infrastructure.Services;

public class AdjuntoService : IAdjuntoService
{
    private readonly IUnitOfWork _uow;
    private readonly ILogger<AdjuntoService> _logger;
    private readonly string _attachmentsRoot;

    public AdjuntoService(
        IUnitOfWork uow,
        ILogger<AdjuntoService> logger,
        string? attachmentsRoot = null)
    {
        _uow = uow;
        _logger = logger;
        _attachmentsRoot = attachmentsRoot ?? Path.Combine(PathHelper.GetAppDataPath(), "attachments");
    }

    public async Task<IReadOnlyList<AdjuntoDto>> GetByEntidadAsync(string entidadTipo, Guid entidadId)
    {
        var entities = await _uow.Repository<Adjunto>()
            .FindAsync(a => a.EntidadTipo == entidadTipo && a.EntidadId == entidadId);

        return entities
            .OrderBy(a => a.NombreArchivo, StringComparer.OrdinalIgnoreCase)
            .Adapt<List<AdjuntoDto>>();
    }

    public async Task<AdjuntoDto> AddFromFileAsync(string entidadTipo, Guid entidadId, string sourceFilePath)
    {
        if (string.IsNullOrWhiteSpace(entidadTipo))
            throw new ArgumentException("El tipo de entidad es obligatorio.", nameof(entidadTipo));

        if (entidadId == Guid.Empty)
            throw new ArgumentException("El identificador de entidad es obligatorio.", nameof(entidadId));

        if (string.IsNullOrWhiteSpace(sourceFilePath) || !File.Exists(sourceFilePath))
            throw new FileNotFoundException("No se encontró el archivo de origen.", sourceFilePath);

        var fileName = SanitizeFileName(Path.GetFileName(sourceFilePath));
        var fileId = Guid.NewGuid();
        var storedFileName = $"{fileId}_{fileName}";
        var relativePath = Path.Combine(entidadTipo, entidadId.ToString(), storedFileName);
        var destinationDirectory = Path.Combine(_attachmentsRoot, entidadTipo, entidadId.ToString());
        var destinationPath = Path.Combine(destinationDirectory, storedFileName);

        Directory.CreateDirectory(destinationDirectory);
        File.Copy(sourceFilePath, destinationPath, overwrite: false);

        var fileInfo = new FileInfo(destinationPath);
        var entity = new Adjunto
        {
            Id = fileId,
            EntidadTipo = entidadTipo,
            EntidadId = entidadId,
            NombreArchivo = fileName,
            RutaRelativa = relativePath.Replace('\\', '/'),
            Mime = GetMimeType(fileName),
            Tamano = fileInfo.Length
        };

        await _uow.Repository<Adjunto>().AddAsync(entity);
        await _uow.SaveChangesAsync();

        _logger.LogInformation(
            "Adjunto agregado {AdjuntoId} para {EntidadTipo}/{EntidadId}: {NombreArchivo}",
            entity.Id,
            entidadTipo,
            entidadId,
            fileName);

        return entity.Adapt<AdjuntoDto>();
    }

    public async Task DeleteAsync(Guid id)
    {
        var repo = _uow.Repository<Adjunto>();
        var entity = await repo.GetByIdAsync(id)
            ?? throw new InvalidOperationException($"Adjunto no encontrado: {id}");

        var fullPath = GetFullPath(entity.RutaRelativa);
        if (File.Exists(fullPath))
        {
            File.Delete(fullPath);
        }

        entity.IsDeleted = true;
        entity.DeletedAt = DateTime.UtcNow;
        entity.UpdatedAt = DateTime.UtcNow;
        repo.Update(entity);
        await _uow.SaveChangesAsync();

        _logger.LogInformation("Adjunto eliminado {AdjuntoId}", id);
    }

    public async Task OpenAsync(Guid id)
    {
        var entity = await _uow.Repository<Adjunto>().GetByIdAsync(id)
            ?? throw new InvalidOperationException($"Adjunto no encontrado: {id}");

        var fullPath = GetFullPath(entity.RutaRelativa);
        if (!File.Exists(fullPath))
            throw new FileNotFoundException("No se encontró el archivo adjunto en disco.", fullPath);

        OpenFile(fullPath);
    }

    private string GetFullPath(string relativePath) =>
        Path.Combine(_attachmentsRoot, relativePath.Replace('/', Path.DirectorySeparatorChar));

    private static string SanitizeFileName(string fileName)
    {
        var invalidChars = Path.GetInvalidFileNameChars();
        var sanitized = new string(fileName
            .Where(c => !invalidChars.Contains(c))
            .ToArray());

        return string.IsNullOrWhiteSpace(sanitized) ? "archivo" : sanitized;
    }

    private static string GetMimeType(string fileName)
    {
        return Path.GetExtension(fileName).ToLowerInvariant() switch
        {
            ".pdf" => "application/pdf",
            ".jpg" or ".jpeg" => "image/jpeg",
            ".png" => "image/png",
            ".gif" => "image/gif",
            ".webp" => "image/webp",
            ".txt" => "text/plain",
            ".csv" => "text/csv",
            ".doc" => "application/msword",
            ".docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ".xls" => "application/vnd.ms-excel",
            ".xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            _ => "application/octet-stream"
        };
    }

    private static void OpenFile(string filePath)
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            Process.Start(new ProcessStartInfo(filePath) { UseShellExecute = true });
            return;
        }

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            Process.Start("xdg-open", filePath);
            return;
        }

        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            Process.Start("open", filePath);
        }
    }
}
