# ElectroObraApp

[![Version](https://img.shields.io/badge/version-1.1.4-blue.svg)](VERSION)
[![.NET Version](https://img.shields.io/badge/.NET-10.0-blueviolet.svg)](https://dotnet.microsoft.com/en-us/)
[![Avalonia UI](https://img.shields.io/badge/Avalonia-12.0%2B-crimson.svg)](https://avaloniaui.net/)
[![Database](https://img.shields.io/badge/SQLite-3.x-green.svg)](https://www.sqlite.org/)
[![License](https://img.shields.io/badge/license-BSL--1.1-orange.svg)](LICENSE)

> **Read in other languages:**
> [Leer en Español :spain:](README.es.md)

**ElectroObraApp** is a professional, multiplatform operational management and cash flow control system. It is designed to simplify and streamline the daily operations of small companies in the maintenance and construction sectors.

Instead of focusing on complex fiscal accounting, ElectroObraApp addresses the raw **operational control and real cash flow** of your business, offering flexibility, stability, and clean engineering.

---

## 🚀 Project Goal & Philosophy

*   **Real Cash Flow Focus:** Track physical money inflows and outflows, monotributo, insurance, tools, and materials.
*   **Minimalist & Premium Design:** A clean, modern, and interactive user interface built with Avalonia UI.
*   **Real-World Flexibility:** Contemplates everyday scenarios such as partial payments, employee salary advances, unjustified absences, and client debts.
*   **Multiplatform Support:** Designed to run seamlessly across Desktop (Windows, Linux, macOS), Mobile (Android, iOS), and Web (Browser) from a single shared C# codebase.

---

## 🏗️ Software Architecture & Design Patterns

The project is built following the principles of **Clean Architecture** to ensure decoupling, ease of testing, and long-term maintainability.

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

### 1. Layers Description
*   **Core (Domain):** Pure business logic, including Domain Entities, Enums, Specifications, and Repository Interfaces.
*   **Application:** Use cases implementation, DTOs, Service Interfaces, Mapping configurations (via Mapster), and validation rules (via FluentValidation).
*   **Infrastructure:** Database persistence (EF Core 10 with SQLite), file system management, external exports (PDF/Excel), and Logging (Serilog).
*   **UI (Avalonia):** Cross-platform user interface using the strict Model-View-ViewModel (MVVM) pattern with `CommunityToolkit.Mvvm` and localization support.

### 2. Applied Design Patterns
*   **Repository & Unit of Work:** Abstracts database persistence and ensures transactional operations.
*   **Dependency Injection (DI):** Native .NET dependency container integration.
*   **Options Pattern:** For strongly-typed configuration loading from `appsettings.json`.
*   **Observer Pattern:** Implemented using `ObservableObject` and `Messenger` from CommunityToolkit.

---

## 🛠️ Tech Stack & Key Libraries

*   **Language & Runtime:** C# (latest) / .NET 10.0 (pinned via `global.json`)
*   **UI Framework:** Avalonia UI 12.0+ (using `Material.Icons.Avalonia` and `LiveChartsCore`)
*   **ORM:** Entity Framework Core 10.0+ (SQLite provider)
*   **Validation:** FluentValidation
*   **Object Mapping:** Mapster
*   **Logging:** Serilog (rotating local files, console, MachineName/ThreadId enrichers)
*   **Reporting:** QuestPDF (PDF generation) and ClosedXML (Excel spreadsheets)
*   **Testing:** xUnit.v3 (174 tests), FluentAssertions, and NSubstitute

---

## 📦 Project Structure

```
ElectroObra/
├── .github/workflows/                 # CI/CD (build-test, release)
├── Docs/                              # Non-technical and technical documentation
├── ElectroObraApp/                    # Shared UI views and resources (Avalonia)
├── ElectroObraApp.Android/            # Android target project
├── ElectroObraApp.Application/        # Application services, DTOs, and mappings
├── ElectroObraApp.Browser/            # Web Browser target project (WebAssembly)
├── ElectroObraApp.Core/               # Domain entities and core interfaces
├── ElectroObraApp.Desktop/            # Desktop target launcher (Windows, macOS, Linux)
├── ElectroObraApp.Infrastructure/     # EF Core DbContext, Repositories, and Exports
├── ElectroObraApp.iOS/                # iOS target project
├── ElectroObraApp.Tests/              # Unit testing suite (xUnit.v3, 174 tests)
├── Directory.Build.props              # Shared MSBuild properties (version from VERSION)
├── Directory.Packages.props           # Central Package Management configuration
├── Directory.Build.props              # Version sync from VERSION file
├── global.json                        # .NET SDK pin (10.0.400+)
├── LICENSE                            # License information (BSL 1.1)
└── VERSION                            # Current application version
```

---

## 📅 Roadmap & Progress

*   **[x] Phase 1: Foundations & Config** — Central package management, Clean Architecture setup, localization engine, and Serilog configuration.
*   **[x] Phase 2: Domain & Persistence** — Cash flow entities, SQLite DbContext, Repository Base, and migrations.
*   **[x] Phase 3: Application Logic** — DTOs, Mapster mappings, and business validation rules with extensive unit testing.
*   **[x] Phase 4: Base UI** — Main navigation layout, transactions DataGrid view, and forms with date proxy stabilization.
*   **[x] Phase 5: Advanced Modules** — Clients/Invoicing, Employees/Payroll, Jobs & Project Profitability (CRUD complete).
*   **[x] Phase 6: Reports, DevOps & Polish** — Dashboard, PDF/Excel exports, CI/CD, environment config, Serilog enrichers.
*   **[x] Phase 7: Documentation** — README, AGENTS.md, technical specs, and BSL license.

---

## ⚙️ Getting Started

### Prerequisites
*   [.NET 10 SDK](https://dotnet.microsoft.com/download) — version pinned in [`global.json`](global.json) (`10.0.400`, rollForward `latestFeature`)
*   An IDE of your choice: [JetBrains Rider](https://www.jetbrains.com/rider/), [Visual Studio 2022](https://visualstudio.microsoft.com/), or [VS Code](https://code.visualstudio.com/) (with C# Dev Kit).

### Optional workloads (mobile / browser targets)
Install only if you plan to build non-desktop targets:

```bash
# Android
dotnet workload install android

# iOS (macOS only)
dotnet workload install ios

# Browser (WebAssembly)
dotnet workload install wasm-tools
```

### Restore & Build
Restore NuGet packages and compile the solution:
```bash
dotnet restore
dotnet build ElectroObraApp.Desktop/ElectroObraApp.Desktop.csproj
```

### Run the Application (Desktop)
To launch the desktop application, run:
```bash
dotnet run --project ElectroObraApp.Desktop
```

For development seed data, set the environment before running:
```bash
# Windows PowerShell
$env:DOTNET_ENVIRONMENT = "Development"
dotnet run --project ElectroObraApp.Desktop

# Linux/macOS
DOTNET_ENVIRONMENT=Development dotnet run --project ElectroObraApp.Desktop
```

`appsettings.Development.json` enables `SeedEnabled=true`; production uses `SeedEnabled=false`.

### Run Unit Tests
To run the automated xUnit.v3 test suite (174 tests), execute:
```bash
dotnet test ElectroObraApp.Tests/ElectroObraApp.Tests.csproj
```

With code coverage:
```bash
dotnet test ElectroObraApp.Tests/ElectroObraApp.Tests.csproj --collect:"XPlat Code Coverage"
```

### Decimal storage in SQLite
Monetary fields use `decimal` in the domain. EF Core persists them as `long` via custom value converters (`DecimalToLongConverter`), replacing the legacy `double` mapping and avoiding floating-point rounding errors.

---

## 📄 License

This project is licensed under the **Business Source License 1.1 (BSL 1.1)**. 

*   **Licensor:** ElectroObraApp Project Owners (FittyAr)
*   **Change Date:** July 6, 2030
*   **Change License:** GNU General Public License v2.0 or later (GPL-2.0-or-later)

For details, please refer to the [LICENSE](LICENSE) file.
