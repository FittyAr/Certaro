using Microsoft.EntityFrameworkCore;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using ElectroObraApp.Infrastructure.Data.Converters;
using System.Linq.Expressions;
using System.Reflection;

namespace ElectroObraApp.Infrastructure.Data;

public class ApplicationDbContext : DbContext
{
    private static readonly byte[] DefaultRowVersion = { 0, 0, 0, 0, 0, 0, 0, 1 };

    public ApplicationDbContext(DbContextOptions<ApplicationDbContext> options) : base(options)
    {
    }

    public DbSet<Movimiento> Movimientos { get; set; }
    public DbSet<TipoMovimiento> TiposMovimiento { get; set; }
    public DbSet<Categoria> Categorias { get; set; }
    public DbSet<Cliente> Clientes { get; set; }
    public DbSet<Empleado> Empleados { get; set; }
    public DbSet<Trabajo> Trabajos { get; set; }
    public DbSet<OrdenTrabajo> OrdenesTrabajo { get; set; }
    public DbSet<OrdenTrabajoItem> OrdenTrabajoItems { get; set; }
    public DbSet<Liquidacion> Liquidaciones { get; set; }
    public DbSet<ClienteContacto> ClienteContactos { get; set; }
    public DbSet<Factura> Facturas { get; set; }

    public override Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        var utcNow = DateTime.UtcNow;
        foreach (var entry in ChangeTracker.Entries<BaseEntity>())
        {
            if (entry.State == EntityState.Added)
            {
                if (entry.Entity.RowVersion.Length == 0)
                    entry.Entity.RowVersion = DefaultRowVersion;
            }
            else if (entry.State == EntityState.Modified)
            {
                entry.Entity.UpdatedAt = utcNow;
            }
        }

        return base.SaveChangesAsync(cancellationToken);
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        foreach (var entityType in modelBuilder.Model.GetEntityTypes())
        {
            foreach (var property in entityType.GetProperties())
            {
                if (property.ClrType == typeof(decimal))
                {
                    modelBuilder.Entity(entityType.ClrType)
                        .Property(property.Name)
                        .HasConversion(new DecimalToLongConverter());
                }
                else if (property.ClrType == typeof(decimal?))
                {
                    modelBuilder.Entity(entityType.ClrType)
                        .Property(property.Name)
                        .HasConversion(new NullableDecimalToLongConverter());
                }
            }

            if (typeof(BaseEntity).IsAssignableFrom(entityType.ClrType))
            {
                modelBuilder.Entity(entityType.ClrType)
                    .Property(nameof(BaseEntity.RowVersion))
                    .HasMaxLength(8)
                    .IsConcurrencyToken()
                    .HasDefaultValue(DefaultRowVersion);

                modelBuilder.Entity(entityType.ClrType)
                    .Property(nameof(BaseEntity.IsDeleted))
                    .HasDefaultValue(false);

                var method = typeof(ApplicationDbContext)
                    .GetMethod(nameof(SetSoftDeleteFilter), BindingFlags.NonPublic | BindingFlags.Static)!
                    .MakeGenericMethod(entityType.ClrType);
                method.Invoke(null, [modelBuilder]);
            }
        }

        modelBuilder.Entity<Cliente>(entity =>
        {
            entity.Property(x => x.Nombre).HasMaxLength(200);
            entity.Property(x => x.Cuit).HasMaxLength(13);
            entity.Property(x => x.Direccion).HasMaxLength(500);
            entity.Property(x => x.Email).HasMaxLength(254);
            entity.Property(x => x.Telefono).HasMaxLength(30);
            entity.Property(x => x.CondicionIva).HasMaxLength(100);
            entity.HasIndex(x => x.Cuit);
        });

        modelBuilder.Entity<Empleado>(entity =>
        {
            entity.Property(x => x.Nombre).HasMaxLength(200);
            entity.Property(x => x.Dni).HasMaxLength(15);
            entity.Property(x => x.Cargo).HasMaxLength(100);
            entity.Property(x => x.Email).HasMaxLength(254);
            entity.Property(x => x.Telefono).HasMaxLength(30);
            entity.HasIndex(x => x.Dni);
        });

        modelBuilder.Entity<Movimiento>(entity =>
        {
            entity.Property(x => x.Concepto).HasMaxLength(500);
            entity.HasIndex(x => x.Fecha);

            entity.HasOne(x => x.Categoria)
                .WithMany(x => x.Movimientos)
                .HasForeignKey(x => x.CategoriaId)
                .OnDelete(DeleteBehavior.Restrict);

            entity.HasOne(x => x.TipoMovimiento)
                .WithMany(x => x.Movimientos)
                .HasForeignKey(x => x.TipoMovimientoId)
                .OnDelete(DeleteBehavior.Restrict);

            entity.HasOne(x => x.Factura)
                .WithMany(x => x.Movimientos)
                .HasForeignKey(x => x.FacturaId)
                .OnDelete(DeleteBehavior.SetNull);
        });

