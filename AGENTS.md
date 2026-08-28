# AGENTS.md - Plan de Implementación "ElectroObraApp"

Este documento detalla la hoja de ruta para el desarrollo de **ElectroObraApp**, un sistema de gestión operativa profesional.

## 🚀 Objetivo
Construir un sistema robusto, escalable y mantenible para el control operativo y flujo de caja, utilizando las mejores prácticas de ingeniería de software.

## 🏗️ Arquitectura y Patrones de Diseño

### Arquitectura Limpia (Clean Architecture)
El proyecto se divide en las siguientes capas para asegurar el desacoplamiento:

1.  **Core (Dominio)**:
    *   **Entidades**: Modelos puros del negocio (Movimiento, Cliente, Trabajo, etc.).
    *   **Enums**: Tipos definidos (TipoMovimiento, EstadoFactura, etc.).
    *   **Interfaces de Repositorio**: Definiciones de cómo acceder a los datos.
    *   **Especificaciones**: Lógica de consulta compleja.

2.  **Application (Aplicación)**:
    *   **DTOs**: Objetos de transferencia de datos para desacoplar la UI del dominio.
    *   **Interfaces de Servicios**: Definiciones de lógica de negocio.
    *   **Implementación de Servicios**: Orquestación de casos de uso.
    *   **Mappings**: Conversión entre Entidades y DTOs (usando Mapster).
    *   **Validaciones**: Reglas de negocio aplicadas a DTOs (usando FluentValidation).

3.  **Infrastructure (Infraestructura)**:
    *   **Persistencia**: Implementación de `DbContext` de EF Core y Repositorios.
    *   **SQLite**: Configuración y migraciones.
    *   **Servicios Externos**: Implementación de exportación (PDF/Excel), Logging, y acceso a archivos.

4.  **UI (Avalonia)**:
    *   **MVVM**: Uso estricto de Model-View-ViewModel con `CommunityToolkit.Mvvm`.
    *   **Views**: Definiciones XAML multiplataforma.
    *   **ViewModels**: Lógica de presentación y comandos.
    *   **Localización**: Gestión de textos mediante archivos JSON.

### Patrones de Diseño
*   **Repository & Unit of Work**: Para abstraer la persistencia y manejar transacciones.
*   **Dependency Injection**: Inyección de dependencias nativa de .NET.
*   **Options Pattern**: Para configuración tipada desde archivos JSON.
*   **Observer Pattern**: Implementado a través de `ObservableObject` y `Messenger` de CommunityToolkit.

## 🛠️ Estándares de Implementación

### 1. Cero Hardcoding
*   **Configuración**: Todo valor variable (cadenas de conexión, rutas, flags) irá en `appsettings.json`.
*   **Textos (i18n)**: Los textos de la UI se cargarán desde `i18n/es.json`.
*   **Constantes**: Valores fijos de negocio se definirán en clases de constantes dedicadas.

### 2. Calidad, Testing y Logging
*   **Unit Testing**: Cada objeto (Servicio, ViewModel, Helper) DEBE tener su correspondiente proyecto de test unitario usando `xUnit.v3`.
*   **Logging (Auditoría)**: Todo proceso de negocio, error o cambio de estado DEBE ser registrado usando `Serilog`.
    *   **Fase 1**: Logs en archivos `.log` locales (rotativos).
    *   **Fase Final**: Migración a base de datos para auditoría centralizada (`SerilogDbSink` preparado).
    *   **Entornos**: Diferenciación de logs para Desarrollo y Producción.
*   **Mocking**: Uso de `NSubstitute` para aislar dependencias en tests.

### 3. Librerías a Utilizar
*   **UI**: Avalonia 12.0+
*   **ORM**: EF Core 10+ (SQLite)
*   **Validación**: FluentValidation
*   **Mapping**: Mapster
*   **Logging**: Serilog (con enrichers MachineName/ThreadId)
*   **Reportes**: QuestPDF & ClosedXML
*   **Iconos**: Material.Icons.Avalonia (Sustituto compatible para Avalonia 12)
*   **Gráficos**: LiveChartsCore.SkiaSharpView.Avalonia (Dashboard)
*   **Notificaciones**: Avalonia.Labs.Notifications
*   **Testing**: xUnit.v3 & FluentAssertions

