using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using ElectroObraApp.Core;
using ElectroObraApp.Infrastructure.Data;
using ElectroObraApp.Infrastructure.Repositories;
using ElectroObraApp.Infrastructure.Services;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging.Abstractions;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Services;

public class AdjuntoServiceTests : IAsyncLifetime
{
    private readonly string _tempRoot = Path.Combine(Path.GetTempPath(), "ElectroObra_Adjuntos_" + Guid.NewGuid());
    private SqliteConnection _connection = null!;
    private ApplicationDbContext _context = null!;
    private UnitOfWork _unitOfWork = null!;
    private AdjuntoService _service = null!;

    public async ValueTask InitializeAsync()
    {
        Directory.CreateDirectory(_tempRoot);

        _connection = new SqliteConnection("DataSource=:memory:");
        await _connection.OpenAsync();

        var options = new DbContextOptionsBuilder<ApplicationDbContext>()
            .UseSqlite(_connection)
            .Options;

        _context = new ApplicationDbContext(options);
        await _context.Database.EnsureCreatedAsync();

        _unitOfWork = new UnitOfWork(_context);
        _service = new AdjuntoService(_unitOfWork, NullLogger<AdjuntoService>.Instance, _tempRoot);
    }

    public async ValueTask DisposeAsync()
    {
        _unitOfWork.Dispose();
        await _context.DisposeAsync();
        await _connection.DisposeAsync();

        if (Directory.Exists(_tempRoot))
            Directory.Delete(_tempRoot, recursive: true);
    }

    [Fact]
    public async Task AddGetAndDelete_ShouldManageAttachmentLifecycle()
    {
        var entidadId = Guid.NewGuid();
        var sourceFile = Path.Combine(_tempRoot, "origen.txt");
        await File.WriteAllTextAsync(sourceFile, "contenido de prueba");

        var added = await _service.AddFromFileAsync(Constants.EntidadesAdjunto.Movimiento, entidadId, sourceFile);

        added.NombreArchivo.Should().Be("origen.txt");
        added.EntidadTipo.Should().Be(Constants.EntidadesAdjunto.Movimiento);
        added.EntidadId.Should().Be(entidadId);
        added.Tamano.Should().BeGreaterThan(0);

        var storedPath = Path.Combine(_tempRoot, added.RutaRelativa.Replace('/', Path.DirectorySeparatorChar));
        File.Exists(storedPath).Should().BeTrue();

        var listed = await _service.GetByEntidadAsync(Constants.EntidadesAdjunto.Movimiento, entidadId);
        listed.Should().ContainSingle(x => x.Id == added.Id && x.NombreArchivo == "origen.txt");

        await _service.DeleteAsync(added.Id);

        File.Exists(storedPath).Should().BeFalse();

        var afterDelete = await _service.GetByEntidadAsync(Constants.EntidadesAdjunto.Movimiento, entidadId);
        afterDelete.Should().BeEmpty();
    }

    [Fact]
    public async Task GetByEntidadAsync_ShouldReturnOnlyMatchingEntityAttachments()
    {
        var entidadA = Guid.NewGuid();
        var otherEntityId = Guid.NewGuid();

        var sourceA = Path.Combine(_tempRoot, "a.txt");
        var sourceB = Path.Combine(_tempRoot, "b.txt");
        await File.WriteAllTextAsync(sourceA, "A");
        await File.WriteAllTextAsync(sourceB, "B");

        await _service.AddFromFileAsync(Constants.EntidadesAdjunto.Movimiento, entidadA, sourceA);
        await _service.AddFromFileAsync(Constants.EntidadesAdjunto.Movimiento, otherEntityId, sourceB);

        var result = await _service.GetByEntidadAsync(Constants.EntidadesAdjunto.Movimiento, entidadA);

        result.Should().ContainSingle();
        result.Single().NombreArchivo.Should().Be("a.txt");
    }
}
