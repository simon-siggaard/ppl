use anyhow::Result;
use rusqlite::{params, Connection};

use crate::models::CustomField;

pub fn set(conn: &Connection, person_id: i64, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO custom_fields (person_id, key, value) VALUES (?1, ?2, ?3)
         ON CONFLICT(person_id, key) DO UPDATE SET value = excluded.value",
        params![person_id, key, value],
    )?;
    Ok(())
}

pub fn list_for_person(conn: &Connection, person_id: i64) -> Result<Vec<CustomField>> {
    let mut stmt = conn.prepare(
        "SELECT id, person_id, key, value FROM custom_fields WHERE person_id = ?1 ORDER BY key",
    )?;
    let rows = stmt.query_map(params![person_id], |row| {
        Ok(CustomField {
            id: row.get(0)?,
            person_id: row.get(1)?,
            key: row.get(2)?,
            value: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
