use anyhow::Result;
use rusqlite::{params, Connection};

use crate::models::Relationship;

pub fn add(
    conn: &Connection,
    person_id: i64,
    rel_type: &str,
    name: &str,
    birthday: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO relationships (person_id, rel_type, name, birthday) VALUES (?1, ?2, ?3, ?4)",
        params![person_id, rel_type, name, birthday],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_for_person(conn: &Connection, person_id: i64) -> Result<Vec<Relationship>> {
    let mut stmt = conn.prepare(
        "SELECT id, person_id, rel_type, name, birthday, created_at FROM relationships WHERE person_id = ?1 ORDER BY rel_type, name",
    )?;
    let rows = stmt.query_map(params![person_id], |row| {
        Ok(Relationship {
            id: row.get(0)?,
            person_id: row.get(1)?,
            rel_type: row.get(2)?,
            name: row.get(3)?,
            birthday: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
