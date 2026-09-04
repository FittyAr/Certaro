# Reporte de Archivos Monolíticos y Plan de Segmentación

> **Fecha de generación:** 2026-09-04  
> **Criterio:** Archivos con >= 300 líneas  
> **Total detectados:** 66 (🔴 2 Críticos, 🟡 0 Altos, 🔵 64 Medios)

## 1. Resumen Ejecutivo

| Métrica | Valor |
| :--- | :--- |
| Archivos monolíticos | **66** |
| 🔴 Críticos (>= 800 líneas) | **2** |
| 🟡 Altos (500 - 799 líneas) | **0** |
| 🔵 Moderados (300 - 499 líneas) | **64** |

## 2. Inventario de Archivos Monolíticos

| # | Nivel | Archivo | Capa | Líneas Totales | Líneas Código | Tamaño (KB) | Sugerencia de Segmentación |
|---|---|---|---|---|---|---|---|
| 1 | 🔴 Crítico | `crates/certaro-infrastructure/tests/dashboard.rs` | Backend (Rust) | 874 | 766 | 25.9 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 2 | 🔴 Crítico | `crates/certaro-import-legacy/tests/import.rs` | Backend (Rust) | 846 | 690 | 34.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 3 | 🔵 Medio | `src/views/calendario/CalendarioView.vue` | Frontend (Vue/TS) | 499 | 440 | 17.5 KB | Dividir vista en subcomponentes modulares (tablas, modales, panels) |
| 4 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/certificado.rs` | Backend (Rust) | 499 | 440 | 17.4 KB | Separar traits/puertos por agregados o entidades de dominio |
| 5 | 🔵 Medio | `crates/certaro-migration/src/m20260101_000001_create_schema.rs` | Backend (Rust) | 499 | 465 | 23.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 6 | 🔵 Medio | `crates/certaro-application/src/use_cases/liquidaciones/calculation.rs` | Backend (Rust) | 495 | 426 | 17.6 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 7 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/liquidacion.rs` | Backend (Rust) | 467 | 409 | 17.1 KB | Separar traits/puertos por agregados o entidades de dominio |
| 8 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/pdf/certificado.rs` | Backend (Rust) | 464 | 399 | 16.5 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 9 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/pdf/table.rs` | Backend (Rust) | 464 | 383 | 13.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 10 | 🔵 Medio | `src/views/ordenes/OrdenesView.vue` | Frontend (Vue/TS) | 458 | 411 | 15.6 KB | Extraer subcomponentes secundarios y modularizar template |
| 11 | 🔵 Medio | `src/views/proyectos/ProyectoDetalleView.vue` | Frontend (Vue/TS) | 450 | 406 | 17.3 KB | Extraer subcomponentes secundarios y modularizar template |
| 12 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/orden_trabajo.rs` | Backend (Rust) | 450 | 403 | 16.0 KB | Separar traits/puertos por agregados o entidades de dominio |
| 13 | 🔵 Medio | `src/views/liquidaciones/components/LiquidacionWizardModal.vue` | Frontend (Vue/TS) | 447 | 404 | 14.8 KB | Extraer subcomponentes secundarios y modularizar template |
| 14 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/pdf/canvas/mod.rs` | Backend (Rust) | 446 | 382 | 14.4 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 15 | 🔵 Medio | `src/views/asistencia/AsistenciaView.vue` | Frontend (Vue/TS) | 441 | 396 | 15.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 16 | 🔵 Medio | `src/views/facturas/FacturasView.vue` | Frontend (Vue/TS) | 441 | 408 | 15.6 KB | Extraer subcomponentes secundarios y modularizar template |
| 17 | 🔵 Medio | `crates/certaro-infrastructure/src/backup/json.rs` | Backend (Rust) | 440 | 371 | 14.3 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 18 | 🔵 Medio | `src/stores/useKanbanStore.ts` | Frontend (Vue/TS) | 439 | 402 | 12.5 KB | Agrupar funciones en módulos utilitarios cohesivos |
| 19 | 🔵 Medio | `src/components/domain/ProyectosTreeTable.vue` | Frontend (Vue/TS) | 438 | 400 | 16.5 KB | Extraer subcomponentes secundarios y modularizar template |
| 20 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/cliente.rs` | Backend (Rust) | 438 | 387 | 15.3 KB | Separar traits/puertos por agregados o entidades de dominio |
| 21 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/proyecto.rs` | Backend (Rust) | 431 | 373 | 15.3 KB | Separar traits/puertos por agregados o entidades de dominio |
| 22 | 🔵 Medio | `src/views/comercial/CuentaCorrienteView.vue` | Frontend (Vue/TS) | 419 | 374 | 15.7 KB | Extraer subcomponentes secundarios y modularizar template |
| 23 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/pdf/liquidacion/sections.rs` | Backend (Rust) | 414 | 367 | 12.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 24 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/calendario.rs` | Backend (Rust) | 401 | 332 | 13.7 KB | Separar traits/puertos por agregados o entidades de dominio |
| 25 | 🔵 Medio | `crates/certaro-infrastructure/tests/backup.rs` | Backend (Rust) | 400 | 318 | 13.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 26 | 🔵 Medio | `crates/certaro-application/src/config/sections.rs` | Backend (Rust) | 391 | 338 | 12.3 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 27 | 🔵 Medio | `crates/certaro-application/src/use_cases/dashboard.rs` | Backend (Rust) | 391 | 331 | 15.9 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 28 | 🔵 Medio | `src/views/movimientos/components/MovimientoDrawer.vue` | Frontend (Vue/TS) | 377 | 343 | 12.1 KB | Extraer subcomponentes secundarios y modularizar template |
| 29 | 🔵 Medio | `src/views/clientes/ClientesView.vue` | Frontend (Vue/TS) | 373 | 334 | 13.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 30 | 🔵 Medio | `crates/certaro-application/src/use_cases/calendario/eventos.rs` | Backend (Rust) | 373 | 321 | 13.1 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 31 | 🔵 Medio | `src-tauri/src/lib.rs` | Backend (Rust) | 369 | 326 | 16.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 32 | 🔵 Medio | `src/views/movimientos/MovimientosView.vue` | Frontend (Vue/TS) | 367 | 335 | 13.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 33 | 🔵 Medio | `src/views/dashboard/DashboardView.vue` | Frontend (Vue/TS) | 366 | 315 | 14.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 34 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/categoria.rs` | Backend (Rust) | 360 | 315 | 12.0 KB | Separar traits/puertos por agregados o entidades de dominio |
| 35 | 🔵 Medio | `src/views/trabajos/TrabajosView.vue` | Frontend (Vue/TS) | 359 | 324 | 12.6 KB | Extraer subcomponentes secundarios y modularizar template |
| 36 | 🔵 Medio | `crates/certaro-application/src/use_cases/liquidaciones/mod.rs` | Backend (Rust) | 357 | 300 | 12.7 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 37 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/unit_of_work.rs` | Backend (Rust) | 355 | 302 | 12.8 KB | Modularizar datos y queries por entidad de base de datos |
| 38 | 🔵 Medio | `crates/certaro-application/src/use_cases/asistencias.rs` | Backend (Rust) | 354 | 302 | 13.0 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 39 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/dashboard/analytics.rs` | Backend (Rust) | 352 | 322 | 11.8 KB | Separar traits/puertos por agregados o entidades de dominio |
| 40 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/xlsx.rs` | Backend (Rust) | 344 | 288 | 12.5 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 41 | 🔵 Medio | `crates/certaro-application/src/validation/clientes.rs` | Backend (Rust) | 343 | 304 | 10.0 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 42 | 🔵 Medio | `crates/certaro-infrastructure/src/backup/service.rs` | Backend (Rust) | 343 | 273 | 12.1 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 43 | 🔵 Medio | `src/views/empleados/EmpleadosView.vue` | Frontend (Vue/TS) | 339 | 304 | 12.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 44 | 🔵 Medio | `src/views/WelcomeView.vue` | Frontend (Vue/TS) | 339 | 293 | 11.8 KB | Extraer subcomponentes secundarios y modularizar template |
| 45 | 🔵 Medio | `src/views/admin/UsuariosView.vue` | Frontend (Vue/TS) | 338 | 311 | 11.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 46 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/trabajo.rs` | Backend (Rust) | 337 | 297 | 11.3 KB | Separar traits/puertos por agregados o entidades de dominio |
| 47 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/factura/search.rs` | Backend (Rust) | 336 | 306 | 10.7 KB | Separar traits/puertos por agregados o entidades de dominio |
| 48 | 🔵 Medio | `crates/certaro-application/src/use_cases/comercial.rs` | Backend (Rust) | 335 | 283 | 11.8 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 49 | 🔵 Medio | `src/views/certificados/CertificadoDetalleView.vue` | Frontend (Vue/TS) | 334 | 299 | 12.0 KB | Extraer subcomponentes secundarios y modularizar template |
| 50 | 🔵 Medio | `src/views/admin/RolesView.vue` | Frontend (Vue/TS) | 330 | 305 | 11.0 KB | Extraer subcomponentes secundarios y modularizar template |
| 51 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/movimiento/mod.rs` | Backend (Rust) | 330 | 286 | 11.7 KB | Separar traits/puertos por agregados o entidades de dominio |
| 52 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/format.rs` | Backend (Rust) | 326 | 273 | 10.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 53 | 🔵 Medio | `src/views/calendario/components/CalendarioEventoModal.vue` | Frontend (Vue/TS) | 324 | 301 | 9.7 KB | Extraer subcomponentes secundarios y modularizar template |
| 54 | 🔵 Medio | `crates/certaro-infrastructure/tests/esquema.rs` | Backend (Rust) | 322 | 280 | 10.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 55 | 🔵 Medio | `crates/certaro-application/src/validation/movimientos.rs` | Backend (Rust) | 319 | 263 | 10.1 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 56 | 🔵 Medio | `crates/certaro-infrastructure/tests/adjuntos.rs` | Backend (Rust) | 318 | 257 | 9.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 57 | 🔵 Medio | `crates/certaro-import-legacy/src/transfer/proyectos.rs` | Backend (Rust) | 313 | 282 | 12.7 KB | Dividir etapas de transferencia/ETL en pipelines específicos |
| 58 | 🔵 Medio | `crates/certaro-infrastructure/src/files/store.rs` | Backend (Rust) | 312 | 264 | 10.7 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 59 | 🔵 Medio | `src/views/facturas/components/FacturaPagosModal.vue` | Frontend (Vue/TS) | 311 | 288 | 10.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 60 | 🔵 Medio | `crates/certaro-infrastructure/tests/tipos_movimiento.rs` | Backend (Rust) | 311 | 263 | 10.3 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 61 | 🔵 Medio | `crates/certaro-application/src/use_cases/calendario/recursos.rs` | Backend (Rust) | 310 | 262 | 9.5 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 62 | 🔵 Medio | `crates/certaro-domain/tests/money.rs` | Backend (Rust) | 306 | 246 | 10.0 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 63 | 🔵 Medio | `src/api/mock/database.ts` | Frontend (Vue/TS) | 304 | 274 | 24.6 KB | Separar responsabilidades por dominio o capa |
| 64 | 🔵 Medio | `src/views/ordenes/components/OrdenEditorModal.vue` | Frontend (Vue/TS) | 304 | 282 | 10.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 65 | 🔵 Medio | `crates/certaro-application/src/validation/ordenes_trabajo.rs` | Backend (Rust) | 304 | 260 | 8.9 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 66 | 🔵 Medio | `crates/certaro-application/src/use_cases/ordenes_trabajo.rs` | Backend (Rust) | 302 | 251 | 11.0 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |

