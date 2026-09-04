# Reporte de Archivos Monolíticos y Plan de Segmentación

> **Fecha de generación:** 2026-09-04  
> **Criterio:** Archivos con >= 300 líneas  
> **Total detectados:** 39 (🔴 0 Críticos, 🟡 0 Altos, 🔵 39 Medios)

## 1. Resumen Ejecutivo

| Métrica | Valor |
| :--- | :--- |
| Archivos monolíticos | **39** |
| 🔴 Críticos (>= 800 líneas) | **0** |
| 🟡 Altos (500 - 799 líneas) | **0** |
| 🔵 Moderados (300 - 499 líneas) | **39** |

## 2. Inventario de Archivos Monolíticos

| # | Nivel | Archivo | Capa | Líneas Totales | Líneas Código | Tamaño (KB) | Sugerencia de Segmentación |
|---|---|---|---|---|---|---|---|
| 1 | 🔵 Medio | `src/views/dashboard/DashboardView.vue` | Frontend (Vue/TS) | 366 | 315 | 14.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 2 | 🔵 Medio | `src/views/liquidaciones/components/LiquidacionWizardModal.vue` | Frontend (Vue/TS) | 363 | 323 | 11.5 KB | Extraer subcomponentes secundarios y modularizar template |
| 3 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/categoria.rs` | Backend (Rust) | 360 | 315 | 12.0 KB | Separar traits/puertos por agregados o entidades de dominio |
| 4 | 🔵 Medio | `src/views/trabajos/TrabajosView.vue` | Frontend (Vue/TS) | 359 | 324 | 12.6 KB | Extraer subcomponentes secundarios y modularizar template |
| 5 | 🔵 Medio | `crates/certaro-application/src/use_cases/liquidaciones/mod.rs` | Backend (Rust) | 357 | 300 | 12.7 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 6 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/unit_of_work.rs` | Backend (Rust) | 355 | 302 | 12.8 KB | Modularizar datos y queries por entidad de base de datos |
| 7 | 🔵 Medio | `crates/certaro-application/src/use_cases/asistencias.rs` | Backend (Rust) | 354 | 302 | 13.0 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 8 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/dashboard/analytics.rs` | Backend (Rust) | 352 | 322 | 11.8 KB | Separar traits/puertos por agregados o entidades de dominio |
| 9 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/xlsx.rs` | Backend (Rust) | 344 | 288 | 12.5 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 10 | 🔵 Medio | `crates/certaro-application/src/validation/clientes.rs` | Backend (Rust) | 343 | 304 | 10.0 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 11 | 🔵 Medio | `crates/certaro-infrastructure/src/backup/service.rs` | Backend (Rust) | 343 | 273 | 12.1 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 12 | 🔵 Medio | `src/views/empleados/EmpleadosView.vue` | Frontend (Vue/TS) | 339 | 304 | 12.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 13 | 🔵 Medio | `src/views/WelcomeView.vue` | Frontend (Vue/TS) | 339 | 293 | 11.8 KB | Extraer subcomponentes secundarios y modularizar template |
| 14 | 🔵 Medio | `src/views/admin/UsuariosView.vue` | Frontend (Vue/TS) | 338 | 311 | 11.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 15 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/trabajo.rs` | Backend (Rust) | 337 | 297 | 11.3 KB | Separar traits/puertos por agregados o entidades de dominio |
| 16 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/factura/search.rs` | Backend (Rust) | 336 | 306 | 10.7 KB | Separar traits/puertos por agregados o entidades de dominio |
| 17 | 🔵 Medio | `crates/certaro-application/src/use_cases/comercial.rs` | Backend (Rust) | 335 | 283 | 11.8 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 18 | 🔵 Medio | `src/views/certificados/CertificadoDetalleView.vue` | Frontend (Vue/TS) | 334 | 299 | 12.0 KB | Extraer subcomponentes secundarios y modularizar template |
| 19 | 🔵 Medio | `src/views/admin/RolesView.vue` | Frontend (Vue/TS) | 330 | 305 | 11.0 KB | Extraer subcomponentes secundarios y modularizar template |
| 20 | 🔵 Medio | `crates/certaro-infrastructure/src/persistence/repositories/movimiento/mod.rs` | Backend (Rust) | 330 | 286 | 11.7 KB | Separar traits/puertos por agregados o entidades de dominio |
| 21 | 🔵 Medio | `src/views/ordenes/components/OrdenFormModal.vue` | Frontend (Vue/TS) | 329 | 304 | 10.3 KB | Extraer subcomponentes secundarios y modularizar template |
| 22 | 🔵 Medio | `src/views/calendario/CalendarioView.vue` | Frontend (Vue/TS) | 326 | 280 | 9.9 KB | Extraer subcomponentes secundarios y modularizar template |
| 23 | 🔵 Medio | `crates/certaro-infrastructure/src/reporting/format.rs` | Backend (Rust) | 326 | 273 | 10.2 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 24 | 🔵 Medio | `src/views/calendario/components/CalendarioEventoModal.vue` | Frontend (Vue/TS) | 324 | 301 | 9.7 KB | Extraer subcomponentes secundarios y modularizar template |
| 25 | 🔵 Medio | `crates/certaro-infrastructure/tests/esquema.rs` | Backend (Rust) | 322 | 280 | 10.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 26 | 🔵 Medio | `crates/certaro-application/src/validation/movimientos.rs` | Backend (Rust) | 319 | 263 | 10.1 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 27 | 🔵 Medio | `crates/certaro-infrastructure/tests/adjuntos.rs` | Backend (Rust) | 318 | 257 | 9.8 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 28 | 🔵 Medio | `crates/certaro-import-legacy/src/transfer/proyectos.rs` | Backend (Rust) | 313 | 282 | 12.7 KB | Dividir etapas de transferencia/ETL en pipelines específicos |
| 29 | 🔵 Medio | `crates/certaro-infrastructure/src/files/store.rs` | Backend (Rust) | 312 | 264 | 10.7 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 30 | 🔵 Medio | `src/views/facturas/components/FacturaPagosModal.vue` | Frontend (Vue/TS) | 311 | 288 | 10.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 31 | 🔵 Medio | `crates/certaro-infrastructure/tests/tipos_movimiento.rs` | Backend (Rust) | 311 | 263 | 10.3 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 32 | 🔵 Medio | `crates/certaro-application/src/use_cases/calendario/recursos.rs` | Backend (Rust) | 310 | 262 | 9.5 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |
| 33 | 🔵 Medio | `crates/certaro-domain/tests/money.rs` | Backend (Rust) | 306 | 246 | 10.0 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 34 | 🔵 Medio | `src/api/mock/database.ts` | Frontend (Vue/TS) | 304 | 274 | 24.6 KB | Separar responsabilidades por dominio o capa |
| 35 | 🔵 Medio | `src/views/ordenes/components/OrdenEditorModal.vue` | Frontend (Vue/TS) | 304 | 282 | 10.2 KB | Extraer subcomponentes secundarios y modularizar template |
| 36 | 🔵 Medio | `crates/certaro-application/src/validation/ordenes_trabajo.rs` | Backend (Rust) | 304 | 260 | 8.9 KB | Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`) |
| 37 | 🔵 Medio | `src/views/comercial/CuentaCorrienteView.vue` | Frontend (Vue/TS) | 303 | 263 | 10.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 38 | 🔵 Medio | `src/views/movimientos/components/MovimientoDrawer.vue` | Frontend (Vue/TS) | 303 | 276 | 9.4 KB | Extraer subcomponentes secundarios y modularizar template |
| 39 | 🔵 Medio | `crates/certaro-application/src/use_cases/ordenes_trabajo.rs` | Backend (Rust) | 302 | 251 | 11.0 KB | Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones) |

## 3. Desglose Detallado por Componente

### 3.1. Frontend (Vue / TypeScript)

#### `src/views/dashboard/DashboardView.vue` (366 líneas - MEDIO)
- **Composición:** `<template>`: 28 líneas | `<script setup>`: 153 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/liquidaciones/components/LiquidacionWizardModal.vue` (363 líneas - MEDIO)
- **Composición:** `<template>`: 69 líneas | `<script setup>`: 290 líneas | `<style>`: 0 líneas
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

#### `src/views/ordenes/components/OrdenFormModal.vue` (329 líneas - MEDIO)
- **Composición:** `<template>`: 148 líneas | `<script setup>`: 177 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/calendario/CalendarioView.vue` (326 líneas - MEDIO)
- **Composición:** `<template>`: 156 líneas | `<script setup>`: 168 líneas | `<style>`: 0 líneas
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

#### `src/views/comercial/CuentaCorrienteView.vue` (303 líneas - MEDIO)
- **Composición:** `<template>`: 17 líneas | `<script setup>`: 209 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

#### `src/views/movimientos/components/MovimientoDrawer.vue` (303 líneas - MEDIO)
- **Composición:** `<template>`: 112 líneas | `<script setup>`: 189 líneas | `<style>`: 0 líneas
- **Estrategia recomendada:** Extraer subcomponentes secundarios y modularizar template

### 3.2. Backend (Rust)

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
