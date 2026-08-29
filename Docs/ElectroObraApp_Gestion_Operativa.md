# ElectroObra: Sistema de Gestión Operativa
## Documento de Visión y Requerimientos (No Técnico)

### 1. Objetivo del Proyecto
El propósito central de este sistema es profesionalizar y simplificar la administración diaria de una pequeña empresa dedicada al mantenimiento y la construcción. El enfoque no es la contabilidad fiscal compleja, sino el **control operativo y el flujo de caja real**.

### 2. Filosofía de Diseño
*   **Minimalista pero Profesional**: Interfaz limpia y fácil de usar, pero con una estructura robusta por detrás.
*   **Flexibilidad Real**: El sistema contempla situaciones cotidianas como pagos parciales, adelantos a empleados, faltas injustificadas y deudas de clientes.
*   **Escalabilidad**: Diseñado para crecer. Aunque inicia como una herramienta personal local, está preparado para migrar a la web o ser multiusuario con un servidor central.

### 3. Módulos Principales

#### 💰 Gestión de Movimientos
Cualquier entrada o salida de dinero se registra aquí:
*   **Ingresos**: Cobros a clientes por servicios.
*   **Gastos**: Insumos, herramientas, seguros, combustible, monotributo.
*   **Pagos a Empleados**: Liquidaciones, quincenas, adelantos.
*   **Ajustes**: Correcciones manuales para cuadrar caja.

#### 👷 Gestión de Empleados y Liquidación
Control total sobre el personal:
*   **Perfiles**: Datos personales y tarifa base de pago.
*   **Frecuencia de Pago**: Configurable (diario, semanal, quincenal, mensual).
*   **Asistencia**: Registro de faltas (completas, medias faltas, justificadas) con impacto automático en el sueldo.
*   **Adelantos**: Registro de dinero entregado antes de la liquidación.

#### 📈 Proyectos y Rentabilidad (Feature Central)
El sistema permite asociar cada gasto o ingreso a un "Trabajo" específico.
*   **Cálculo de Rentabilidad**: Ingresos generados por el trabajo menos los costos (materiales + mano de obra).
*   **Ranking**: Saber qué trabajos son más rentables y cuáles no.

#### 🧾 Facturación y Deuda de Clientes
Control interno de lo facturado vs lo cobrado:
*   **Estado de Facturas**: Pendientes, parciales o cobradas.
*   **Saldo de Cliente**: Ver rápidamente cuánto debe cada cliente en tiempo real.

#### 🗃️ Otras Funcionalidades
*   **Multi-moneda**: Soporte para trabajar en Pesos Argentinos (ARS) y Dólares (USD) con tipos de cambio manuales.
*   **Adjuntos**: Capacidad de subir fotos de comprobantes, PDFs de facturas o imágenes de las obras.
*   **Exportaciones**: Generación de reportes profesionales en PDF, Excel para análisis profundo, y otros formatos (CSV, JSON).

### 4. Resumen de Beneficios
*   **Eliminación de Excel**: Centralización de datos dispersos.
*   **Visibilidad Financiera**: Dashboard con balance general, caja y alertas de deuda.
*   **Ahorro de Tiempo**: Cálculos automáticos de liquidaciones y rentabilidad.
