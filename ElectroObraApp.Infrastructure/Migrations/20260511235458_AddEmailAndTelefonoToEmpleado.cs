using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ElectroObraApp.Infrastructure.Migrations
{
    /// <inheritdoc />
    public partial class AddEmailAndTelefonoToEmpleado : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "Email",
                table: "Empleados",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<string>(
                name: "Telefono",
                table: "Empleados",
                type: "TEXT",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "Email",
                table: "Empleados");

            migrationBuilder.DropColumn(
                name: "Telefono",
                table: "Empleados");
        }
    }
}
