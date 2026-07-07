# ElectroObraApp

[![Versión](https://img.shields.io/badge/versión-1.1.4-blue.svg)](VERSION)
[![Versión .NET](https://img.shields.io/badge/.NET-10.0-blueviolet.svg)](https://dotnet.microsoft.com/es-es/)
[![Avalonia UI](https://img.shields.io/badge/Avalonia-12.0%2B-crimson.svg)](https://avaloniaui.net/)
[![Base de Datos](https://img.shields.io/badge/SQLite-3.x-green.svg)](https://www.sqlite.org/)
[![Licencia](https://img.shields.io/badge/licencia-BSL--1.1-orange.svg)](LISENSE)

> **Leer en otros idiomas:**
> [Read in English :uk:](README.md)

**ElectroObraApp** es un sistema profesional de gestión operativa y control de flujo de caja multiplataforma. Está diseñado para simplificar y optimizar la administración diaria de pequeñas empresas dedicadas al sector del mantenimiento y la construcción.

En lugar de enfocarse en una contabilidad fiscal compleja, ElectroObraApp se orienta al **control operativo y al flujo de caja real** del negocio, ofreciendo flexibilidad, estabilidad y una arquitectura limpia y profesional.

---

## 🚀 Objetivo y Filosofía del Proyecto

*   **Enfoque en el Flujo de Caja Real:** Control físico de entradas y salidas de dinero, monotributo, seguros, insumos, herramientas y materiales.
*   **Diseño Minimalista y Premium:** Interfaz limpia, moderna y dinámica desarrollada con Avalonia UI.
*   **Flexibilidad en Escenarios Reales:** Contempla situaciones cotidianas de obras, como pagos parciales, adelantos quincenales a empleados, faltas injustificadas y saldos pendientes de clientes.
*   **Soporte Multiplataforma:** Diseñado para ejecutarse en Desktop (Windows, Linux, macOS), Móvil (Android, iOS) y Web (Navegador) a partir de una única base de código compartida en C#.

---

## 🏗️ Arquitectura de Software y Patrones de Diseño

El proyecto está construido siguiendo los principios de **Clean Architecture (Arquitectura Limpia)** para asegurar el desacoplamiento, la facilidad de pruebas unitarias y la mantenibilidad a largo plazo.

```mermaid
graph TD
    UI[ElectroObraApp.UI - Avalonia] --> Application[ElectroObraApp.Application]
    Infrastructure[ElectroObraApp.Infrastructure] --> Application
    Application --> Core[ElectroObraApp.Core]
    Infrastructure --> Core
    
    style Core fill:#f9f,stroke:#333,stroke-width:2px
    style Application fill:#bbf,stroke:#333,stroke-width:2px
    style Infrastructure fill:#ddf,stroke:#333,stroke-width:2px
    style UI fill:#dfd,stroke:#333,stroke-width:2px
```

### 1. Descripción de Capas
*   **Core (Dominio):** Lógica pura del negocio, incluyendo Entidades de Dominio ([Movement](file:///d:/GitHub/ElectroObra/ElectroObraApp.Core), [Client](file:///d:/GitHub/ElectroObra/ElectroObraApp.Core), [Job](file:///d:/GitHub/ElectroObra/ElectroObraApp.Core), [Employee](file:///d:/GitHub/ElectroObra/ElectroObraApp.Core)), Enums, Especificaciones e Interfaces de Repositorio.
*   **Application (Aplicación):** Implementación de casos de uso, DTOs (Objetos de Transferencia de Datos), Interfaces de Servicio, configuraciones de mapeo (con Mapster) y reglas de validación (con FluentValidation).
*   **Infrastructure (Infraestructura):** Persistencia en base de datos (EF Core con SQLite), gestión de archivos locales, exportaciones externas (PDF/Excel) e implementación de Logging.
*   **UI (Avalonia):** Interfaz de usuario multiplataforma siguiendo el patrón estricto Model-View-ViewModel (MVVM) con `CommunityToolkit.Mvvm` y soporte de localización (i18n).

### 2. Patrones de Diseño Aplicados
*   **Repository & Unit of Work:** Abstrae la persistencia de datos y garantiza transacciones atómicas.
*   **Dependency Injection (DI):** Integración nativa del contenedor de dependencias de .NET.
*   **Options Pattern:** Carga de configuración fuertemente tipada desde el archivo `appsettings.json`.
*   **Observer Pattern:** Implementado a través de `ObservableObject` y el sistema de mensajería `Messenger` de CommunityToolkit.

---

## 🛠️ Stack Tecnológico y Librerías Clave

*   **Lenguaje y Runtime:** C# 10 / .NET 10.0
*   **Framework de UI:** Avalonia UI 12.0+ (utilizando `Material.Icons.Avalonia` y `LiveChartsCore`)
*   **ORM:** Entity Framework Core 10.0+ (Proveedor SQLite)
*   **Validación:** FluentValidation
*   **Mapeo de Objetos:** Mapster
*   **Logging:** Serilog (configurado con archivos locales rotativos y salida en consola)
*   **Reportes:** QuestPDF (generación de PDF) y ClosedXML (planillas de Excel)
*   **Testing:** xUnit.v3, FluentAssertions y NSubstitute (para mocks en tests unitarios)

---

## 📦 Estructura del Proyecto

```
ElectroObra/
├── Docs/                              # Documentación técnica y funcional
├── ElectroObraApp/                    # Vistas y recursos compartidos de UI (Avalonia)
├── ElectroObraApp.Android/            # Proyecto específico para Android
├── ElectroObraApp.Application/        # Servicios de aplicación, DTOs y mappings
├── ElectroObraApp.Browser/            # Proyecto específico para Navegador (WebAssembly)
├── ElectroObraApp.Core/               # Entidades de dominio e interfaces core
├── ElectroObraApp.Desktop/            # Lanzador para Escritorio (Windows, macOS, Linux)
├── ElectroObraApp.Infrastructure/     # DbContext de EF Core, Repositorios y Exportación
├── ElectroObraApp.iOS/                # Proyecto específico para iOS
├── ElectroObraApp.Tests/              # Suite de pruebas unitarias (xUnit.v3)
├── Directory.Packages.props           # Configuración centralizada de paquetes NuGet
├── LISENSE                            # Información de licencia (BSL 1.1)
└── VERSION                            # Versión actual de la aplicación
```

---

## 📅 Roadmap e Implementación

*   **[x] Fase 1: Cimientos y Configuración** — Gestión centralizada de paquetes, estructura Clean Architecture, sistema de localización JSON y configuración de Serilog.
*   **[x] Fase 2: Dominio y Persistencia** — Entidades de flujo de caja, DbContext SQLite, repositorios base y migraciones iniciales.
*   **[x] Fase 3: Lógica de Aplicación** — DTOs, mapeos de Mapster y reglas de validación de negocio con cobertura de tests unitarios.
*   **[x] Fase 4: UI Base** — Maquetación de navegación principal, DataGrid de movimientos y formularios con fecha proxy estabilizada.
*   **[/] Fase 5: Módulos Avanzados** — Clientes y Facturación (CRUD completado), Empleados y Liquidación (CRUD completado), Trabajos y Rentabilidad de Obras (CRUD completado).
*   **[/] Fase 6: Reportes y Pulido** — Exportaciones básicas a PDF y Excel (Completadas), gráficos de Dashboard y optimización fina de UI en Avalonia 12.

---

## ⚙️ Primeros Pasos

### Requisitos Previos
*   [.NET 10 SDK](https://dotnet.microsoft.com/download)
*   Un IDE de desarrollo: [JetBrains Rider](https://www.jetbrains.com/rider/), [Visual Studio 2022](https://visualstudio.microsoft.com/) o [VS Code](https://code.visualstudio.com/) (con C# Dev Kit).

### Restaurar y Compilar
Restaura los paquetes NuGet y compila la solución completa:
```bash
dotnet restore
dotnet build
```

### Ejecutar la Aplicación (Escritorio)
Para iniciar la aplicación de escritorio:
```bash
dotnet run --project ElectroObraApp.Desktop
```

### Ejecutar Pruebas Unitarias
Para correr la suite de pruebas automatizadas con xUnit.v3:
```bash
dotnet test
```

---

## 📄 Licencia

Este proyecto está bajo la licencia **Business Source License 1.1 (BSL 1.1)**.

*   **Licenciante:** Flota-HAS Project Owners (FittyAr)
*   **Fecha de Cambio:** 6 de Julio de 2030
*   **Licencia de Cambio:** GNU General Public License v2.0 or later (GPL-2.0-or-later)

Para más detalles, consulta el archivo [LISENSE](file:///d:/GitHub/ElectroObra/LISENSE).
