mod cli;
mod config;
mod db;
mod display;
mod interactive;
mod models;
mod search;

use anyhow::{bail, Context, Result};
use clap::Parser;

use cli::{Cli, Command};
use display::DisplayConfig;
use models::PersonDetail;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = config::db_path(cli.db.as_deref())?;
    let conn = db::open(&db_path)?;
    let dcfg = DisplayConfig::new(cli.no_color);

    match cli.command {
        Command::Add {
            name,
            nickname,
            email,
            phone,
            company,
            team,
            department,
            job_title,
            birthday,
            employment_date,
        } => {
            let has_flags = nickname.is_some()
                || email.is_some()
                || phone.is_some()
                || company.is_some()
                || team.is_some()
                || department.is_some()
                || job_title.is_some()
                || birthday.is_some()
                || employment_date.is_some();

            let fields = if has_flags {
                interactive::PersonFields {
                    name: Some(name.clone()),
                    nickname,
                    email,
                    phone,
                    company,
                    team,
                    department,
                    job_title,
                    birthday,
                    employment_date,
                }
            } else {
                interactive::prompt_add(&name)?
            };

            let id = db::people::insert(&conn, fields.name.as_deref().unwrap_or(&name))?;
            set_fields(&conn, id, &fields)?;

            if cli.json {
                let person = db::people::get_by_id(&conn, id)?;
                println!("{}", serde_json::to_string_pretty(&person)?);
            } else {
                println!("Added {} (id {id}).", fields.name.as_deref().unwrap_or(&name));
            }
        }

        Command::Edit {
            name,
            nickname,
            email,
            phone,
            company,
            team,
            department,
            job_title,
            birthday,
            employment_date,
            set_name,
        } => {
            let (id, _resolved) = resolve_person(&conn, name.as_deref())?;

            let has_flags = nickname.is_some()
                || email.is_some()
                || phone.is_some()
                || company.is_some()
                || team.is_some()
                || department.is_some()
                || job_title.is_some()
                || birthday.is_some()
                || employment_date.is_some()
                || set_name.is_some();

            if has_flags {
                let fields = interactive::PersonFields {
                    name: set_name,
                    nickname,
                    email,
                    phone,
                    company,
                    team,
                    department,
                    job_title,
                    birthday,
                    employment_date,
                };
                set_fields(&conn, id, &fields)?;
            } else {
                let person = db::people::get_by_id(&conn, id)?;
                let fields = interactive::prompt_edit(&person)?;
                set_fields(&conn, id, &fields)?;
            }

            let person = db::people::get_by_id(&conn, id)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&person)?);
            } else {
                println!("Updated {}.", person.name);
            }
        }

        Command::Show { name } => {
            let (id, _) = resolve_person(&conn, name.as_deref())?;
            let detail = load_detail(&conn, id)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&detail)?);
            } else {
                display::show_person(&detail, &dcfg);
            }
        }

        Command::List { tag, company, sort } => {
            let people = match (&tag, &company) {
                (Some(t), _) => db::people::list_by_tag(&conn, t, &sort)?,
                (_, Some(c)) => db::people::list_by_company(&conn, c, &sort)?,
                _ => db::people::list_all(&conn, &sort)?,
            };
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&people)?);
            } else {
                display::list_people(&people, &dcfg);
            }
        }

        Command::Rm { name } => {
            let (id, resolved) = resolve_person(&conn, name.as_deref())?;
            let confirm = inquire::Confirm::new(&format!("Delete {resolved}?"))
                .with_default(false)
                .prompt()?;
            if confirm {
                db::people::delete(&conn, id)?;
                println!("Deleted {resolved}.");
            } else {
                println!("Cancelled.");
            }
        }

        Command::Note { name, text } => {
            let (id, resolved) = resolve_person(&conn, name.as_deref())?;
            let text = match text {
                Some(t) => t,
                None => inquire::Text::new("Note:").prompt()?,
            };
            db::notes::add(&conn, id, &text)?;
            if cli.json {
                let notes = db::notes::list_for_person(&conn, id)?;
                println!("{}", serde_json::to_string_pretty(&notes)?);
            } else {
                println!("Note added to {resolved}.");
            }
        }

        Command::Tag { name, tag } => {
            let (id, resolved) = resolve_person(&conn, name.as_deref())?;
            let tag = match tag {
                Some(t) => t,
                None => inquire::Text::new("Tag:").prompt()?,
            };
            db::tags::add_tag(&conn, id, &tag)?;
            println!("Tagged {resolved} with \"{tag}\".");
        }

        Command::Untag { name, tag } => {
            let (id, resolved) = resolve_person(&conn, name.as_deref())?;
            let tag = match tag {
                Some(t) => t,
                None => {
                    let tags = db::tags::list_for_person(&conn, id)?;
                    if tags.is_empty() {
                        bail!("{resolved} has no tags.");
                    }
                    inquire::Select::new("Select tag to remove:", tags).prompt()?
                }
            };
            if db::tags::remove_tag(&conn, id, &tag)? {
                println!("Removed tag \"{tag}\" from {resolved}.");
            } else {
                println!("{resolved} doesn't have tag \"{tag}\".");
            }
        }

        Command::Dates { range } => {
            let dr = db::dates::DateRange::parse(&range)?;
            let events = db::dates::query_events(&conn, &dr)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&events)?);
            } else {
                display::show_dates(&events, &range, &dcfg);
            }
        }

        Command::Search { query } => {
            let results = search::search(&conn, &query)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else if results.is_empty() {
                println!("No results for \"{query}\".");
            } else {
                println!("Search results for \"{query}\":\n");
                for r in &results {
                    println!("  {} — {}", r.person_name, r.match_context);
                }
            }
        }

        Command::Export { path } => {
            let people = db::people::list_all(&conn, "name")?;
            let mut details: Vec<PersonDetail> = Vec::new();
            for p in &people {
                details.push(load_detail(&conn, p.id)?);
            }
            let json = serde_json::to_string_pretty(&details)?;
            match path {
                Some(p) => {
                    std::fs::write(&p, &json)
                        .with_context(|| format!("Failed to write {p}"))?;
                    println!("Exported {} people to {p}.", details.len());
                }
                None => println!("{json}"),
            }
        }

        Command::Import { path } => {
            let data = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {path}"))?;
            let details: Vec<PersonDetail> = serde_json::from_str(&data)
                .with_context(|| "Failed to parse JSON")?;
            let mut count = 0;
            for detail in &details {
                let p = &detail.person;
                let id = db::people::insert_full(
                    &conn, &p.name, p.nickname.as_deref(), p.email.as_deref(),
                    p.phone.as_deref(), p.company.as_deref(), p.team.as_deref(),
                    p.department.as_deref(), p.job_title.as_deref(),
                    p.birthday.as_deref(), p.employment_date.as_deref(),
                    &p.created_at, &p.updated_at,
                )?;
                for rel in &detail.relationships {
                    db::relationships::add(
                        &conn, id, &rel.rel_type, &rel.name,
                        rel.birthday.as_deref(),
                    )?;
                }
                for tag in &detail.tags {
                    db::tags::add_tag(&conn, id, tag)?;
                }
                for note in &detail.notes {
                    db::notes::add(&conn, id, &note.content)?;
                }
                for cf in &detail.custom_fields {
                    db::custom_fields::set(&conn, id, &cf.key, &cf.value)?;
                }
                count += 1;
            }
            println!("Imported {count} people.");
        }
    }

    Ok(())
}

