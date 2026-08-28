using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

namespace ElectroObraApp.Infrastructure.Data.Converters;

/// <summary>
/// Stores decimal values as scaled long integers (4 decimal places) for SQLite precision.
/// </summary>
public class DecimalToLongConverter : ValueConverter<decimal, long>
{
    public const int Scale = 10_000;

    public DecimalToLongConverter()
        : base(
            v => (long)Math.Round(v * Scale, MidpointRounding.AwayFromZero),
            v => v / (decimal)Scale)
    {
    }
}

public class NullableDecimalToLongConverter : ValueConverter<decimal?, long?>
{
    public NullableDecimalToLongConverter()
        : base(
            v => v.HasValue ? (long?)Math.Round(v.Value * DecimalToLongConverter.Scale, MidpointRounding.AwayFromZero) : null,
            v => v.HasValue ? v.Value / (decimal)DecimalToLongConverter.Scale : null)
    {
    }
}
