# Reporte de Archivos Monolíticos y Plan de Segmentación

> **Fecha de generación:** 2026-09-04  
> **Criterio:** Archivos con >= 300 líneas  
> **Total detectados:** 66 (🔴 7 Críticos, 🟡 21 Altos, 🔵 38 Medios)

## 1. Resumen Ejecutivo

| Métrica | Valor |
| :--- | :--- |
| Archivos monolíticos | **66** |
| 🔴 Críticos (>= 800 líneas) | **7** |
| 🟡 Altos (500 - 799 líneas) | **21** |
| 🔵 Moderados (300 - 499 líneas) | **38** |

## 2. Inventario de Archivos Monolíticos

| # | Nivel | Archivo | Capa | Líneas Totales | Líneas Código | Tamaño (KB) | Sugerencia de Segmentación |
|---|---|---|---|---|---|---|---|
| 1 | 🔴 Crítico | `src/api/client.ts` | Frontend (Vue/TS) | 1713 | 1613 | 83.9 KB | Dividir en clientes de API modulares por dominio (órdenes, facturas, personal) |
| 2 | 🔴 Crítico | `crates/certaro-import-legacy/src/transfer.rs` | Backend (Rust) | 1352 | 1218 | 55.4 KB | Dividir etapas de transferencia/ETL en pipelines específicos |
| 3 | 🔴 Crítico | `crates/certaro-application/src/ports/repositories.rs` | Backend (Rust) | 1173 | 851 | 48.1 KB | Separar traits/puertos por agregados o entidades de dominio |
| 4 | 🔴 Crítico | `src/views/calendario/CalendarioView.vue` | Frontend (Vue/TS) | 1043 | 929 | 35.3 KB | Extraer logica y estado a composables dedicados (`use*.ts`) |
| 5 | 🔴 Crítico | `src/views/kanban/KanbanView.vue` | Frontend (Vue/TS) | 909 | 777 | 27.9 KB | Extraer logica y estado a composables dedicados (`use*.ts`) |
| 6 | 🔴 Crítico | `crates/certaro-application/src/use_cases/liquidaciones.rs` | Backend (Rust) | 833 | 712 | 30.7 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 7 | 🔴 Crítico | `crates/certaro-application/src/use_cases/kanban.rs` | Backend (Rust) | 804 | 698 | 29.0 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 8 | 🟡 Alto | `src/lib/helpRegistry.ts` | Frontend (Vue/TS) | 746 | 716 | 41.6 KB | Separar diccionarios o contenidos estáticos en archivos de datos por sección |
| 9 | 🟡 Alto | `src/views/ordenes/OrdenDetalleView.vue` | Frontend (Vue/TS) | 746 | 682 | 28.7 KB | Extraer subcomponentes secundarios y modularizar template |
| 10 | 🟡 Alto | `crates/certaro-infrastructure/src/persistence/seed.rs` | Backend (Rust) | 689 | 617 | 31.0 KB | Modularizar datos y queries por entidad de base de datos |
| 11 | 🟡 Alto | `crates/certaro-application/src/use_cases/calendario.rs` | Backend (Rust) | 684 | 587 | 23.4 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 12 | 🟡 Alto | `src/views/facturas/FacturasView.vue` | Frontend (Vue/TS) | 668 | 606 | 24.4 KB | Extraer logica y estado a composables dedicados (`use*.ts`) |
| 13 | 🟡 Alto | `src/views/movimientos/MovimientosView.vue` | Frontend (Vue/TS) | 667 | 611 | 24.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 14 | 🟡 Alto | `src/views/liquidaciones/LiquidacionesView.vue` | Frontend (Vue/TS) | 664 | 589 | 24.9 KB | Extraer logica y estado a composables dedicados (`use*.ts`) |
| 15 | 🟡 Alto | `crates/certaro-infrastructure/src/persistence/repositories/dashboard.rs` | Backend (Rust) | 639 | 555 | 22.1 KB | Separar traits/puertos por agregados o entidades de dominio |
| 16 | 🟡 Alto | `crates/certaro-application/src/use_cases/certificados.rs` | Backend (Rust) | 590 | 500 | 22.3 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 17 | 🟡 Alto | `crates/certaro-application/src/use_cases/auth.rs` | Backend (Rust) | 580 | 493 | 19.4 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 18 | 🟡 Alto | `crates/certaro-application/src/use_cases/facturas.rs` | Backend (Rust) | 580 | 481 | 20.4 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 19 | 🟡 Alto | `crates/certaro-infrastructure/src/persistence/repositories/factura.rs` | Backend (Rust) | 570 | 502 | 19.7 KB | Separar traits/puertos por agregados o entidades de dominio |
| 20 | 🟡 Alto | `crates/certaro-application/src/config.rs` | Backend (Rust) | 565 | 482 | 18.4 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 21 | 🟡 Alto | `crates/certaro-infrastructure/src/persistence/repositories/kanban.rs` | Backend (Rust) | 549 | 458 | 19.1 KB | Separar traits/puertos por agregados o entidades de dominio |
| 22 | 🟡 Alto | `crates/certaro-infrastructure/src/reporting/pdf/liquidacion.rs` | Backend (Rust) | 546 | 470 | 17.5 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 23 | 🟡 Alto | `crates/certaro-infrastructure/src/persistence/repositories/movimiento.rs` | Backend (Rust) | 537 | 474 | 19.6 KB | Separar traits/puertos por agregados o entidades de dominio |
| 24 | 🟡 Alto | `crates/certaro-infrastructure/src/reporting/pdf/canvas.rs` | Backend (Rust) | 535 | 455 | 17.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 25 | 🟡 Alto | `src/views/dashboard/DashboardView.vue` | Frontend (Vue/TS) | 529 | 471 | 21.7 KB | Extraer subcomponentes secundarios y modularizar template |
| 26 | 🟡 Alto | `crates/certaro-infrastructure/src/persistence/repositories/auth.rs` | Backend (Rust) | 524 | 438 | 18.0 KB | Separar traits/puertos por agregados o entidades de dominio |
| 27 | 🟡 Alto | `crates/certaro-import-legacy/src/derive.rs` | Backend (Rust) | 521 | 424 | 20.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 28 | 🟡 Alto | `crates/certaro-domain/src/enums.rs` | Backend (Rust) | 516 | 421 | 15.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 29 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/certificado.rs` | Backend (Rust) | 499 | 440 | 17.4 KB | Separar traits/puertos por agregados o entidades de dominio |
| 30 | 🔵 Medio | `crates/certaro-migration/src/m20260101_000001_create_schema.rs` | Backend (Rust) | 499 | 465 | 23.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 31 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/liquidacion.rs` | Backend (Rust) | 467 | 409 | 17.1 KB | Separar traits/puertos por agregados o entidades de dominio |
| 32 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/pdf/certificado.rs` | Backend (Rust) | 464 | 399 | 16.5 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 33 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/pdf/table.rs` | Backend (Rust) | 464 | 383 | 13.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 34 | 🔵 Medio | `src/views/ordenes/OrdenesView.vue` | Frontend (Vue/TS) | 458 | 411 | 15.6 KB | Extraer subcomponentes secundarios y modularizar template |
| 35 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/orden_trabajo.rs` | Backend (Rust) | 450 | 403 | 16.0 KB | Separar traits/puertos por agregados o entidades de dominio |
| 36 | 🔵 Medio | `src/views/proyectos/ProyectoDetalleView.vue` | Frontend (Vue/TS) | 448 | 404 | 17.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 37 | 🔵 Medio | `src/views/asistencia/AsistenciaView.vue` | Frontend (Vue/TS) | 441 | 396 | 15.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 38 | 🔵 Medio | `crates/certaro-infrastructure/src/backup/json.rs` | Backend (Rust) | 440 | 371 | 14.3 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 39 | 🔵 Medio | `src/stores/useKanbanStore.ts` | Frontend (Vue/TS) | 439 | 402 | 12.5 KB | Agrupar funciones en módulos utilitarios cohesivos |
| 40 | 🔵 Medio | `src/components/domain/ProyectosTreeTable.vue` | Frontend (Vue/TS) | 438 | 400 | 16.5 KB | Extraer subcomponentes secundarios y modularizar template |
| 41 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/cliente.rs` | Backend (Rust) | 438 | 387 | 15.3 KB | Separar traits/puertos por agregados o entidades de dominio |
| 42 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/proyecto.rs` | Backend (Rust) | 431 | 373 | 15.3 KB | Separar traits/puertos por agregados o entidades de dominio |
| 43 | 🔵 Medio | `src/views/comercial/CuentaCorrienteView.vue` | Frontend (Vue/TS) | 419 | 374 | 15.7 KB | Extraer subcomponentes secundarios y modularizar template |
| 44 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/calendario.rs` | Backend (Rust) | 401 | 332 | 13.7 KB | Separar traits/puertos por agregados o entidades de dominio |
| 45 | 🔵 Medio | `crates/certaro-application/src/use_cases/dashboard.rs` | Backend (Rust) | 391 | 331 | 15.9 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 46 | 🔵 Medio | `src/views/clientes/ClientesView.vue` | Frontend (Vue/TS) | 373 | 334 | 13.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 47 | 🔵 Medio | `src-tauri/src/lib.rs` | Backend (Rust) | 369 | 326 | 16.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 48 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/categoria.rs` | Backend (Rust) | 360 | 315 | 12.0 KB | Separar traits/puertos por agregados o entidades de dominio |
| 49 | 🔵 Medio | `src/views/trabajos/TrabajosView.vue` | Frontend (Vue/TS) | 359 | 324 | 12.6 KB | Extraer subcomponentes secundarios y modularizar template |
| 50 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/unit_of_work.rs` | Backend (Rust) | 355 | 302 | 12.8 KB | Modularizar datos y queries por entidad de base de datos |
| 51 | 🔵 Medio | `crates/certaro-application/src/use_cases/asistencias.rs` | Backend (Rust) | 354 | 302 | 13.0 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 52 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/xlsx.rs` | Backend (Rust) | 344 | 288 | 12.5 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 53 | 🔵 Medio | `crates/certaro-application/src/validation/clientes.rs` | Backend (Rust) | 343 | 304 | 10.0 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 54 | 🔵 Medio | `crates/certaro-infrastructure/src/backup/service.rs` | Backend (Rust) | 343 | 273 | 12.1 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 55 | 🔵 Medio | `src/views/empleados/EmpleadosView.vue` | Frontend (Vue/TS) | 339 | 304 | 12.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 56 | 🔵 Medio | `src/views/WelcomeView.vue` | Frontend (Vue/TS) | 339 | 293 | 11.8 KB | Extraer subcomponentes secundarios y modularizar template |
| 57 | 🔵 Medio | `src/views/admin/UsuariosView.vue` | Frontend (Vue/TS) | 338 | 311 | 11.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 58 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/trabajo.rs` | Backend (Rust) | 337 | 297 | 11.3 KB | Separar traits/puertos por agregados o entidades de dominio |
| 59 | 🔵 Medio | `crates/certaro-application/src/use_cases/comercial.rs` | Backend (Rust) | 335 | 283 | 11.8 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 60 | 🔵 Medio | `src/views/admin/RolesView.vue` | Frontend (Vue/TS) | 330 | 305 | 11.0 KB | Extraer subcomponentes secundarios y modularizar template |
| 61 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/format.rs` | Backend (Rust) | 326 | 273 | 10.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 62 | 🔵 Medio | `crates/certaro-application/src/validation/movimientos.rs` | Backend (Rust) | 319 | 263 | 10.1 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 63 | 🔵 Medio | `crates/certaro-infrastructure/src/files/store.rs` | Backend (Rust) | 312 | 264 | 10.7 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 64 | 🔵 Medio | `src/views/certificados/CertificadoDetalleView.vue` | Frontend (Vue/TS) | 307 | 274 | 11.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 65 | 🔵 Medio | `crates/certaro-application/src/validation/ordenes_trabajo.rs` | Backend (Rust) | 304 | 260 | 8.9 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 66 | 🔵 Medio | `crates/certaro-application/src/use_cases/ordenes_trabajo.rs` | Backend (Rust) | 302 | 251 | 11.0 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |

## 3. Desglose Detallado por Componente

### 3.1. Frontend (Vue / TypeScript)

#### `src/api/client.ts` (1713 líneas - CRÍTICO)
- **Estructuras:** 21 funciones/métodos, 15 interfaces, 1 tipos
- **Estrategia recomendada:** Dividir en clientes de API modulares por dominio (órdenes, facturas, personal)

#### `src/views/calendario/CalendarioView.vue` (1043 líneas - CRÍTICO)
- **Composición:** `<template>`: 540 líneas | `<script setup>`: 498 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer logica y estado a composables dedicados (`use*.ts`)

#### `src/views/kanban/KanbanView.vue` (909 líneas - CRÍTICO)
- **Composición:** `<template>`: 173 líneas | `<script setup>`: 734 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer logica y estado a composables dedicados (`use*.ts`)

#### `src/lib/helpRegistry.ts` (746 líneas - ALTO)
- **Estructuras:** 0 funciones/métodos, 1 interfaces, 0 tipos
- **Estrategia recomendada:** Separar diccionarios o contenidos estáticos en archivos de datos por sección

#### `src/views/ordenes/OrdenDetalleView.vue` (746 líneas - ALTO)
- **Composición:** `<template>`: 18 líneas | `<script setup>`: 312 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/facturas/FacturasView.vue` (668 líneas - ALTO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 353 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer logica y estado a composables dedicados (`use*.ts`)

#### `src/views/movimientos/MovimientosView.vue` (667 líneas - ALTO)
- **Composición:** `<template>`: 16 líneas | `<script setup>`: 283 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/liquidaciones/LiquidacionesView.vue` (664 líneas - ALTO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 366 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer logica y estado a composables dedicados (`use*.ts`)

#### `src/views/dashboard/DashboardView.vue` (529 líneas - ALTO)
- **Composición:** `<template>`: 28 líneas | `<script setup>`: 162 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/ordenes/OrdenesView.vue` (458 líneas - MEDIO)
- **Composición:** `<template>`: 14 líneas | `<script setup>`: 237 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/proyectos/ProyectoDetalleView.vue` (448 líneas - MEDIO)
- **Composición:** `<template>`: 28 líneas | `<script setup>`: 163 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/asistencia/AsistenciaView.vue` (441 líneas - MEDIO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 222 líneas | `<style>`: 0 líneas
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

#### `src/views/clientes/ClientesView.vue` (373 líneas - MEDIO)
- **Composición:** `<template>`: 10 líneas | `<script setup>`: 162 líneas | `<style>`: 0 líneas
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

#### `src/views/admin/RolesView.vue` (330 líneas - MEDIO)
- **Composición:** `<template>`: 12 líneas | `<script setup>`: 152 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/certificados/CertificadoDetalleView.vue` (307 líneas - MEDIO)
- **Composición:** `<template>`: 47 líneas | `<script setup>`: 161 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

### 3.2. Backend (Rust)

#### `crates/certaro-import-legacy/src/transfer.rs` (1352 líneas - CRÍTICO)
- **Estructuras:** 22 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Dividir etapas de transferencia/ETL en pipelines específicos

#### `crates/certaro-application/src/ports/repositories.rs` (1173 líneas - CRÍTICO)
- **Estructuras:** 274 funciones, 2 bloques impl, 28 structs, 2 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-application/src/use_cases/liquidaciones.rs` (833 líneas - CRÍTICO)
- **Estructuras:** 35 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-application/src/use_cases/kanban.rs` (804 líneas - CRÍTICO)
- **Estructuras:** 25 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/persistence/seed.rs` (689 líneas - ALTO)
- **Estructuras:** 1 funciones, 0 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar datos y queries por entidad de base de datos

#### `crates/certaro-application/src/use_cases/calendario.rs` (684 líneas - ALTO)
- **Estructuras:** 15 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/persistence/repositories/dashboard.rs` (639 líneas - ALTO)
- **Estructuras:** 24 funciones, 2 bloques impl, 8 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-application/src/use_cases/certificados.rs` (590 líneas - ALTO)
- **Estructuras:** 23 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-application/src/use_cases/auth.rs` (580 líneas - ALTO)
- **Estructuras:** 18 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-application/src/use_cases/facturas.rs` (580 líneas - ALTO)
- **Estructuras:** 21 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/persistence/repositories/factura.rs` (570 líneas - ALTO)
- **Estructuras:** 27 funciones, 3 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-application/src/config.rs` (565 líneas - ALTO)
- **Estructuras:** 17 funciones, 15 bloques impl, 15 structs, 6 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/kanban.rs` (549 líneas - ALTO)
- **Estructuras:** 40 funciones, 10 bloques impl, 5 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/reporting/pdf/liquidacion.rs` (546 líneas - ALTO)
- **Estructuras:** 17 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/movimiento.rs` (537 líneas - ALTO)
- **Estructuras:** 19 funciones, 3 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/reporting/pdf/canvas.rs` (535 líneas - ALTO)
- **Estructuras:** 36 funciones, 4 bloques impl, 3 structs, 2 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/auth.rs` (524 líneas - ALTO)
- **Estructuras:** 41 funciones, 10 bloques impl, 5 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-import-legacy/src/derive.rs` (521 líneas - ALTO)
- **Estructuras:** 6 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-domain/src/enums.rs` (516 líneas - ALTO)
- **Estructuras:** 36 funciones, 7 bloques impl, 0 structs, 8 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/certificado.rs` (499 líneas - MEDIO)
- **Estructuras:** 24 funciones, 3 bloques impl, 4 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-migration/src/m20260101_000001_create_schema.rs` (499 líneas - MEDIO)
- **Estructuras:** 2 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

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

#### `crates/certaro-infrastructure/src/backup/json.rs` (440 líneas - MEDIO)
- **Estructuras:** 14 funciones, 0 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/cliente.rs` (438 líneas - MEDIO)
- **Estructuras:** 23 funciones, 3 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/persistence/repositories/proyecto.rs` (431 líneas - MEDIO)
- **Estructuras:** 22 funciones, 3 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/persistence/repositories/calendario.rs` (401 líneas - MEDIO)
- **Estructuras:** 27 funciones, 6 bloques impl, 3 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-application/src/use_cases/dashboard.rs` (391 líneas - MEDIO)
- **Estructuras:** 14 funciones, 1 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `src-tauri/src/lib.rs` (369 líneas - MEDIO)
- **Estructuras:** 3 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/persistence/repositories/categoria.rs` (360 líneas - MEDIO)
- **Estructuras:** 19 funciones, 3 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Separar traits/puertos por agregados o entidades de dominio

#### `crates/certaro-infrastructure/src/persistence/unit_of_work.rs` (355 líneas - MEDIO)
- **Estructuras:** 35 funciones, 4 bloques impl, 2 structs, 0 enums
- **Estrategia recomendada:** Modularizar datos y queries por entidad de base de datos

#### `crates/certaro-application/src/use_cases/asistencias.rs` (354 líneas - MEDIO)
- **Estructuras:** 13 funciones, 2 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

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

#### `crates/certaro-application/src/use_cases/comercial.rs` (335 líneas - MEDIO)
- **Estructuras:** 14 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)

#### `crates/certaro-infrastructure/src/reporting/format.rs` (326 líneas - MEDIO)
- **Estructuras:** 21 funciones, 0 bloques impl, 0 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-application/src/validation/movimientos.rs` (319 líneas - MEDIO)
- **Estructuras:** 17 funciones, 1 bloques impl, 1 structs, 0 enums
- **Estrategia recomendada:** Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)

#### `crates/certaro-infrastructure/src/files/store.rs` (312 líneas - MEDIO)
- **Estructuras:** 14 funciones, 3 bloques impl, 1 structs, 0 enums
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