fn resolve_person(conn: &rusqlite::Connection, query: Option<&str>) -> Result<(i64, String)> {
    match query {
        Some(q) => {
            let matches = db::people::resolve_name(conn, q)?;
            match matches.len() {
                0 => bail!("No person found matching \"{q}\"."),
                1 => Ok(matches.into_iter().next().unwrap()),
                _ => interactive::disambiguate(&matches),
            }
        }
        None => interactive::pick_person(conn),
    }
}

fn set_fields(
    conn: &rusqlite::Connection,
    id: i64,
    fields: &interactive::PersonFields,
) -> Result<()> {
    let updates: Vec<(&str, &Option<String>)> = vec![
        ("name", &fields.name),
        ("nickname", &fields.nickname),
        ("email", &fields.email),
        ("phone", &fields.phone),
        ("company", &fields.company),
        ("team", &fields.team),
        ("department", &fields.department),
        ("job_title", &fields.job_title),
        ("birthday", &fields.birthday),
        ("employment_date", &fields.employment_date),
    ];
    for (field, value) in updates {
        if let Some(v) = value {
            db::people::update_field(conn, id, field, Some(v))?;
        }
    }
    Ok(())
}

fn load_detail(conn: &rusqlite::Connection, id: i64) -> Result<PersonDetail> {
    let person = db::people::get_by_id(conn, id)?;
    let relationships = db::relationships::list_for_person(conn, id)?;
    let tags = db::tags::list_for_person(conn, id)?;
    let notes = db::notes::list_for_person(conn, id)?;
    let custom_fields = db::custom_fields::list_for_person(conn, id)?;
    Ok(PersonDetail {
        person,
        relationships,
        tags,
        notes,
        custom_fields,
    })
}
