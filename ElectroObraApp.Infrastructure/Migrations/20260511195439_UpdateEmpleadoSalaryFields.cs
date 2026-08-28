using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ElectroObraApp.Infrastructure.Migrations
{
    /// <inheritdoc />
    public partial class UpdateEmpleadoSalaryFields : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<bool>(
                name: "IncluirDomingos",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<bool>(
                name: "IncluirFeriados",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<bool>(
                name: "IncluirSabados",
                table: "Liquidaciones",
                type: "INTEGER",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<double>(
                name: "MultiplicadorDomingo",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0);

            migrationBuilder.AddColumn<double>(
                name: "MultiplicadorFeriado",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0);

            migrationBuilder.AddColumn<double>(
                name: "MultiplicadorSabado",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0);

            migrationBuilder.AddColumn<double>(
                name: "TotalBruto",
                table: "Liquidaciones",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0);

            migrationBuilder.AddColumn<int>(
                name: "PagoFrecuencia",
                table: "Empleados",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "IncluirDomingos",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "IncluirFeriados",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "IncluirSabados",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "MultiplicadorDomingo",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "MultiplicadorFeriado",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "MultiplicadorSabado",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "TotalBruto",
                table: "Liquidaciones");

            migrationBuilder.DropColumn(
                name: "PagoFrecuencia",
                table: "Empleados");
        }
    }
}
