use anyhow::Result;
use rusqlite::{params, Connection};

pub fn add_tag(conn: &Connection, person_id: i64, tag_name: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
        params![tag_name],
    )?;
    let tag_id: i64 = conn.query_row(
        "SELECT id FROM tags WHERE name = ?1",
        params![tag_name],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO person_tags (person_id, tag_id) VALUES (?1, ?2)",
        params![person_id, tag_id],
    )?;
    Ok(())
}

pub fn remove_tag(conn: &Connection, person_id: i64, tag_name: &str) -> Result<bool> {
    let tag_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag_name],
            |r| r.get(0),
        )
        .ok();
    if let Some(tid) = tag_id {
        let deleted = conn.execute(
            "DELETE FROM person_tags WHERE person_id = ?1 AND tag_id = ?2",
            params![person_id, tid],
        )?;
        Ok(deleted > 0)
    } else {
        Ok(false)
    }
}

pub fn list_for_person(conn: &Connection, person_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT t.name FROM tags t JOIN person_tags pt ON pt.tag_id = t.id WHERE pt.person_id = ?1 ORDER BY t.name",
    )?;
    let rows = stmt.query_map(params![person_id], |r| r.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
