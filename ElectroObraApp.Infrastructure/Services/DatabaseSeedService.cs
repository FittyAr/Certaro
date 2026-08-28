using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Enums;
using ElectroObraApp.Infrastructure.Data;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;

namespace ElectroObraApp.Infrastructure.Services;

public class DatabaseSeedService : IDatabaseSeedService
{
    private readonly ApplicationDbContext _context;
    private readonly IConfiguration _configuration;
    private readonly ILogger<DatabaseSeedService> _logger;
    private readonly Random _random = new();

    public DatabaseSeedService(ApplicationDbContext context, IConfiguration configuration, ILogger<DatabaseSeedService> logger)
    {
        _context = context;
        _configuration = configuration;
        _logger = logger;
    }

    public bool IsSeedEnabled()
    {
        return _configuration.GetValue<bool>("Application:SeedEnabled");
    }

    public async Task SeedAsync()
    {
        if (!IsSeedEnabled()) return;

        var poolPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data", "SeedData", "SeedPool.json");
        if (!File.Exists(poolPath))
        {
            poolPath = Path.Combine(Directory.GetCurrentDirectory(), "..", "ElectroObraApp.Infrastructure", "Data", "SeedData", "SeedPool.json");
        }

        var json = await File.ReadAllTextAsync(poolPath);
        var pool = JsonSerializer.Deserialize<SeedDataPool>(json, new JsonSerializerOptions { PropertyNameCaseInsensitive = true });

        if (pool == null) return;

        await using var transaction = await _context.Database.BeginTransactionAsync();
        try
        {
            _logger.LogInformation("Iniciando limpieza de tablas para sembrado...");
        
        // Limpiar tablas
        _context.Movimientos.RemoveRange(_context.Movimientos);
        _context.Trabajos.RemoveRange(_context.Trabajos);
        _context.Clientes.RemoveRange(_context.Clientes);
        _context.Empleados.RemoveRange(_context.Empleados);
        _context.Categorias.RemoveRange(_context.Categorias);
        
        // No borramos TiposMovimiento porque tienen datos de sistema en HasData
        // Pero si quisiéramos sobrescribir los que no son de sistema:
        var customTypes = _context.TiposMovimiento.Where(t => !t.EsSistema);
        _context.TiposMovimiento.RemoveRange(customTypes);

        await _context.SaveChangesAsync();
        _logger.LogInformation("Limpieza completada.");

        // 1. Sembrar Categorías
        _logger.LogInformation("Sembrando Categorías...");
        var categorias = pool.Categorias.Select(name => new Categoria { Nombre = name }).ToList();
        _context.Categorias.AddRange(categorias);

        // 2. Sembrar Tipos de Movimiento (adicionales)
        _logger.LogInformation("Sembrando Tipos de Movimiento...");
        var existingTypes = await _context.TiposMovimiento.ToListAsync();
        foreach (var typeName in pool.TiposMovimiento)
        {
            if (!existingTypes.Any(t => t.Nombre == typeName))
            {
                _context.TiposMovimiento.Add(new TipoMovimiento { Nombre = typeName, EsIngreso = _random.Next(2) == 0 });
            }
        }
        await _context.SaveChangesAsync();
        
        var allTipos = await _context.TiposMovimiento.ToListAsync();

        // 3. Sembrar Empleados
        _logger.LogInformation("Sembrando Empleados...");
        int empCount = _random.Next(5, 11);
        var empleados = pool.NombresEmpleados.OrderBy(x => _random.Next()).Take(empCount)
            .Select(name => new Empleado 
            { 
                Nombre = name, 
                Dni = $"{_random.Next(10, 50)}.{_random.Next(100, 999)}.{_random.Next(100, 999)}",
                Cargo = "Operario",
                SueldoBase = _random.Next(200000, 500000),
                TarifaDiaria = _random.Next(35000, 55000), // Nueva tarifa diaria
                FechaIngreso = DateTime.Now.AddDays(-_random.Next(30, 365 * 5)),
                Activo = true
            }).ToList();
        _context.Empleados.AddRange(empleados);

        // 4. Sembrar Clientes
        _logger.LogInformation("Sembrando Clientes...");
        int cliCount = _random.Next(5, 11);
        var clientes = pool.NombresClientes.OrderBy(x => _random.Next()).Take(cliCount)
            .Select(name => new Cliente 
            { 
                Nombre = name, 
                Cuit = $"20-{_random.Next(10000000, 99999999)}-{_random.Next(0, 9)}",
                Direccion = $"Calle Falsa {_random.Next(100, 5000)}",
                Email = $"{name.Replace(" ", ".").ToLower()}@gmail.com",
                Telefono = $"54911{_random.Next(10000000, 99999999)}",
                CondicionIva = "Consumidor Final",
                Contactos = new List<ClienteContacto>
                {
                    new() { Etiqueta = "Administración", Email = $"admin.{name.Replace(" ", "").ToLower()}@empresa.com" },
                    new() { Etiqueta = "Pagos", Email = $"pagos.{name.Replace(" ", "").ToLower()}@empresa.com" }
                }
            }).ToList();
        _context.Clientes.AddRange(clientes);

        await _context.SaveChangesAsync();

        // 5. Sembrar Trabajos y Movimientos
        _logger.LogInformation("Sembrando Trabajos y Movimientos (esto puede tardar unos segundos)...");
        var allMovimientos = new List<Movimiento>();
        var startYear = DateTime.Now.AddYears(-1);
        foreach (var cliente in clientes)
        {
            int workCount = _random.Next(10, 21);
            for (int i = 0; i < workCount; i++)
            {
                var workStartDate = startYear.AddDays(_random.Next(0, 365));
                var trabajo = new Trabajo
                {
                    Descripcion = $"{pool.DescripcionesTrabajos[_random.Next(pool.DescripcionesTrabajos.Count)]} #{i + 1}",
                    ClienteId = cliente.Id,
                    FechaInicio = workStartDate,
                    Presupuesto = _random.Next(50000, 500000),
                    Finalizado = workStartDate.AddDays(30) < DateTime.Now && _random.Next(10) > 2
                };
                if (trabajo.Finalizado) trabajo.FechaFin = trabajo.FechaInicio.AddDays(_random.Next(5, 45));
                
                // 5.1 Sembrar Ordenes de Trabajo (Certificados)
                int certCount = _random.Next(1, 4);
                for (int c = 0; c < certCount; c++)
                {
                    var orden = new OrdenTrabajo
                    {
                        Titulo = $"Certificado de Avance {c + 1}",
                        NumeroCertificado = (c + 1).ToString(),
                        Fecha = trabajo.FechaInicio.AddDays(30 * (c + 1)),
                        AjusteUocraPorcentaje = 8,
                        Items = new List<OrdenTrabajoItem>
                        {
                            new() { Descripcion = "Cableado baja tensión", Cantidad = 1000, Unidad = "MTS", PrecioUnitario = 2500, PorcentajeAnterior = c * 30, PorcentajeActual = 30 },
                            new() { Descripcion = "Instalación de luminarias", Cantidad = 50, Unidad = "U", PrecioUnitario = 15000, PorcentajeAnterior = c * 20, PorcentajeActual = 20 }
                        }
                    };
                    trabajo.OrdenesTrabajo.Add(orden);
                }

                _context.Trabajos.Add(trabajo);

                // 6. Sembrar Movimientos
                int movCount = _random.Next(15, 26);
                for (int j = 0; j < movCount; j++)
                {
                    var movDate = trabajo.FechaInicio.AddDays(_random.Next(0, 30));
                    if (movDate > DateTime.Now) movDate = DateTime.Now;

                    var mov = new Movimiento
                    {
                        Concepto = pool.ConceptosMovimientos[_random.Next(pool.ConceptosMovimientos.Count)],
                        Monto = _random.Next(1000, 20000),
                        Cantidad = _random.Next(1, 10),
                        Fecha = movDate,
                        TipoMovimientoId = allTipos[_random.Next(allTipos.Count)].Id,
                        CategoriaId = categorias[_random.Next(categorias.Count)].Id,
                        ClienteId = cliente.Id,
                        EmpleadoId = empleados[_random.Next(empleados.Count)].Id,
                        Trabajo = trabajo,
                        Moneda = Moneda.ARS
                    };

                    // Asegurar algunos adelantos para pruebas de liquidación (Tipo ID 3)
                    if (j % 5 == 0) 
                    {
                        mov.TipoMovimientoId = Guid.Parse("00000000-0000-0000-0000-000000000003");
                        mov.Concepto = "Adelanto quincenal";
                        mov.Monto = _random.Next(30000, 60000);
                        mov.Cantidad = 1;
                    }

                    allMovimientos.Add(mov);
                }
            }
        }

        _logger.LogInformation($"Insertando {allMovimientos.Count} movimientos...");
        _context.Movimientos.AddRange(allMovimientos);
        await _context.SaveChangesAsync();
        await transaction.CommitAsync();
        _logger.LogInformation("Sembrado de datos completado exitosamente.");
        }
        catch (Exception ex)
        {
            await transaction.RollbackAsync();
            _logger.LogError(ex, "Error durante el sembrado de datos. Se revirtió la transacción.");
            throw;
        }
    }
}

