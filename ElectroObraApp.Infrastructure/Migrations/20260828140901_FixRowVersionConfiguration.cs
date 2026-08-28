using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ElectroObraApp.Infrastructure.Migrations
{
    /// <inheritdoc />
    public partial class FixRowVersionConfiguration : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Trabajos",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "TiposMovimiento",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "OrdenTrabajoItems",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "OrdenesTrabajo",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Movimientos",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Liquidaciones",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Facturas",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Empleados",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Clientes",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "ClienteContactos",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Categorias",
                type: "BLOB",
                maxLength: 8,
                nullable: false,
                defaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 },
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldRowVersion: true);

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000001"),
                column: "RowVersion",
                value: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000002"),
                column: "RowVersion",
                value: new byte[] { 0, 0, 0, 0, 0, 0, 0, 2 });

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000003"),
                column: "RowVersion",
                value: new byte[] { 0, 0, 0, 0, 0, 0, 0, 3 });

            migrationBuilder.UpdateData(
                table: "TiposMovimiento",
                keyColumn: "Id",
                keyValue: new Guid("00000000-0000-0000-0000-000000000004"),
                column: "RowVersion",
                value: new byte[] { 0, 0, 0, 0, 0, 0, 0, 4 });
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Trabajos",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "TiposMovimiento",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "OrdenTrabajoItems",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "OrdenesTrabajo",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Movimientos",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Liquidaciones",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Facturas",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Empleados",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Clientes",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "ClienteContactos",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });

            migrationBuilder.AlterColumn<byte[]>(
                name: "RowVersion",
                table: "Categorias",
                type: "BLOB",
                rowVersion: true,
                nullable: false,
                oldClrType: typeof(byte[]),
                oldType: "BLOB",
                oldMaxLength: 8,
                oldDefaultValue: new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 });
        }
    }
}