        modelBuilder.Entity<Trabajo>(entity =>
        {
            entity.Property(x => x.Descripcion).HasMaxLength(500);

            entity.HasOne(x => x.Cliente)
                .WithMany()
                .HasForeignKey(x => x.ClienteId)
                .OnDelete(DeleteBehavior.Restrict);
        });

        modelBuilder.Entity<Categoria>(entity =>
        {
            entity.Property(x => x.Nombre).HasMaxLength(100);
            entity.Property(x => x.Descripcion).HasMaxLength(500);
            entity.Property(x => x.Icono).HasMaxLength(50);
            entity.Property(x => x.ColorHex).HasMaxLength(7);
        });

        modelBuilder.Entity<TipoMovimiento>(entity =>
        {
            entity.Property(x => x.Nombre).HasMaxLength(100);
        });

        modelBuilder.Entity<OrdenTrabajo>(entity =>
        {
            entity.Property(x => x.Titulo).HasMaxLength(200);
            entity.Property(x => x.NumeroCertificado).HasMaxLength(50);

            entity.HasOne(x => x.Trabajo)
                .WithMany(x => x.OrdenesTrabajo)
                .HasForeignKey(x => x.TrabajoId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<OrdenTrabajoItem>(entity =>
        {
            entity.Property(x => x.Descripcion).HasMaxLength(500);
            entity.Property(x => x.Unidad).HasMaxLength(20);

            entity.HasOne(x => x.OrdenTrabajo)
                .WithMany(x => x.Items)
                .HasForeignKey(x => x.OrdenTrabajoId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<Liquidacion>(entity =>
        {
            entity.Property(x => x.Observaciones).HasMaxLength(1000);

            entity.HasOne(x => x.Empleado)
                .WithMany(x => x.Liquidaciones)
                .HasForeignKey(x => x.EmpleadoId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<ClienteContacto>(entity =>
        {
            entity.Property(x => x.Etiqueta).HasMaxLength(100);
            entity.Property(x => x.Email).HasMaxLength(254);

            entity.HasOne(x => x.Cliente)
                .WithMany(x => x.Contactos)
                .HasForeignKey(x => x.ClienteId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<Factura>(entity =>
        {
            entity.Property(x => x.Numero).HasMaxLength(50);
            entity.Property(x => x.Observaciones).HasMaxLength(1000);
            entity.HasIndex(x => x.Numero);
            entity.HasIndex(x => x.Fecha);

            entity.HasOne(x => x.Cliente)
                .WithMany()
                .HasForeignKey(x => x.ClienteId)
                .OnDelete(DeleteBehavior.Restrict);
        });

        var systemDate = new DateTime(2026, 1, 1, 0, 0, 0, DateTimeKind.Utc);
        modelBuilder.Entity<TipoMovimiento>().HasData(
            new TipoMovimiento { Id = Guid.Parse("00000000-0000-0000-0000-000000000001"), Nombre = "Ingreso", EsIngreso = true, EsSistema = true, CreatedAt = systemDate, RowVersion = new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 }, IsDeleted = false },
            new TipoMovimiento { Id = Guid.Parse("00000000-0000-0000-0000-000000000002"), Nombre = "Gasto", EsIngreso = false, EsSistema = true, CreatedAt = systemDate, RowVersion = new byte[] { 0, 0, 0, 0, 0, 0, 0, 2 }, IsDeleted = false },
            new TipoMovimiento { Id = Guid.Parse("00000000-0000-0000-0000-000000000003"), Nombre = "Adelanto", EsIngreso = false, EsSistema = true, CreatedAt = systemDate, RowVersion = new byte[] { 0, 0, 0, 0, 0, 0, 0, 3 }, IsDeleted = false },
            new TipoMovimiento { Id = Guid.Parse("00000000-0000-0000-0000-000000000004"), Nombre = "Ajuste", EsIngreso = true, EsSistema = true, CreatedAt = systemDate, RowVersion = new byte[] { 0, 0, 0, 0, 0, 0, 0, 4 }, IsDeleted = false }
        );
    }

    private static void SetSoftDeleteFilter<TEntity>(ModelBuilder modelBuilder) where TEntity : BaseEntity
    {
        modelBuilder.Entity<TEntity>().HasQueryFilter(e => !e.IsDeleted);
    }
}
