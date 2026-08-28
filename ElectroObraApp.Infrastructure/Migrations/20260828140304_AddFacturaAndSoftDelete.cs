using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ElectroObraApp.Infrastructure.Migrations
{
    /// <inheritdoc />
    public partial class AddFacturaAndSoftDelete : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "FK_Trabajos_Clientes_ClienteId",
                table: "Trabajos");

            migrationBuilder.AlterColumn<long>(
                name: "Presupuesto",
                table: "Trabajos",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "Trabajos",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "Trabajos",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "Trabajos",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "TiposMovimiento",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "TiposMovimiento",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "TiposMovimiento",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AlterColumn<long>(
                name: "PrecioUnitario",
                table: "OrdenTrabajoItems",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "PorcentajeAnterior",
                table: "OrdenTrabajoItems",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "PorcentajeActual",
                table: "OrdenTrabajoItems",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "Cantidad",
                table: "OrdenTrabajoItems",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "OrdenTrabajoItems",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "OrdenTrabajoItems",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "OrdenTrabajoItems",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AlterColumn<long>(
                name: "OtrosDescuentos",
                table: "OrdenesTrabajo",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "AjusteUocraPorcentaje",
                table: "OrdenesTrabajo",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "OrdenesTrabajo",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "OrdenesTrabajo",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "OrdenesTrabajo",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AlterColumn<long>(
                name: "Monto",
                table: "Movimientos",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "Cantidad",
                table: "Movimientos",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "Movimientos",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "Movimientos",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "Movimientos",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AlterColumn<long>(
                name: "TotalBruto",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "TotalAdelantos",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "TarifaAplicada",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "MultiplicadorSabado",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "MultiplicadorFeriado",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "MultiplicadorDomingo",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "DiasTrabajados",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "Liquidaciones",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "Liquidaciones",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AlterColumn<long>(
                name: "TarifaDiaria",
                table: "Empleados",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<long>(
                name: "SueldoBase",
                table: "Empleados",
                type: "INTEGER",
                nullable: false,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "Empleados",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "Empleados",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "Empleados",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "Clientes",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "Clientes",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "Clientes",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "ClienteContactos",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "ClienteContactos",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "ClienteContactos",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.AddColumn<DateTime>(
                name: "DeletedAt",
                table: "Categorias",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<bool>(
                name: "IsDeleted",
                table: "Categorias",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<byte[]>(
                name: "RowVersion",
                table: "Categorias",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                defaultValue: new byte[0]);

            migrationBuilder.CreateTable(
                name: "Facturas",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    Numero = table.Column<string>(type: "TEXT", maxLength: 50, nullable: false),
                    Fecha = table.Column<DateTime>(type: "TEXT", nullable: false),
                    ClienteId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Estado = table.Column<int>(type: "INTEGER", nullable: false),
                    Subtotal = table.Column<long>(type: "INTEGER", nullable: false),
                    Iva = table.Column<long>(type: "INTEGER", nullable: false),
                    Total = table.Column<long>(type: "INTEGER", nullable: false),
                    Observaciones = table.Column<string>(type: "TEXT", maxLength: 1000, nullable: true),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    UpdatedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    RowVersion = table.Column<byte[]>(type: "BLOB", rowVersion: true, nullable: false),
                    IsDeleted = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    DeletedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Facturas", x => x.Id);
                    table.ForeignKey(
                        name: "FK_Facturas_Clientes_ClienteId",
                        column: x => x.ClienteId,
                        principalTable: "Clientes",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Restrict);
                });

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000001"),
                column: "DeletedAt",
                value: null);

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000002"),
                column: "DeletedAt",
                value: null);

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000003"),
                column: "DeletedAt",
                value: null);

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000004"),
                column: "DeletedAt",
                value: null);

            migrationBuilder.CreateIndex(
                name: "IX_Movimientos_FacturaId",
                table: "Movimientos",
                column: "FacturaId");

            migrationBuilder.CreateIndex(
                name: "IX_Movimientos_Fecha",
                table: "Movimientos",
                column: "Fecha");

            migrationBuilder.CreateIndex(
                name: "IX_Empleados_Dni",
                table: "Empleados",
                column: "Dni");

            migrationBuilder.CreateIndex(
                name: "IX_Clientes_Cuit",
                table: "Clientes",
                column: "Cuit");

            migrationBuilder.CreateIndex(
                name: "IX_Facturas_ClienteId",
                table: "Facturas",
                column: "ClienteId");

            migrationBuilder.CreateIndex(
                name: "IX_Facturas_Fecha",
                table: "Facturas",
                column: "Fecha");

            migrationBuilder.CreateIndex(
                name: "IX_Facturas_Numero",
                table: "Facturas",
                column: "Numero");

            migrationBuilder.AddForeignKey(
                name: "FK_Movimientos_Facturas_FacturaId",
                table: "Movimientos",
                column: "FacturaId",
                principalTable: "Facturas",
                principalColumn: "Id",
                onDelete: ReferentialAction.SetNull);

            migrationBuilder.AddForeignKey(
                name: "FK_Trabajos_Clientes_ClienteId",
                table: "Trabajos",
                column: "ClienteId",
                principalTable: "Clientes",
                principalColumn: "Id",
                onDelete: ReferentialAction.Restrict);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "FK_Movimientos_Facturas_FacturaId",
                table: "Movimientos");

            migrationBuilder.DropForeignKey(
                name: "FK_Trabajos_Clientes_ClienteId",
                table: "Trabajos");

            migrationBuilder.DropTable(
                name: "Facturas");

            migrationBuilder.DropIndex(
                name: "IX_Movimientos_FacturaId",
                table: "Movimientos");

            migrationBuilder.DropIndex(
                name: "IX_Movimientos_Fecha",
                table: "Movimientos");

            migrationBuilder.DropIndex(
                name: "IX_Empleados_Dni",
                table: "Empleados");

            migrationBuilder.DropIndex(
                name: "IX_Clientes_Cuit",
                table: "Clientes");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "Trabajos");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "Trabajos");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "Trabajos");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "TiposMovimiento");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "TiposMovimiento");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "TiposMovimiento");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "OrdenTrabajoItems");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "OrdenTrabajoItems");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "OrdenTrabajoItems");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "OrdenesTrabajo");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "OrdenesTrabajo");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "OrdenesTrabajo");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "Movimientos");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "Movimientos");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "Movimientos");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "Empleados");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "Empleados");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "Empleados");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "Clientes");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "Clientes");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "Clientes");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "ClienteContactos");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "ClienteContactos");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "ClienteContactos");

            migrationBuilder.DropColumn(
                name: "DeletedAt",
                table: "Categorias");

            migrationBuilder.DropColumn(
                name: "IsDeleted",
                table: "Categorias");

            migrationBuilder.DropColumn(
                name: "RowVersion",
                table: "Categorias");

            migrationBuilder.AlterColumn<double>(
                name: "Presupuesto",
                table: "Trabajos",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "PrecioUnitario",
                table: "OrdenTrabajoItems",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "PorcentajeAnterior",
                table: "OrdenTrabajoItems",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "PorcentajeActual",
                table: "OrdenTrabajoItems",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "Cantidad",
                table: "OrdenTrabajoItems",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "OtrosDescuentos",
                table: "OrdenesTrabajo",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "AjusteUocraPorcentaje",
                table: "OrdenesTrabajo",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "Monto",
                table: "Movimientos",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "Cantidad",
                table: "Movimientos",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "TotalBruto",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "TotalAdelantos",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "TarifaAplicada",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "MultiplicadorSabado",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "MultiplicadorFeriado",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "MultiplicadorDomingo",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "DiasTrabajados",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "TarifaDiaria",
                table: "Empleados",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AlterColumn<double>(
                name: "SueldoBase",
                table: "Empleados",
                type: "REAL",
                nullable: false,
                oldClrType: typeof(long),
                oldType: "INTEGER");

            migrationBuilder.AddForeignKey(
                name: "FK_Trabajos_Clientes_ClienteId",
                table: "Trabajos",
                column: "ClienteId",
                principalTable: "Clientes",
                principalColumn: "Id",
                onDelete: ReferentialAction.Cascade);
        }
    }
}
