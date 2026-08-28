using FluentAssertions;
using ElectroObraApp.Infrastructure.Data.Converters;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Data;

public class DecimalToLongConverterTests
{
    [Theory]
    [InlineData(0)]
    [InlineData(1.2345)]
    [InlineData(99999.9999)]
    [InlineData(-42.5678)]
    public void DecimalToLongConverter_ShouldRoundTripValues(decimal value)
    {
        var converter = new DecimalToLongConverter();
        var toProvider = converter.ConvertToProviderExpression.Compile();
        var fromProvider = converter.ConvertFromProviderExpression.Compile();

        var stored = (long)toProvider(value)!;
        var restored = (decimal)fromProvider(stored)!;

        restored.Should().Be(value);
    }

    [Fact]
    public void DecimalToLongConverter_ShouldUseExpectedScale()
    {
        DecimalToLongConverter.Scale.Should().Be(10_000);
    }

    [Fact]
    public void NullableDecimalToLongConverter_ShouldHandleNull()
    {
        var converter = new NullableDecimalToLongConverter();
        var toProvider = converter.ConvertToProviderExpression.Compile();
        var fromProvider = converter.ConvertFromProviderExpression.Compile();

        ((long?)toProvider(null)).Should().BeNull();
        ((decimal?)fromProvider(null)).Should().BeNull();
    }

    [Fact]
    public void NullableDecimalToLongConverter_ShouldRoundTripNonNullValues()
    {
        var converter = new NullableDecimalToLongConverter();
        var toProvider = converter.ConvertToProviderExpression.Compile();
        var fromProvider = converter.ConvertFromProviderExpression.Compile();

        decimal? input = 150.25m;
        var stored = (long?)toProvider(input);
        var restored = (decimal?)fromProvider(stored);

        restored.Should().Be(150.25m);
    }
}
