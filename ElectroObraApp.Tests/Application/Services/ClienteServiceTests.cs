using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FluentAssertions;
using Microsoft.Extensions.Logging;
using NSubstitute;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Services;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Interfaces;
using Xunit;

namespace ElectroObraApp.Tests.Application.Services;

public class ClienteServiceTests
{
    private readonly IUnitOfWork _uow;
    private readonly IRepository<Cliente> _repo;
    private readonly IClienteRepository _clienteRepo;
    private readonly ILogger<ClienteService> _logger;
    private readonly ClienteService _service;

    public ClienteServiceTests()
    {
        _uow = Substitute.For<IUnitOfWork>();
        _repo = Substitute.For<IRepository<Cliente>>();
        _clienteRepo = Substitute.For<IClienteRepository>();
        _uow.Repository<Cliente>().Returns(_repo);
        _uow.Clientes.Returns(_clienteRepo);
        _logger = Substitute.For<ILogger<ClienteService>>();
        _service = new ClienteService(_uow, _logger, new ElectroObraApp.Application.Validators.ClienteValidator());
    }

    [Fact]
    public async Task GetAllAsync_ShouldReturnList()
    {
        // Arrange
        var list = new List<Cliente> { new() { Nombre = "Juan" } };
        _clienteRepo.GetAllWithContactosAsync().Returns(list);

        // Act
        var result = await _service.GetAllAsync();

        // Assert
        result.Should().HaveCount(1);
        result.First().Nombre.Should().Be("Juan");
    }

    [Fact]
    public async Task GetByIdAsync_WithContacts_ShouldMapCorrectly()
    {
        // Arrange
        var id = Guid.NewGuid();
        var cliente = new Cliente 
        { 
            Id = id, 
            Nombre = "Empresa X",
            Contactos = new List<ClienteContacto> 
            { 
                new() { Etiqueta = "Admin", Email = "admin@x.com" },
                new() { Etiqueta = "Ventas", Email = "ventas@x.com" }
            }
        };
        _clienteRepo.GetByIdWithContactosAsync(id).Returns(cliente);

        // Act
        var result = await _service.GetByIdAsync(id);

        // Assert
        result.Should().NotBeNull();
        result!.Contactos.Should().HaveCount(2);
        result.Contactos[0].Etiqueta.Should().Be("Admin");
        result.Contactos[1].Email.Should().Be("ventas@x.com");
    }
}

