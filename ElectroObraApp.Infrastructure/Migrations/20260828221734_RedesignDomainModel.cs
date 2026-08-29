using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ElectroObraApp.Infrastructure.Migrations
{
    /// <inheritdoc />
    public partial class RedesignDomainModel : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "Obras",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    Numero = table.Column<int>(type: "INTEGER", nullable: false),
                    Nombre = table.Column<string>(type: "TEXT", maxLength: 200, nullable: false),
                    Direccion = table.Column<string>(type: "TEXT", maxLength: 500, nullable: true),
                    Localidad = table.Column<string>(type: "TEXT", maxLength: 200, nullable: true),
                    ClienteId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Estado = table.Column<int>(type: "INTEGER", nullable: false),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    UpdatedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    RowVersion = table.Column<byte[]>(type: "BLOB", maxLength: 8, nullable: false, defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 }),
                    IsDeleted = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    DeletedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Obras", x => x.Id);
                    table.ForeignKey(
                        name: "FK_Obras_Clientes_ClienteId",
                        column: x => x.ClienteId,
                        principalTable: "Clientes",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                });

            migrationBuilder.CreateIndex(
                name: "IX_Obras_ClienteId",
                table: "Obras",
                column: "ClienteId");

            migrationBuilder.CreateIndex(
                name: "IX_Obras_Numero",
                table: "Obras",
                column: "Numero",
                unique: true);

            migrationBuilder.Sql("""
                INSERT INTO Obras (Id, Numero, Nombre, Direccion, ClienteId, Estado, CreatedAt, RowVersion, IsDeleted)
                SELECT
                    t.Id,
                    (SELECT COUNT(*) FROM Trabajos t2 WHERE t2.CreatedAt <= t.CreatedAt),
                    COALESCE(NULLIF(t.Descripcion, ''), 'Obra'),
                    c.Direccion,
                    t.ClienteId,
                    CASE WHEN t.Finalizado = 1 THEN 3 ELSE 0 END,
                    t.CreatedAt,
                    COALESCE(t.RowVersion, X'0000000000000001'),
                    COALESCE(t.IsDeleted, 0)
                FROM Trabajos t
                INNER JOIN Clientes c ON c.Id = t.ClienteId;
                """);

            migrationBuilder.DropForeignKey(
                name: "FK_Trabajos_Clientes_ClienteId",
                table: "Trabajos");

            migrationBuilder.RenameColumn(
                name: "ClienteId",
                table: "Trabajos",
                newName: "ObraId");

            migrationBuilder.RenameIndex(
                name: "IX_Trabajos_ClienteId",
                table: "Trabajos",
                newName: "IX_Trabajos_ObraId");

            migrationBuilder.Sql("""
                UPDATE Trabajos SET ObraId = Id;
                """);

            migrationBuilder.AddColumn<int>(
                name: "Estado",
                table: "Trabajos",
                type: "INTEGER",
                nullable: false,
                defaultValue: 1);

            migrationBuilder.Sql("""
                UPDATE Trabajos
                SET Estado = CASE WHEN Finalizado = 1 THEN 3 ELSE 1 END;
                """);

            migrationBuilder.DropColumn(
                name: "Finalizado",
                table: "Trabajos");

            migrationBuilder.AddColumn<bool>(
                name: "Ejecutado",
                table: "OrdenTrabajoItems",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<string>(
                name: "Nota",
                table: "OrdenTrabajoItems",
                type: "TEXT",
                maxLength: 1000,
                nullable: true);

            migrationBuilder.AddColumn<long>(
                name: "CotizacionAplicada",
                table: "Movimientos",
                type: "INTEGER",
                nullable: true);

            migrationBuilder.AddColumn<Guid>(
                name: "TipoConceptoPagoId",
                table: "Movimientos",
                type: "TEXT",
                nullable: true);

            migrationBuilder.CreateTable(
                name: "Adjuntos",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    EntidadTipo = table.Column<string>(type: "TEXT", maxLength: 50, nullable: false),
                    EntidadId = table.Column<Guid>(type: "TEXT", nullable: false),
                    NombreArchivo = table.Column<string>(type: "TEXT", maxLength: 255, nullable: false),
                    RutaRelativa = table.Column<string>(type: "TEXT", maxLength: 500, nullable: false),
                    Mime = table.Column<string>(type: "TEXT", maxLength: 100, nullable: false),
                    Tamano = table.Column<long>(type: "INTEGER", nullable: false),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    UpdatedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    RowVersion = table.Column<byte[]>(type: "BLOB", maxLength: 8, nullable: false, defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 }),
                    IsDeleted = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    DeletedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Adjuntos", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "AsistenciasEmpleado",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    EmpleadoId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Fecha = table.Column<DateTime>(type: "TEXT", nullable: false),
                    TipoJornada = table.Column<int>(type: "INTEGER", nullable: false),
                    TrabajoId = table.Column<Guid>(type: "TEXT", nullable: true),
                    Observaciones = table.Column<string>(type: "TEXT", maxLength: 1000, nullable: true),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    UpdatedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    RowVersion = table.Column<byte[]>(type: "BLOB", maxLength: 8, nullable: false, defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 }),
                    IsDeleted = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    DeletedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_AsistenciasEmpleado", x => x.Id);
                    table.ForeignKey(
                        name: "FK_AsistenciasEmpleado_Empleados_EmpleadoId",
                        column: x => x.EmpleadoId,
                        principalTable: "Empleados",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "FK_AsistenciasEmpleado_Trabajos_TrabajoId",
                        column: x => x.TrabajoId,
                        principalTable: "Trabajos",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.SetNull);
                });

            migrationBuilder.CreateTable(
                name: "PagosFactura",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    FacturaId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Fecha = table.Column<DateTime>(type: "TEXT", nullable: false),
                    Monto = table.Column<long>(type: "INTEGER", nullable: false),
                    MedioPago = table.Column<string>(type: "TEXT", maxLength: 100, nullable: false),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    UpdatedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    RowVersion = table.Column<byte[]>(type: "BLOB", maxLength: 8, nullable: false, defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 }),
                    IsDeleted = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    DeletedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_PagosFactura", x => x.Id);
                    table.ForeignKey(
                        name: "FK_PagosFactura_Facturas_FacturaId",
                        column: x => x.FacturaId,
                        principalTable: "Facturas",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "TiposConceptoPago",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    Nombre = table.Column<string>(type: "TEXT", maxLength: 100, nullable: false),
                    EsSistema = table.Column<bool>(type: "INTEGER", nullable: false),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    UpdatedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    RowVersion = table.Column<byte[]>(type: "BLOB", maxLength: 8, nullable: false, defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 }),
                    IsDeleted = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    DeletedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_TiposConceptoPago", x => x.Id);
                });

            migrationBuilder.Sql("""
                INSERT INTO PagosFactura (Id, FacturaId, Fecha, Monto, MedioPago, CreatedAt, RowVersion, IsDeleted)
                SELECT
                    lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
                    f.Id,
                    f.Fecha,
                    f.Total,
                    'Migracion',
                    datetime('now'),
                    X'0000000000000001',
                    0
                FROM Facturas f
                WHERE f.Estado = 3;
                """);

            migrationBuilder.Sql("""
                INSERT INTO AsistenciasEmpleado (Id, EmpleadoId, Fecha, TipoJornada, CreatedAt, RowVersion, IsDeleted)
                SELECT
                    lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
                    l.EmpleadoId,
                    date(l.FechaInicio, '+' || CAST(d.value AS TEXT) || ' days'),
                    0,
                    datetime('now'),
                    X'0000000000000001',
                    0
                FROM Liquidaciones l
                CROSS JOIN (
                    SELECT 0 AS value UNION SELECT 1 UNION SELECT 2 UNION SELECT 3 UNION SELECT 4
                    UNION SELECT 5 UNION SELECT 6 UNION SELECT 7 UNION SELECT 8 UNION SELECT 9
                    UNION SELECT 10 UNION SELECT 11 UNION SELECT 12 UNION SELECT 13 UNION SELECT 14
                    UNION SELECT 15 UNION SELECT 16 UNION SELECT 17 UNION SELECT 18 UNION SELECT 19
                    UNION SELECT 20 UNION SELECT 21 UNION SELECT 22 UNION SELECT 23 UNION SELECT 24
                    UNION SELECT 25 UNION SELECT 26 UNION SELECT 27 UNION SELECT 28 UNION SELECT 29
                    UNION SELECT 30
                ) d
                WHERE d.value < l.DiasTrabajados / 10000;
                """);

            migrationBuilder.CreateIndex(
                name: "IX_Movimientos_TipoConceptoPagoId",
                table: "Movimientos",
                column: "TipoConceptoPagoId");

            migrationBuilder.CreateIndex(
                name: "IX_Adjuntos_EntidadTipo_EntidadId",
                table: "Adjuntos",
                columns: new[] { "EntidadTipo", "EntidadId" });

            migrationBuilder.CreateIndex(
                name: "IX_AsistenciasEmpleado_EmpleadoId_Fecha",
                table: "AsistenciasEmpleado",
                columns: new[] { "EmpleadoId", "Fecha" },
                unique: true);

            migrationBuilder.CreateIndex(
                name: "IX_AsistenciasEmpleado_TrabajoId",
                table: "AsistenciasEmpleado",
                column: "TrabajoId");

            migrationBuilder.CreateIndex(
                name: "IX_PagosFactura_FacturaId",
                table: "PagosFactura",
                column: "FacturaId");

            migrationBuilder.CreateIndex(
                name: "IX_PagosFactura_Fecha",
                table: "PagosFactura",
                column: "Fecha");

            migrationBuilder.AddForeignKey(
                name: "FK_Movimientos_TiposConceptoPago_TipoConceptoPagoId",
                table: "Movimientos",
                column: "TipoConceptoPagoId",
                principalTable: "TiposConceptoPago",
                principalColumn: "Id",
                onDelete: ReferentialAction.SetNull);

            migrationBuilder.AddForeignKey(
                name: "FK_Trabajos_Obras_ObraId",
                table: "Trabajos",
                column: "ObraId",
                principalTable: "Obras",
                principalColumn: "Id",
                onDelete: ReferentialAction.Restrict);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "FK_Movimientos_TiposConceptoPago_TipoConceptoPagoId",
                table: "Movimientos");

            migrationBuilder.DropForeignKey(
                name: "FK_Trabajos_Obras_ObraId",
                table: "Trabajos");

            migrationBuilder.DropTable(name: "Adjuntos");
            migrationBuilder.DropTable(name: "AsistenciasEmpleado");
            migrationBuilder.DropTable(name: "PagosFactura");
            migrationBuilder.DropTable(name: "TiposConceptoPago");
            migrationBuilder.DropTable(name: "Obras");

            migrationBuilder.DropIndex(
                name: "IX_Movimientos_TipoConceptoPagoId",
                table: "Movimientos");

            migrationBuilder.DropColumn(name: "Ejecutado", table: "OrdenTrabajoItems");
            migrationBuilder.DropColumn(name: "Nota", table: "OrdenTrabajoItems");
            migrationBuilder.DropColumn(name: "CotizacionAplicada", table: "Movimientos");
            migrationBuilder.DropColumn(name: "TipoConceptoPagoId", table: "Movimientos");
            migrationBuilder.DropColumn(name: "Estado", table: "Trabajos");

            migrationBuilder.RenameColumn(name: "ObraId", table: "Trabajos", newName: "ClienteId");
            migrationBuilder.RenameIndex(name: "IX_Trabajos_ObraId", table: "Trabajos", newName: "IX_Trabajos_ClienteId");

            migrationBuilder.AddColumn<bool>(
                name: "Finalizado",
                table: "Trabajos",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddForeignKey(
                name: "FK_Trabajos_Clientes_ClienteId",
                table: "Trabajos",
                column: "ClienteId",
                principalTable: "Clientes",
                principalColumn: "Id",
                onDelete: ReferentialAction.Restrict);
        }
    }
}
