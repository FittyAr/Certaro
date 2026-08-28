using System;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using NSubstitute;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Infrastructure.Services;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Services;

public class HolidayServiceTests
{
    private readonly IUserSettingsService _settingsService;

    public HolidayServiceTests()
    {
        _settingsService = Substitute.For<IUserSettingsService>();
        _settingsService.GetHolidayApiUrl().Returns("https://example.com/feriados/");
    }

    [Fact]
    public async Task GetHolidaysAsync_ShouldParseHolidays_WhenApiReturnsValidJson()
    {
        // Arrange
        const string json = """
            [
              {"fecha":"2026-01-01","nombre":"Año Nuevo"},
              {"fecha":"2026-05-25","nombre":"Día de la Revolución de Mayo"}
            ]
            """;
        var handler = new FakeHttpMessageHandler(_ => new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        });
        var service = new HolidayService(new HttpClient(handler), _settingsService);

        // Act
        var result = await service.GetHolidaysAsync(2026);

        // Assert
        result.Should().HaveCount(2);
        result[0].Date.Should().Be(new DateTime(2026, 1, 1));
        result[0].Name.Should().Be("Año Nuevo");
        result[1].Date.Should().Be(new DateTime(2026, 5, 25));
    }

    [Theory]
    [InlineData(2025)]
    [InlineData(2026)]
    [InlineData(2027)]
    public async Task GetHolidaysAsync_ShouldRequestYearInUrl(int year)
    {
        // Arrange
        string? requestedUrl = null;
        var handler = new FakeHttpMessageHandler(request =>
        {
            requestedUrl = request.RequestUri?.ToString();
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("[]", Encoding.UTF8, "application/json")
            });
        });
        var service = new HolidayService(new HttpClient(handler), _settingsService);

        // Act
        await service.GetHolidaysAsync(year);

        // Assert
        requestedUrl.Should().Be($"https://example.com/feriados/{year}");
    }

    [Fact]
    public async Task GetHolidaysAsync_ShouldReturnEmptyList_WhenApiUrlIsMissing()
    {
        // Arrange
        _settingsService.GetHolidayApiUrl().Returns(string.Empty);
        Func<HttpRequestMessage, Task<HttpResponseMessage>> handlerFunc = _ => Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent("[]", Encoding.UTF8, "application/json")
        });
        var handler = new FakeHttpMessageHandler(handlerFunc);
        var service = new HolidayService(new HttpClient(handler), _settingsService);

        // Act
        var result = await service.GetHolidaysAsync(2026);

        // Assert
        result.Should().BeEmpty();
    }

    [Fact]
    public async Task GetHolidaysAsync_ShouldReturnEmptyList_WhenApiFails()
    {
        // Arrange
        var handler = new FakeHttpMessageHandler(_ => new HttpResponseMessage(HttpStatusCode.ServiceUnavailable));
        var service = new HolidayService(new HttpClient(handler), _settingsService);

        // Act
        var result = await service.GetHolidaysAsync(2026);

        // Assert
        result.Should().BeEmpty();
    }

    private sealed class FakeHttpMessageHandler : HttpMessageHandler
    {
        private readonly Func<HttpRequestMessage, Task<HttpResponseMessage>> _handler;

        public FakeHttpMessageHandler(Func<HttpRequestMessage, HttpResponseMessage> handler)
            : this(request => Task.FromResult(handler(request)))
        {
        }

        public FakeHttpMessageHandler(Func<HttpRequestMessage, Task<HttpResponseMessage>> handler)
        {
            _handler = handler;
        }

        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
            => _handler(request);
    }
}
