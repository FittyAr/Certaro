using System;
using System.Collections.Generic;
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

public class DollarServiceTests
{
    private readonly IUserSettingsService _settingsService;

    public DollarServiceTests()
    {
        _settingsService = Substitute.For<IUserSettingsService>();
        _settingsService.GetDollarApiUrl().Returns("https://example.com/dolares");
    }

    [Fact]
    public async Task GetDollarRatesAsync_ShouldReturnRates_WhenApiRespondsSuccessfully()
    {
        // Arrange
        const string json = """
            [
              {"nombre":"Oficial","compra":950.5,"venta":990.5,"casa":"BNA","fechaActualizacion":"2026-08-28T10:00:00"}
            ]
            """;
        var handler = new FakeHttpMessageHandler(_ => new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        });
        var service = new DollarService(new HttpClient(handler), _settingsService);

        // Act
        var result = await service.GetDollarRatesAsync();

        // Assert
        result.Should().HaveCount(1);
        result[0].Nombre.Should().Be("Oficial");
        result[0].Compra.Should().Be(950.5m);
        result[0].Venta.Should().Be(990.5m);
    }

    [Theory]
    [InlineData(HttpStatusCode.InternalServerError)]
    [InlineData(HttpStatusCode.NotFound)]
    [InlineData(HttpStatusCode.BadGateway)]
    public async Task GetDollarRatesAsync_ShouldReturnEmptyList_WhenApiFails(HttpStatusCode statusCode)
    {
        // Arrange
        var handler = new FakeHttpMessageHandler(_ => new HttpResponseMessage(statusCode));
        var service = new DollarService(new HttpClient(handler), _settingsService);

        // Act
        var result = await service.GetDollarRatesAsync();

        // Assert
        result.Should().BeEmpty();
    }

    [Theory]
    [InlineData("https://api.example.com/v1/dolares")]
    [InlineData("https://dolarapi.com/v1/dolares/blue")]
    public async Task GetDollarRatesAsync_ShouldCallConfiguredUrl(string apiUrl)
    {
        // Arrange
        _settingsService.GetDollarApiUrl().Returns(apiUrl);
        string? requestedUrl = null;
        var handler = new FakeHttpMessageHandler(request =>
        {
            requestedUrl = request.RequestUri?.ToString();
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("[]", Encoding.UTF8, "application/json")
            });
        });
        var service = new DollarService(new HttpClient(handler), _settingsService);

        // Act
        await service.GetDollarRatesAsync();

        // Assert
        requestedUrl.Should().Be(apiUrl);
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
