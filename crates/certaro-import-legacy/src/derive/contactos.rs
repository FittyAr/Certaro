use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use crate::report::ImportReport;

/// Derives cliente_contactos from Clientes.Email.
/// Returns the count of new contacts created.
#[allow(unused_variables)]
pub async fn derive_contactos(db: &DatabaseTransaction, report: &mut ImportReport) -> Result<u64> {
    // Find clients with an email that doesn't already have a matching contact.
    let clientes = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT c.id, c.email, c.created_at FROM clientes c              WHERE c.email IS NOT NULL AND c.email != '' AND c.is_deleted = 0"
                .to_owned(),
        ))
        .await
        .context("getting clients with email")?;

    let mut count = 0u64;

    for cliente in &clientes {
        let cliente_id: String = cliente.try_get("", "id").unwrap_or_default();
        let email: String = cliente.try_get("", "email").unwrap_or_default();
        let created_at: String = cliente.try_get("", "created_at").unwrap_or_default();

        // Check if a contact with this email already exists.
        let existing = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "SELECT id FROM cliente_contactos                      WHERE cliente_id = '{}' AND LOWER(email) = LOWER('{}') AND is_deleted = 0",
                    cliente_id.replace('\'', "''"),
                    email.replace('\'', "''"),
                ),
            ))
            .await
            .context("checking existing contact")?;

        if existing.is_some() {
            // Mark existing as principal.
            db.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "UPDATE cliente_contactos SET es_principal = 1                      WHERE cliente_id = '{}' AND LOWER(email) = LOWER('{}') AND is_deleted = 0",
                    cliente_id.replace('\'', "''"),
                    email.replace('\'', "''"),
                ),
            ))
            .await
            .context("marking contact as principal")?;
        } else {
            // Create new contact.
            let contacto_id = uuid::Uuid::now_v7().to_string();
            let sql = format!(
                "INSERT INTO cliente_contactos (id, cliente_id, email, etiqueta, nombre, telefono,                  es_principal, created_at, updated_at, row_version, is_deleted)                  VALUES ('{}', '{}', '{}', 'Principal', NULL, NULL, 1, '{}', '{}', X'0000000000000001', 0)",
                contacto_id,
                cliente_id.replace('\'', "''"),
                email.replace('\'', "''"),
                created_at,
                chrono::Utc::now().to_rfc3339(),
            );
            db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
                .await
                .context("inserting derived contact")?;
            count += 1;
        }
    }

    // Ensure every client with contacts has at least one principal.
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "UPDATE cliente_contactos SET es_principal = 1          WHERE rowid IN (              SELECT cc.rowid FROM cliente_contactos cc              WHERE cc.is_deleted = 0 AND cc.es_principal = 0              AND NOT EXISTS (                  SELECT 1 FROM cliente_contactos cc2                  WHERE cc2.cliente_id = cc.cliente_id AND cc2.es_principal = 1 AND cc2.is_deleted = 0              )              AND cc.created_at = (                  SELECT MIN(cc3.created_at) FROM cliente_contactos cc3                  WHERE cc3.cliente_id = cc.cliente_id AND cc3.is_deleted = 0              )          )"
            .to_owned(),
    ))
    .await
    .context("ensuring principal contacts")?;

    Ok(count)
}
