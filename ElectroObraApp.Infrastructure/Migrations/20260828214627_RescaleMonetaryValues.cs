using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ElectroObraApp.Infrastructure.Migrations
{
    /// <inheritdoc />
    public partial class RescaleMonetaryValues : Migration
    {
        private const int Scale = 10_000;

        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.Sql("""
                INSERT OR IGNORE INTO AppMetadata (Key, Value, UpdatedAt)
                VALUES ('MonetaryValuesRescaled', 'false', datetime('now'));
                """);

            foreach (var (table, column) in MonetaryColumnRegistry.Columns)
            {
                migrationBuilder.Sql($"""
                    UPDATE "{table}"
                    SET "{column}" = "{column}" * {Scale}
                    WHERE EXISTS (
                        SELECT 1 FROM AppMetadata
                        WHERE Key = 'MonetaryValuesRescaled' AND Value = 'false'
                    )
                    AND "{column}" IS NOT NULL
                    AND "{column}" != 0;
                    """);
            }

            migrationBuilder.Sql("""
                UPDATE AppMetadata
                SET Value = 'true', UpdatedAt = datetime('now')
                WHERE Key = 'MonetaryValuesRescaled' AND Value = 'false';
                """);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.Sql("""
                UPDATE AppMetadata
                SET Value = 'false', UpdatedAt = datetime('now')
                WHERE Key = 'MonetaryValuesRescaled';
                """);

            foreach (var (table, column) in MonetaryColumnRegistry.Columns)
            {
                migrationBuilder.Sql($"""
                    UPDATE "{table}"
                    SET "{column}" = "{column}" / {Scale}
                    WHERE "{column}" IS NOT NULL
                    AND "{column}" != 0
                    AND ABS("{column}") >= {Scale};
                    """);
            }
        }
    }
}
