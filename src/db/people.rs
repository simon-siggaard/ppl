use anyhow::{bail, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::{params, Connection};

use crate::models::Person;

pub fn insert(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute("INSERT INTO people (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
pub fn insert_full(
    conn: &Connection,
    name: &str,
    nickname: Option<&str>,
    email: Option<&str>,
    phone: Option<&str>,
    company: Option<&str>,
    team: Option<&str>,
    department: Option<&str>,
    job_title: Option<&str>,
    birthday: Option<&str>,
    employment_date: Option<&str>,
    created_at: &str,
    updated_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO people (name, nickname, email, phone, company, team, department, job_title, birthday, employment_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![name, nickname, email, phone, company, team, department, job_title, birthday, employment_date, created_at, updated_at],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_field(conn: &Connection, id: i64, field: &str, value: Option<&str>) -> Result<()> {
    let allowed = [
        "name",
        "nickname",
        "email",
        "phone",
        "company",
        "team",
        "department",
        "job_title",
        "birthday",
        "employment_date",
    ];
    if !allowed.contains(&field) {
        bail!("Invalid field: {field}");
    }
    let sql = format!(
        "UPDATE people SET {field} = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?2"
    );
    conn.execute(&sql, params![value, id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM people WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Person> {
    let p = conn.query_row("SELECT * FROM people WHERE id = ?1", params![id], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            nickname: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            company: row.get(5)?,
            team: row.get(6)?,
            department: row.get(7)?,
            job_title: row.get(8)?,
            birthday: row.get(9)?,
            employment_date: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    Ok(p)
}

pub fn list_all(conn: &Connection, sort: &str) -> Result<Vec<Person>> {
    let order = match sort {
        "created" => "created_at DESC",
        "company" => "company ASC, name ASC",
        _ => "name ASC",
    };
    let sql = format!("SELECT * FROM people ORDER BY {order}");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            nickname: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            company: row.get(5)?,
            team: row.get(6)?,
            department: row.get(7)?,
            job_title: row.get(8)?,
            birthday: row.get(9)?,
            employment_date: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_by_tag(conn: &Connection, tag: &str, sort: &str) -> Result<Vec<Person>> {
    let order = match sort {
        "created" => "p.created_at DESC",
        "company" => "p.company ASC, p.name ASC",
        _ => "p.name ASC",
    };
    let sql = format!(
        "SELECT p.* FROM people p
         JOIN person_tags pt ON pt.person_id = p.id
         JOIN tags t ON t.id = pt.tag_id
         WHERE t.name = ?1
         ORDER BY {order}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![tag], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            nickname: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            company: row.get(5)?,
            team: row.get(6)?,
            department: row.get(7)?,
            job_title: row.get(8)?,
            birthday: row.get(9)?,
            employment_date: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_by_company(conn: &Connection, company: &str, sort: &str) -> Result<Vec<Person>> {
    let order = match sort {
        "created" => "created_at DESC",
        "company" => "company ASC, name ASC",
        _ => "name ASC",
    };
    let sql = format!(
        "SELECT * FROM people WHERE company LIKE ?1 ORDER BY {order}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let pattern = format!("%{company}%");
    let rows = stmt.query_map(params![pattern], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            nickname: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            company: row.get(5)?,
            team: row.get(6)?,
            department: row.get(7)?,
            job_title: row.get(8)?,
            birthday: row.get(9)?,
            employment_date: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Resolve a name query to a single person ID.
/// Exact match → case-insensitive partial → fuzzy.
/// Returns Vec of (id, name) matches for disambiguation.
pub fn resolve_name(conn: &Connection, query: &str) -> Result<Vec<(i64, String)>> {
    // Exact match
    let mut stmt = conn.prepare("SELECT id, name FROM people WHERE name = ?1")?;
    let exact: Vec<(i64, String)> = stmt
        .query_map(params![query], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    if !exact.is_empty() {
        return Ok(exact);
    }

    // Case-insensitive partial
    let pattern = format!("%{query}%");
    let mut stmt =
        conn.prepare("SELECT id, name FROM people WHERE name LIKE ?1 COLLATE NOCASE")?;
    let partial: Vec<(i64, String)> = stmt
        .query_map(params![pattern], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    if !partial.is_empty() {
        return Ok(partial);
    }

    // Fuzzy match
    let mut stmt = conn.prepare("SELECT id, name FROM people")?;
    let all: Vec<(i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(i64, String, i64)> = all
        .into_iter()
        .filter_map(|(id, name)| {
            matcher
                .fuzzy_match(&name, query)
                .map(|score| (id, name, score))
        })
        .collect();
    scored.sort_by(|a, b| b.2.cmp(&a.2));

    Ok(scored.into_iter().map(|(id, name, _)| (id, name)).collect())
}
