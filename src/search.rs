use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub person_id: i64,
    pub person_name: String,
    pub match_context: String,
}

pub fn search(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let mut results: Vec<SearchResult> = Vec::new();
    let pattern = format!("%{query}%");

    // Search people fields
    let mut stmt = conn.prepare(
        "SELECT id, name, nickname, email, phone, company, team, department, job_title
         FROM people
         WHERE name LIKE ?1 COLLATE NOCASE
            OR nickname LIKE ?1 COLLATE NOCASE
            OR email LIKE ?1 COLLATE NOCASE
            OR phone LIKE ?1 COLLATE NOCASE
            OR company LIKE ?1 COLLATE NOCASE
            OR team LIKE ?1 COLLATE NOCASE
            OR department LIKE ?1 COLLATE NOCASE
            OR job_title LIKE ?1 COLLATE NOCASE",
    )?;

    let people_matches = stmt.query_map(params![pattern], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        // Find which field matched
        let fields: Vec<(&str, Option<String>)> = vec![
            ("name", Some(name.clone())),
            ("nickname", row.get(2)?),
            ("email", row.get(3)?),
            ("phone", row.get(4)?),
            ("company", row.get(5)?),
            ("team", row.get(6)?),
            ("department", row.get(7)?),
            ("job_title", row.get(8)?),
        ];
        let query_lower = query.to_lowercase();
        let matched: Vec<String> = fields
            .into_iter()
            .filter_map(|(label, val)| {
                val.filter(|v| v.to_lowercase().contains(&query_lower))
                    .map(|v| format!("{label}: {v}"))
            })
            .collect();
        Ok(SearchResult {
            person_id: id,
            person_name: name,
            match_context: matched.join(", "),
        })
    })?;

    for r in people_matches {
        results.push(r?);
    }

    // Search notes
    let mut stmt = conn.prepare(
        "SELECT n.person_id, p.name, n.content FROM notes n
         JOIN people p ON p.id = n.person_id
         WHERE n.content LIKE ?1 COLLATE NOCASE",
    )?;
    let note_matches = stmt.query_map(params![pattern], |row| {
        Ok(SearchResult {
            person_id: row.get(0)?,
            person_name: row.get(1)?,
            match_context: format!("note: {}", row.get::<_, String>(2)?),
        })
    })?;
    for r in note_matches {
        results.push(r?);
    }

    // Search tags
    let mut stmt = conn.prepare(
        "SELECT pt.person_id, p.name, t.name FROM person_tags pt
         JOIN people p ON p.id = pt.person_id
         JOIN tags t ON t.id = pt.tag_id
         WHERE t.name LIKE ?1 COLLATE NOCASE",
    )?;
    let tag_matches = stmt.query_map(params![pattern], |row| {
        Ok(SearchResult {
            person_id: row.get(0)?,
            person_name: row.get(1)?,
            match_context: format!("tag: {}", row.get::<_, String>(2)?),
        })
    })?;
    for r in tag_matches {
        results.push(r?);
    }

    // Search custom fields
    let mut stmt = conn.prepare(
        "SELECT cf.person_id, p.name, cf.key, cf.value FROM custom_fields cf
         JOIN people p ON p.id = cf.person_id
         WHERE cf.key LIKE ?1 COLLATE NOCASE OR cf.value LIKE ?1 COLLATE NOCASE",
    )?;
    let cf_matches = stmt.query_map(params![pattern], |row| {
        Ok(SearchResult {
            person_id: row.get(0)?,
            person_name: row.get(1)?,
            match_context: format!("{}: {}", row.get::<_, String>(2)?, row.get::<_, String>(3)?),
        })
    })?;
    for r in cf_matches {
        results.push(r?);
    }

    // Fuzzy match on names for anything not already found
    let found_ids: std::collections::HashSet<i64> =
        results.iter().map(|r| r.person_id).collect();
    let mut stmt = conn.prepare("SELECT id, name FROM people")?;
    let all: Vec<(i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    let matcher = SkimMatcherV2::default();
    for (id, name) in all {
        if !found_ids.contains(&id) {
            if let Some(score) = matcher.fuzzy_match(&name, query) {
                if score > 50 {
                    results.push(SearchResult {
                        person_id: id,
                        person_name: name,
                        match_context: "fuzzy name match".to_string(),
                    });
                }
            }
        }
    }

    Ok(results)
}
