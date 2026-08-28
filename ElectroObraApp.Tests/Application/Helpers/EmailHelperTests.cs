using FluentAssertions;
using ElectroObraApp.Application.Helpers;
using ElectroObraApp.Infrastructure.Services;
using Xunit;

namespace ElectroObraApp.Tests.Application.Helpers;

public class EmailHelperTests
{
    [Fact]
    public void BuildMailtoUrl_ShouldEncodeSubjectAndBody()
    {
        var url = EmailHelper.BuildMailtoUrl("test@example.com", "Hola & Adiós", "Línea 1\nLínea 2");

        url.Should().StartWith("mailto:test@example.com?");
        url.Should().Contain("subject=Hola%20%26%20Adi%C3%B3s");
        url.Should().Contain("body=L%C3%ADnea%201%0AL%C3%ADnea%202");
    }
}

public class ExportCsvEscapeTests
{
    [Fact]
    public void EscapeCsv_ShouldQuoteValuesWithCommaOrQuotes()
    {
        ExportService.EscapeCsv("normal").Should().Be("normal");
        ExportService.EscapeCsv("a,b").Should().Be("\"a,b\"");
        ExportService.EscapeCsv("say \"hi\"").Should().Be("\"say \"\"hi\"\"\"");
    }
}