## 3. Desglose Detallado por Componente

### 3.1. Frontend (Vue / TypeScript)

#### `src/views/calendario/CalendarioView.vue` (499 líneas - MEDIO)
- **Composición:** `<template>`: 317 líneas | `<script setup>`: 180 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Dividir vista en subcomponentes modulares (tablas, modales, panels)

#### `src/views/ordenes/OrdenesView.vue` (458 líneas - MEDIO)
- **Composición:** `<template>`: 14 líneas | `<script setup>`: 237 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/proyectos/ProyectoDetalleView.vue` (450 líneas - MEDIO)
- **Composición:** `<template>`: 28 líneas | `<script setup>`: 163 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/liquidaciones/components/LiquidacionWizardModal.vue` (447 líneas - MEDIO)
- **Composición:** `<template>`: 150 líneas | `<script setup>`: 293 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/asistencia/AsistenciaView.vue` (441 líneas - MEDIO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 222 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/facturas/FacturasView.vue` (441 líneas - MEDIO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 198 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/stores/useKanbanStore.ts` (439 líneas - MEDIO)
- **Estructuras:** 24 funciones/métodos, 0 interfaces, 0 tipos
- **Estrategia recomendada:** Agrupar funciones en módulos utilitarios cohesivos

#### `src/components/domain/ProyectosTreeTable.vue` (438 líneas - MEDIO)
- **Composición:** `<template>`: 13 líneas | `<script setup>`: 267 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/comercial/CuentaCorrienteView.vue` (419 líneas - MEDIO)
- **Composición:** `<template>`: 17 líneas | `<script setup>`: 215 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/movimientos/components/MovimientoDrawer.vue` (377 líneas - MEDIO)
- **Composición:** `<template>`: 179 líneas | `<script setup>`: 196 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/clientes/ClientesView.vue` (373 líneas - MEDIO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 162 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/movimientos/MovimientosView.vue` (367 líneas - MEDIO)
- **Composición:** `<template>`: 16 líneas | `<script setup>`: 151 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/dashboard/DashboardView.vue` (366 líneas - MEDIO)
- **Composición:** `<template>`: 28 líneas | `<script setup>`: 153 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/trabajos/TrabajosView.vue` (359 líneas - MEDIO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 171 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/empleados/EmpleadosView.vue` (339 líneas - MEDIO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 135 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/WelcomeView.vue` (339 líneas - MEDIO)
- **Composición:** `<template>`: 218 líneas | `<script setup>`: 119 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/admin/UsuariosView.vue` (338 líneas - MEDIO)
- **Composición:** `<template>`: 12 líneas | `<script setup>`: 140 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/certificados/CertificadoDetalleView.vue` (334 líneas - MEDIO)
- **Composición:** `<template>`: 47 líneas | `<script setup>`: 188 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/admin/RolesView.vue` (330 líneas - MEDIO)
- **Composición:** `<template>`: 12 líneas | `<script setup>`: 152 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/calendario/components/CalendarioEventoModal.vue` (324 líneas - MEDIO)
- **Composición:** `<template>`: 145 líneas | `<script setup>`: 177 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/facturas/components/FacturaPagosModal.vue` (311 líneas - MEDIO)
- **Composición:** `<template>`: 27 líneas | `<script setup>`: 197 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/api/mock/database.ts` (304 líneas - MEDIO)
- **Estructuras:** 9 funciones/métodos, 0 interfaces, 0 tipos
- **Estrategia recomendada:** Separar responsabilidades por dominio o capa

#### `src/views/ordenes/components/OrdenEditorModal.vue` (304 líneas - MEDIO)
- **Composición:** `<template>`: 150 líneas | `<script setup>`: 150 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

### 3.2. Backend (Rust)

#### `crates/certaro-infrastructure/tests/dashboard.rs` (874 líneas - CRÍTICO)
- **Estructuras:** 31 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-import-legacy/tests/import.rs` (846 líneas - CRÍTICO)
- **Estructuras:** 22 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/certificado.rs` (499 líneas - MEDIO)
- **Estructuras:** 24 funciones, 3 bloques impl, 4 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-migration/src/m20260101_000001_create_schema.rs` (499 líneas - MEDIO)
- **Estructuras:** 2 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/use_cases/liquidaciones/calculation.rs` (495 líneas - MEDIO)
- **Estructuras:** 22 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/persistence/repositories/liquidacion.rs` (467 líneas - MEDIO)
- **Estructuras:** 24 funciones, 3 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/reporting/pdf/certificado.rs` (464 líneas - MEDIO)
- **Estructuras:** 14 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/reporting/pdf/table.rs` (464 líneas - MEDIO)
- **Estructuras:** 32 funciones, 5 bloques impl, 4 structs, 1 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/orden_trabajo.rs` (450 líneas - MEDIO)
- **Estructuras:** 25 funciones, 3 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/reporting/pdf/canvas/mod.rs` (446 líneas - MEDIO)
- **Estructuras:** 28 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/backup/json.rs` (440 líneas - MEDIO)
- **Estructuras:** 14 funciones, 0 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/cliente.rs` (438 líneas - MEDIO)
- **Estructuras:** 23 funciones, 3 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/persistence/repositories/proyecto.rs` (431 líneas - MEDIO)
- **Estructuras:** 22 funciones, 3 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/reporting/pdf/liquidacion/sections.rs` (414 líneas - MEDIO)
- **Estructuras:** 9 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/calendario.rs` (401 líneas - MEDIO)
- **Estructuras:** 27 funciones, 6 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/tests/backup.rs` (400 líneas - MEDIO)
- **Estructuras:** 24 funciones, 4 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/config/sections.rs` (391 líneas - MEDIO)
- **Estructuras:** 14 funciones, 14 bloques impl, 14 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/use_cases/dashboard.rs` (391 líneas - MEDIO)
- **Estructuras:** 14 funciones, 1 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-application/src/use_cases/calendario/eventos.rs` (373 líneas - MEDIO)
- **Estructuras:** 5 funciones, 1 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `src-tauri/src/lib.rs` (369 líneas - MEDIO)
- **Estructuras:** 3 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/categoria.rs` (360 líneas - MEDIO)
- **Estructuras:** 19 funciones, 3 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-application/src/use_cases/liquidaciones/mod.rs` (357 líneas - MEDIO)
- **Estructuras:** 13 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/persistence/unit_of_work.rs` (355 líneas - MEDIO)
- **Estructuras:** 35 funciones, 4 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Modularizar datos y queries por entidad de base de datos

#### `crates/certaro-application/src/use_cases/asistencias.rs` (354 líneas - MEDIO)
- **Estructuras:** 13 funciones, 2 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/persistence/repositories/dashboard/analytics.rs` (352 líneas - MEDIO)
- **Estructuras:** 7 funciones, 1 bloques impl, 4 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/reporting/xlsx.rs` (344 líneas - MEDIO)
- **Estructuras:** 13 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/validation/clientes.rs` (343 líneas - MEDIO)
- **Estructuras:** 16 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/backup/service.rs` (343 líneas - MEDIO)
- **Estructuras:** 16 funciones, 4 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/trabajo.rs` (337 líneas - MEDIO)
- **Estructuras:** 19 funciones, 3 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/persistence/repositories/factura/search.rs` (336 líneas - MEDIO)
- **Estructuras:** 12 funciones, 2 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-application/src/use_cases/comercial.rs` (335 líneas - MEDIO)
- **Estructuras:** 14 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/persistence/repositories/movimiento/mod.rs` (330 líneas - MEDIO)
- **Estructuras:** 11 funciones, 2 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/reporting/format.rs` (326 líneas - MEDIO)
- **Estructuras:** 21 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/tests/esquema.rs` (322 líneas - MEDIO)
- **Estructuras:** 14 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/validation/movimientos.rs` (319 líneas - MEDIO)
- **Estructuras:** 17 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/tests/adjuntos.rs` (318 líneas - MEDIO)
- **Estructuras:** 20 funciones, 3 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-import-legacy/src/transfer/proyectos.rs` (313 líneas - MEDIO)
- **Estructuras:** 4 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Dividir etapas de transferencia/ETL en pipelines específicos

#### `crates/certaro-infrastructure/src/files/store.rs` (312 líneas - MEDIO)
- **Estructuras:** 14 funciones, 3 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/tests/tipos_movimiento.rs` (311 líneas - MEDIO)
- **Estructuras:** 20 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/use_cases/calendario/recursos.rs` (310 líneas - MEDIO)
- **Estructuras:** 9 funciones, 1 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-domain/tests/money.rs` (306 líneas - MEDIO)
- **Estructuras:** 28 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/validation/ordenes_trabajo.rs` (304 líneas - MEDIO)
- **Estructuras:** 16 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/use_cases/ordenes_trabajo.rs` (302 líneas - MEDIO)
- **Estructuras:** 11 funciones, 2 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

## 4. Plan de Segmentación Recomendado

A partir de los archivos identificados, se recomienda priorizar en las siguientes fases:

1. **Fase 1: Desacoplamiento de Clientes API Frontend**
   - Descomponer `src/api/client.ts` (1700+ líneas) en submódulos por dominio (`src/api/domains/ordenes.ts`, `personal.ts`, `facturas.ts`, etc.).
2. **Fase 2: Segmentación de Vistas Complejas en Vue**
   - Extraer la lógica de `CalendarioView.vue` y `KanbanView.vue` a composables (`useCalendario.ts`, `useKanban.ts`).
   - Modularizar sub-componentes visuales (tablas, formularios modales, cards).
3. **Fase 3: Refactorización de Capas de Negocio y Repositorios en Rust**
   - Descomponer `repositories.rs` dividiendo los traits de puertos en archivos específicos por agregado.
   - Segmentar casos de uso extensos (`liquidaciones.rs`, `kanban.rs`, `calendario.rs`) en submódulos jerárquicos.