## 📅 Fases de Implementación

### Fase 1: Cimientos y Configuración
- [x] Configuración de `Directory.Packages.props`.
- [x] Creación de proyectos `ElectroObraApp.Core`, `ElectroObraApp.Application`, `ElectroObraApp.Infrastructure` y `ElectroObraApp.Tests`.
- [x] Implementación del sistema de Localización (JSON).
- [x] Configuración de Serilog y DI.

### Fase 2: Dominio y Persistencia (Movimientos)
- [x] Entidades de Movimiento, Categoría y Tipos de Movimiento (Tabla).
- [x] DbContext y Migraciones iniciales.
- [x] Repositorios Base y Unit of Work.

### Fase 3: Lógica de Aplicación
- [x] DTOs y Mappings.
- [x] Servicios de Movimientos con validación y Logging.
- [x] Tests Unitarios de servicios (Migrados a xUnit.v3).

### Fase 4: UI Base
- [x] MainView con navegación lateral funcional.
- [x] Listado de Movimientos con DataGrid.
- [x] Formulario de Alta/Edición de Movimientos con soporte para `DateTimeOffset`.

### Fase 5: Módulos Avanzados
- [x] Clientes y Facturación (CRUD completo).
- [x] Empleados y Liquidaciones (CRUD completo).
- [x] Gestión de Trabajos y Rentabilidad (CRUD completo).

### Fase 6: Reportes, DevOps y Pulido
- [x] Exportación básica a PDF y Excel (Movimientos).
- [x] Dashboard de estadísticas.
- [x] Optimización de UI y UX (Estabilización Avalonia 12).
- [x] DevOps: `.editorconfig`, `Directory.Build.props`, CI/CD (build-test + release), Dependabot.
- [x] Serilog centralizado con enrichers y stub de sink DB.
- [x] Configuración por entorno (`appsettings.Development.json` / `appsettings.Production.json`).

### Fase 7: Documentación
- [x] Licencia BSL corregida (`LICENSE`).
- [x] README (EN/ES), AGENTS.md y especificaciones técnicas actualizadas.

---

## 📝 Hitos y Decisiones Técnicas Recientes

### Migración a Avalonia 12 (Mayo 2026)
*   **Motivo**: Mantener el proyecto con las últimas capacidades de performance y estabilidad del framework.
*   **Desafíos**: Se detectó una incompatibilidad crítica con `FluentIcons.Avalonia` que causaba `MissingMethodException`. Se decidió remover la librería y simplificar los controles de iconos hasta encontrar un sustituto compatible.
*   **Ajustes**: Se actualizó `SkiaSharp` a versiones preview compatibles y se corrigieron cambios en las APIs de XAML (ej: `Watermark` -> `PlaceholderText`).

### Estabilización de Tipos de Datos (Fechas)
*   **Problema**: Error de casting entre `System.DateTime` y `System.DateTimeOffset?` en los controles `DatePicker` de Avalonia.
*   **Solución**: Implementación de propiedades proxy (`FechaOffset`) en los ViewModels para manejar la conversión sin ensuciar las entidades de dominio que prefieren `DateTime`.

### Migración decimal en SQLite
*   **Problema**: SQLite no soporta `decimal` nativamente.
*   **Solución**: Conversión EF Core a `long` mediante `DecimalToLongConverter` / `NullableDecimalToLongConverter` en `ApplicationDbContext`, preservando precisión en el dominio.

### Migración a xUnit.v3
*   **Cambio**: Actualización de la suite de pruebas a la versión 3 para aprovechar las mejoras en el runner y soporte moderno de .NET.
*   **Estado**: **174 tests unitarios** pasan correctamente con xUnit.v3, incluyendo servicios HTTP mock (`DollarService`, `HolidayService`) y ViewModels de configuración.

### DevOps (Agosto 2026)
*   **CI**: GitHub Actions (`build-test.yml`) compila Desktop, ejecuta tests y recolecta cobertura.
*   **Release**: Workflow manual (`release.yml`) ejecuta `build_installer.ps1`.
*   **Versionado**: `Directory.Build.props` lee la versión desde el archivo `VERSION`.

---
**Nota**: Cada paso de la implementación será validado con sus respectivos tests antes de pasar al siguiente.
