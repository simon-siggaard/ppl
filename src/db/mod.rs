pub mod custom_fields;
pub mod dates;
pub mod notes;
pub mod people;
pub mod relationships;
pub mod tags;

use anyhow::Result;
use rusqlite::Connection;

pub fn open(path: &std::path::Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS people (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL,
            nickname        TEXT,
            email           TEXT,
            phone           TEXT,
            company         TEXT,
            team            TEXT,
            department      TEXT,
            job_title       TEXT,
            birthday        TEXT,
            employment_date TEXT,
            created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        CREATE TABLE IF NOT EXISTS relationships (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            rel_type  TEXT NOT NULL,
            name      TEXT NOT NULL,
            birthday  TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        CREATE TABLE IF NOT EXISTS tags (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS person_tags (
            person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            tag_id    INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (person_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS notes (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            content   TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        CREATE TABLE IF NOT EXISTS custom_fields (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
            key       TEXT NOT NULL,
            value     TEXT NOT NULL,
            UNIQUE(person_id, key)
        );

        CREATE VIEW IF NOT EXISTS date_events AS
            SELECT p.id AS person_id, p.name AS person_name, 'birthday' AS event_type,
                   p.birthday AS event_date, p.name AS subject_name
            FROM people p WHERE p.birthday IS NOT NULL
            UNION ALL
            SELECT p.id, p.name, 'employment_anniversary',
                   p.employment_date, p.name
            FROM people p WHERE p.employment_date IS NOT NULL
            UNION ALL
            SELECT r.person_id, p.name, r.rel_type || '_birthday',
                   r.birthday, r.name
            FROM relationships r JOIN people p ON p.id = r.person_id
            WHERE r.birthday IS NOT NULL;
        ",
    )?;
    Ok(())
}
