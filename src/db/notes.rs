use anyhow::Result;
use rusqlite::{params, Connection};

use crate::models::Note;

pub fn add(conn: &Connection, person_id: i64, content: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO notes (person_id, content) VALUES (?1, ?2)",
        params![person_id, content],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_for_person(conn: &Connection, person_id: i64) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, person_id, content, created_at FROM notes WHERE person_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![person_id], |row| {
        Ok(Note {
            id: row.get(0)?,
            person_id: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
