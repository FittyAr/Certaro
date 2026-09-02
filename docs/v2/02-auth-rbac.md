# Certaro v2 - Sistema de Autenticación, Roles y Permisos (RBAC)

## 1. Modelo de Seguridad y Entidades

Certaro v2 cuenta con un sistema empresarial de Control de Acceso Basado en Roles (RBAC) con 7 entidades normalizadas:
- `usuarios`: Cuentas con email único, hash Argon2id, bandera de doble factor (2FA) y auditoría.
- `roles`: Roles configurables (ej. "Administrador", "Operador", "Técnico", "Comercial").
- `permisos`: 39 permisos granulares de lectura, creación, modificación y eliminación por módulo.
- `usuario_roles`: Asociación N:M entre usuarios y roles.
- `rol_permisos`: Asociación N:M entre roles y permisos.
- `sesiones`: Tokens JWT y estado de sesión.
- `auth_externo`: Vinculaciones de inicio de sesión externo/OAuth.

## 2. Regla de Bypass en Modo SQLite

- **Modo SQLite (Local / Desktop autónomo)**:
  - No obliga al usuario a autenticarse.
  - La llamada a `auth_get_mode` retorna `bypassAuth: true`.
  - El store frontend `useAuthStore` asume automáticamente la identidad de Super Admin (`admin@certaro.local`).
  - Todas las comprobaciones `hasPermission(...)` evalúan como `true`.
- **Modo Servidor / Docker (PostgreSQL / MySQL)**:
  - Requiere autenticación mediante pantalla de Login.
  - Generación de token de sesión seguro.
  - Soporte para autenticación en dos pasos (TOTP / Google Authenticator).
  - Los comandos IPC y vistas validan permisos específicos.

## 3. Usuario Inicial Super Admin

En la inicialización del esquema se siembra automáticamente un usuario Super Administrador:
- **Email**: `admin@certaro.local`
- **Contraseña predeterminada**: `admin123` (Argon2id)
- **Rol asignado**: `Administrador` (con los 39 permisos del catálogo activados)
