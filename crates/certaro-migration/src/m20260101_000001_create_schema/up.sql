CREATE TABLE tipos_movimiento (
    id           TEXT    NOT NULL PRIMARY KEY,
    nombre       TEXT    NOT NULL,
    descripcion  TEXT        NULL,
    es_ingreso   INTEGER NOT NULL DEFAULT 0 CHECK (es_ingreso IN (0,1)),
    es_sistema   INTEGER NOT NULL DEFAULT 0 CHECK (es_sistema IN (0,1)),
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE UNIQUE INDEX ux_tipos_movimiento_nombre ON tipos_movimiento (nombre) WHERE is_deleted = 0;
CREATE INDEX ix_tipos_movimiento_is_deleted ON tipos_movimiento (is_deleted);

CREATE TABLE tipos_concepto_pago (
    id           TEXT    NOT NULL PRIMARY KEY,
    nombre       TEXT    NOT NULL,
    es_sistema   INTEGER NOT NULL DEFAULT 0 CHECK (es_sistema IN (0,1)),
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL
);
CREATE UNIQUE INDEX ux_tipos_concepto_pago_nombre ON tipos_concepto_pago (nombre) WHERE is_deleted = 0;
CREATE INDEX ix_tipos_concepto_pago_is_deleted ON tipos_concepto_pago (is_deleted);

CREATE TABLE categorias (
    id                 TEXT    NOT NULL PRIMARY KEY,
    nombre             TEXT    NOT NULL,
    descripcion        TEXT        NULL,
    color_hex          TEXT        NULL,
    icono              TEXT        NULL,
    categoria_padre_id TEXT        NULL,
    created_at         TEXT    NOT NULL,
    updated_at         TEXT        NULL,
    row_version        BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted         INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at         TEXT        NULL,
    CONSTRAINT fk_categorias_padre FOREIGN KEY (categoria_padre_id)
        REFERENCES categorias (id) ON DELETE RESTRICT
);
CREATE INDEX ix_categorias_categoria_padre_id ON categorias (categoria_padre_id);
CREATE UNIQUE INDEX ux_categorias_nombre_padre
    ON categorias (nombre, IFNULL(categoria_padre_id, '')) WHERE is_deleted = 0;
CREATE INDEX ix_categorias_is_deleted ON categorias (is_deleted);

CREATE TABLE clientes (
    id            TEXT    NOT NULL PRIMARY KEY,
    nombre        TEXT    NOT NULL,
    cuit          TEXT        NULL,
    direccion     TEXT        NULL,
    telefono      TEXT        NULL,
    email         TEXT        NULL,
    condicion_iva TEXT        NULL,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT        NULL,
    row_version   BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted    INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at    TEXT        NULL
);
CREATE INDEX ix_clientes_cuit ON clientes (cuit);
CREATE INDEX ix_clientes_nombre ON clientes (nombre);
CREATE INDEX ix_clientes_is_deleted ON clientes (is_deleted);

CREATE TABLE cliente_contactos (
    id           TEXT    NOT NULL PRIMARY KEY,
    cliente_id   TEXT    NOT NULL,
    etiqueta     TEXT    NOT NULL DEFAULT 'General',
    email        TEXT    NOT NULL,
    nombre       TEXT        NULL,
    telefono     TEXT        NULL,
    es_principal INTEGER NOT NULL DEFAULT 0 CHECK (es_principal IN (0,1)),
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL,
    CONSTRAINT fk_cliente_contactos_cliente FOREIGN KEY (cliente_id)
        REFERENCES clientes (id) ON DELETE CASCADE
);
CREATE INDEX ix_cliente_contactos_cliente_id ON cliente_contactos (cliente_id);
CREATE UNIQUE INDEX ux_cliente_contactos_cliente_email
    ON cliente_contactos (cliente_id, email) WHERE is_deleted = 0;
CREATE INDEX ix_cliente_contactos_is_deleted ON cliente_contactos (is_deleted);

CREATE TABLE proyectos (
    id          TEXT    NOT NULL PRIMARY KEY,
    numero      INTEGER NOT NULL,
    nombre      TEXT    NOT NULL,
    direccion   TEXT        NULL,
    localidad   TEXT        NULL,
    cliente_id  TEXT    NOT NULL,
    estado      INTEGER NOT NULL DEFAULT 0 CHECK (estado BETWEEN 0 AND 3),
    created_at  TEXT    NOT NULL,
    updated_at  TEXT        NULL,
    row_version BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted  INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at  TEXT        NULL,
    CONSTRAINT fk_proyectos_cliente FOREIGN KEY (cliente_id)
        REFERENCES clientes (id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX ux_proyectos_numero ON proyectos (numero);
CREATE INDEX ix_proyectos_cliente_id ON proyectos (cliente_id);
CREATE INDEX ix_proyectos_estado ON proyectos (estado);
CREATE INDEX ix_proyectos_is_deleted ON proyectos (is_deleted);

CREATE TABLE trabajos (
    id           TEXT    NOT NULL PRIMARY KEY,
    proyecto_id      TEXT    NOT NULL,
    descripcion  TEXT    NOT NULL,
    fecha_inicio TEXT    NOT NULL,
    fecha_fin    TEXT        NULL,
    presupuesto  INTEGER NOT NULL DEFAULT 0,
    estado       INTEGER NOT NULL DEFAULT 0 CHECK (estado BETWEEN 0 AND 4),
    created_at   TEXT    NOT NULL,
    updated_at   TEXT        NULL,
    row_version  BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted   INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at   TEXT        NULL,
    CONSTRAINT fk_trabajos_proyecto FOREIGN KEY (proyecto_id)
        REFERENCES proyectos (id) ON DELETE RESTRICT
);
CREATE INDEX ix_trabajos_proyecto_id ON trabajos (proyecto_id);
CREATE INDEX ix_trabajos_estado ON trabajos (estado);
CREATE INDEX ix_trabajos_fecha_inicio ON trabajos (fecha_inicio);
CREATE INDEX ix_trabajos_is_deleted ON trabajos (is_deleted);

CREATE TABLE ordenes_trabajo (
    id                      TEXT    NOT NULL PRIMARY KEY,
    trabajo_id              TEXT    NOT NULL,
    titulo                  TEXT    NOT NULL,
    numero_certificado      TEXT        NULL,
    fecha                   TEXT    NOT NULL,
    observaciones           TEXT        NULL,
    ajuste_uocra_porcentaje INTEGER NOT NULL DEFAULT 0,
    otros_descuentos        INTEGER NOT NULL DEFAULT 0,
    created_at              TEXT    NOT NULL,
    updated_at              TEXT        NULL,
    row_version             BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted              INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at              TEXT        NULL,
    CONSTRAINT fk_ordenes_trabajo_trabajo FOREIGN KEY (trabajo_id)
        REFERENCES trabajos (id) ON DELETE CASCADE
);
CREATE INDEX ix_ordenes_trabajo_trabajo_id ON ordenes_trabajo (trabajo_id);
CREATE INDEX ix_ordenes_trabajo_fecha ON ordenes_trabajo (fecha);
CREATE INDEX ix_ordenes_trabajo_is_deleted ON ordenes_trabajo (is_deleted);

CREATE TABLE orden_trabajo_items (
    id                  TEXT    NOT NULL PRIMARY KEY,
    orden_trabajo_id    TEXT    NOT NULL,
    descripcion         TEXT    NOT NULL,
    unidad              TEXT    NOT NULL DEFAULT 'u',
    cantidad            INTEGER NOT NULL DEFAULT 0,
    precio_unitario     INTEGER NOT NULL DEFAULT 0,
    porcentaje_anterior INTEGER NOT NULL DEFAULT 0,
    porcentaje_actual   INTEGER NOT NULL DEFAULT 0,
    ejecutado           INTEGER NOT NULL DEFAULT 0 CHECK (ejecutado IN (0,1)),
    nota                TEXT        NULL,
    orden               INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT        NULL,
    row_version         BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted          INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at          TEXT        NULL,
    CONSTRAINT fk_orden_trabajo_items_orden FOREIGN KEY (orden_trabajo_id)
        REFERENCES ordenes_trabajo (id) ON DELETE CASCADE
);
CREATE INDEX ix_orden_trabajo_items_orden_trabajo_id ON orden_trabajo_items (orden_trabajo_id);
CREATE INDEX ix_orden_trabajo_items_is_deleted ON orden_trabajo_items (is_deleted);

CREATE TABLE facturas (
    id                TEXT    NOT NULL PRIMARY KEY,
    numero            TEXT    NOT NULL,
    fecha             TEXT    NOT NULL,
    fecha_vencimiento TEXT        NULL,
    cliente_id        TEXT    NOT NULL,
    estado            INTEGER NOT NULL DEFAULT 0 CHECK (estado BETWEEN 0 AND 5),
    subtotal          INTEGER NOT NULL DEFAULT 0,
    iva               INTEGER NOT NULL DEFAULT 0,
    total             INTEGER NOT NULL DEFAULT 0,
    observaciones     TEXT        NULL,
    created_at        TEXT    NOT NULL,
    updated_at        TEXT        NULL,
    row_version       BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted        INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at        TEXT        NULL,
    CONSTRAINT fk_facturas_cliente FOREIGN KEY (cliente_id)
        REFERENCES clientes (id) ON DELETE RESTRICT
);
CREATE INDEX ix_facturas_cliente_id ON facturas (cliente_id);
CREATE INDEX ix_facturas_fecha ON facturas (fecha);
CREATE INDEX ix_facturas_numero ON facturas (numero);
CREATE INDEX ix_facturas_estado ON facturas (estado);
CREATE INDEX ix_facturas_is_deleted ON facturas (is_deleted);

CREATE TABLE pagos_factura (
    id          TEXT    NOT NULL PRIMARY KEY,
    factura_id  TEXT    NOT NULL,
    fecha       TEXT    NOT NULL,
    monto       INTEGER NOT NULL DEFAULT 0,
    medio_pago  TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    updated_at  TEXT        NULL,
    row_version BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted  INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at  TEXT        NULL,
    CONSTRAINT fk_pagos_factura_factura FOREIGN KEY (factura_id)
        REFERENCES facturas (id) ON DELETE CASCADE
);
CREATE INDEX ix_pagos_factura_factura_id ON pagos_factura (factura_id);
CREATE INDEX ix_pagos_factura_fecha ON pagos_factura (fecha);
CREATE INDEX ix_pagos_factura_is_deleted ON pagos_factura (is_deleted);

CREATE TABLE empleados (
    id              TEXT    NOT NULL PRIMARY KEY,
    nombre          TEXT    NOT NULL,
    dni             TEXT        NULL,
    cargo           TEXT        NULL,
    sueldo_base     INTEGER NOT NULL DEFAULT 0,
    pago_frecuencia INTEGER NOT NULL DEFAULT 3 CHECK (pago_frecuencia BETWEEN 0 AND 3),
    tarifa_diaria   INTEGER NOT NULL DEFAULT 0,
    multiplicador_sabado  INTEGER NOT NULL DEFAULT 10000,
    multiplicador_domingo INTEGER NOT NULL DEFAULT 10000,
    multiplicador_feriado INTEGER NOT NULL DEFAULT 10000,
    email           TEXT        NULL,
    telefono        TEXT        NULL,
    fecha_ingreso   TEXT    NOT NULL,
    fecha_egreso    TEXT        NULL,
    activo          INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0,1)),
    created_at      TEXT    NOT NULL,
    updated_at      TEXT        NULL,
    row_version     BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted      INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at      TEXT        NULL
);
CREATE INDEX ix_empleados_dni ON empleados (dni);
CREATE INDEX ix_empleados_activo ON empleados (activo);
CREATE INDEX ix_empleados_is_deleted ON empleados (is_deleted);

CREATE TABLE asistencias_empleado (
    id            TEXT    NOT NULL PRIMARY KEY,
    empleado_id   TEXT    NOT NULL,
    fecha         TEXT    NOT NULL,
    tipo_jornada  INTEGER NOT NULL DEFAULT 0 CHECK (tipo_jornada BETWEEN 0 AND 4),
    trabajo_id    TEXT        NULL,
    observaciones TEXT        NULL,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT        NULL,
    row_version   BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted    INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at    TEXT        NULL,
    CONSTRAINT fk_asistencias_empleado_empleado FOREIGN KEY (empleado_id)
        REFERENCES empleados (id) ON DELETE CASCADE,
    CONSTRAINT fk_asistencias_empleado_trabajo FOREIGN KEY (trabajo_id)
        REFERENCES trabajos (id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX ux_asistencias_empleado_empleado_fecha
    ON asistencias_empleado (empleado_id, fecha);
CREATE INDEX ix_asistencias_empleado_trabajo_id ON asistencias_empleado (trabajo_id);
CREATE INDEX ix_asistencias_empleado_fecha ON asistencias_empleado (fecha);
CREATE INDEX ix_asistencias_empleado_is_deleted ON asistencias_empleado (is_deleted);

CREATE TABLE liquidaciones (
    id                     TEXT    NOT NULL PRIMARY KEY,
    empleado_id            TEXT    NOT NULL,
    fecha_inicio           TEXT    NOT NULL,
    fecha_fin              TEXT    NOT NULL,
    dias_trabajados        INTEGER NOT NULL DEFAULT 0,
    tarifa_aplicada        INTEGER NOT NULL DEFAULT 0,
    incluir_sabados        INTEGER NOT NULL DEFAULT 0 CHECK (incluir_sabados IN (0,1)),
    incluir_domingos       INTEGER NOT NULL DEFAULT 0 CHECK (incluir_domingos IN (0,1)),
    incluir_feriados       INTEGER NOT NULL DEFAULT 0 CHECK (incluir_feriados IN (0,1)),
    multiplicador_sabado   INTEGER NOT NULL DEFAULT 10000,
    multiplicador_domingo  INTEGER NOT NULL DEFAULT 10000,
    multiplicador_feriado  INTEGER NOT NULL DEFAULT 10000,
    total_bruto            INTEGER NOT NULL DEFAULT 0,
    total_adelantos        INTEGER NOT NULL DEFAULT 0,
    observaciones          TEXT        NULL,
    pdf_generado_at        TEXT        NULL,
    created_at             TEXT    NOT NULL,
    updated_at             TEXT        NULL,
    row_version            BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted             INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at             TEXT        NULL,
    CONSTRAINT fk_liquidaciones_empleado FOREIGN KEY (empleado_id)
        REFERENCES empleados (id) ON DELETE CASCADE
);
CREATE INDEX ix_liquidaciones_empleado_id ON liquidaciones (empleado_id);
CREATE INDEX ix_liquidaciones_fecha_inicio ON liquidaciones (fecha_inicio);
CREATE INDEX ix_liquidaciones_is_deleted ON liquidaciones (is_deleted);

CREATE TABLE movimientos (
    id                     TEXT    NOT NULL PRIMARY KEY,
    fecha                  TEXT    NOT NULL,
    concepto               TEXT    NOT NULL,
    monto                  INTEGER NOT NULL DEFAULT 0,
    cantidad               INTEGER NOT NULL DEFAULT 10000,
    tipo_movimiento_id     TEXT    NOT NULL,
    moneda                 INTEGER NOT NULL DEFAULT 0 CHECK (moneda BETWEEN 0 AND 1),
    cotizacion_aplicada    INTEGER     NULL,
    tipo_concepto_pago_id  TEXT        NULL,
    categoria_id           TEXT        NULL,
    cliente_id             TEXT        NULL,
    trabajo_id             TEXT        NULL,
    empleado_id            TEXT        NULL,
    factura_id             TEXT        NULL,
    created_at             TEXT    NOT NULL,
    updated_at             TEXT        NULL,
    row_version            BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted             INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at             TEXT        NULL,
    CONSTRAINT fk_movimientos_tipo_movimiento FOREIGN KEY (tipo_movimiento_id)
        REFERENCES tipos_movimiento (id) ON DELETE RESTRICT,
    CONSTRAINT fk_movimientos_categoria FOREIGN KEY (categoria_id)
        REFERENCES categorias (id) ON DELETE RESTRICT,
    CONSTRAINT fk_movimientos_tipo_concepto_pago FOREIGN KEY (tipo_concepto_pago_id)
        REFERENCES tipos_concepto_pago (id) ON DELETE SET NULL,
    CONSTRAINT fk_movimientos_factura FOREIGN KEY (factura_id)
        REFERENCES facturas (id) ON DELETE SET NULL,
    CONSTRAINT fk_movimientos_cliente FOREIGN KEY (cliente_id)
        REFERENCES clientes (id) ON DELETE SET NULL,
    CONSTRAINT fk_movimientos_trabajo FOREIGN KEY (trabajo_id)
        REFERENCES trabajos (id) ON DELETE SET NULL,
    CONSTRAINT fk_movimientos_empleado FOREIGN KEY (empleado_id)
        REFERENCES empleados (id) ON DELETE SET NULL
);
CREATE INDEX ix_movimientos_fecha ON movimientos (fecha);
CREATE INDEX ix_movimientos_tipo_movimiento_id ON movimientos (tipo_movimiento_id);
CREATE INDEX ix_movimientos_categoria_id ON movimientos (categoria_id);
CREATE INDEX ix_movimientos_cliente_id ON movimientos (cliente_id);
CREATE INDEX ix_movimientos_trabajo_id ON movimientos (trabajo_id);
CREATE INDEX ix_movimientos_empleado_id ON movimientos (empleado_id);
CREATE INDEX ix_movimientos_factura_id ON movimientos (factura_id);
CREATE INDEX ix_movimientos_tipo_concepto_pago_id ON movimientos (tipo_concepto_pago_id);
CREATE INDEX ix_movimientos_is_deleted ON movimientos (is_deleted);
CREATE INDEX ix_movimientos_empleado_tipo_fecha
    ON movimientos (empleado_id, tipo_movimiento_id, fecha);

CREATE TABLE adjuntos (
    id             TEXT    NOT NULL PRIMARY KEY,
    entidad_tipo   TEXT    NOT NULL,
    entidad_id     TEXT    NOT NULL,
    nombre_archivo TEXT    NOT NULL,
    ruta_relativa  TEXT    NOT NULL,
    mime           TEXT    NOT NULL,
    tamano         INTEGER NOT NULL,
    created_at     TEXT    NOT NULL,
    updated_at     TEXT        NULL,
    row_version    BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted     INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at     TEXT        NULL
);
CREATE INDEX ix_adjuntos_entidad ON adjuntos (entidad_tipo, entidad_id);
CREATE INDEX ix_adjuntos_is_deleted ON adjuntos (is_deleted);

CREATE TABLE app_metadata (
    key        TEXT NOT NULL PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE certificados (
    id                TEXT    NOT NULL PRIMARY KEY,
    orden_trabajo_id  TEXT    NOT NULL,
    numero            INTEGER NOT NULL,
    fecha             TEXT    NOT NULL,
    observaciones     TEXT        NULL,
    total_certificado INTEGER NOT NULL DEFAULT 0,
    ajuste_uocra      INTEGER NOT NULL DEFAULT 0,
    otros_descuentos  INTEGER NOT NULL DEFAULT 0,
    total_neto        INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT    NOT NULL,
    updated_at        TEXT        NULL,
    row_version       BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted        INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at        TEXT        NULL,
    CONSTRAINT fk_certificados_orden FOREIGN KEY (orden_trabajo_id)
        REFERENCES ordenes_trabajo (id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX ux_certificados_orden_numero
    ON certificados (orden_trabajo_id, numero);
CREATE INDEX ix_certificados_fecha ON certificados (fecha);
CREATE INDEX ix_certificados_is_deleted ON certificados (is_deleted);

CREATE TABLE certificado_items (
    id                    TEXT    NOT NULL PRIMARY KEY,
    certificado_id        TEXT    NOT NULL,
    orden_trabajo_item_id TEXT    NOT NULL,
    cantidad              INTEGER NOT NULL DEFAULT 0,
    precio_unitario       INTEGER NOT NULL DEFAULT 0,
    porcentaje_anterior   INTEGER NOT NULL DEFAULT 0,
    porcentaje_actual     INTEGER NOT NULL DEFAULT 0,
    subtotal_actual       INTEGER NOT NULL DEFAULT 0,
    subtotal_acumulado    INTEGER NOT NULL DEFAULT 0,
    created_at            TEXT    NOT NULL,
    updated_at            TEXT        NULL,
    row_version           BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted            INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at            TEXT        NULL,
    CONSTRAINT fk_certificado_items_certificado FOREIGN KEY (certificado_id)
        REFERENCES certificados (id) ON DELETE CASCADE,
    CONSTRAINT fk_certificado_items_item FOREIGN KEY (orden_trabajo_item_id)
        REFERENCES orden_trabajo_items (id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX ux_certificado_items_certificado_item
    ON certificado_items (certificado_id, orden_trabajo_item_id);
CREATE INDEX ix_certificado_items_orden_trabajo_item_id
    ON certificado_items (orden_trabajo_item_id);
CREATE INDEX ix_certificado_items_is_deleted ON certificado_items (is_deleted);

CREATE TABLE liquidacion_adelantos (
    id             TEXT    NOT NULL PRIMARY KEY,
    liquidacion_id TEXT    NOT NULL,
    movimiento_id  TEXT    NOT NULL,
    monto          INTEGER NOT NULL DEFAULT 0,
    fecha          TEXT    NOT NULL,
    concepto       TEXT    NOT NULL,
    created_at     TEXT    NOT NULL,
    updated_at     TEXT        NULL,
    row_version    BLOB    NOT NULL DEFAULT X'0000000000000001',
    is_deleted     INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0,1)),
    deleted_at     TEXT        NULL,
    CONSTRAINT fk_liquidacion_adelantos_liquidacion FOREIGN KEY (liquidacion_id)
        REFERENCES liquidaciones (id) ON DELETE CASCADE,
    CONSTRAINT fk_liquidacion_adelantos_movimiento FOREIGN KEY (movimiento_id)
        REFERENCES movimientos (id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX ux_liquidacion_adelantos_movimiento
    ON liquidacion_adelantos (movimiento_id) WHERE is_deleted = 0;
CREATE INDEX ix_liquidacion_adelantos_liquidacion_id
    ON liquidacion_adelantos (liquidacion_id);
CREATE INDEX ix_liquidacion_adelantos_is_deleted ON liquidacion_adelantos (is_deleted);

CREATE TABLE feriados (
    fecha      TEXT NOT NULL PRIMARY KEY,
    nombre     TEXT NOT NULL,
    tipo       TEXT     NULL,
    origen     TEXT NOT NULL CHECK (origen IN ('Api','Manual')),
    created_at TEXT NOT NULL,
    updated_at TEXT     NULL
) WITHOUT ROWID;
CREATE INDEX ix_feriados_origen ON feriados (origen);
