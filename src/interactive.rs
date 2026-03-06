use anyhow::{bail, Result};
use inquire::{Select, Text};

use crate::db;
use crate::models::Person;

pub struct PersonFields {
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub team: Option<String>,
    pub department: Option<String>,
    pub job_title: Option<String>,
    pub birthday: Option<String>,
    pub employment_date: Option<String>,
}

fn prompt_optional(label: &str, default: Option<&str>) -> Result<Option<String>> {
    let mut p = Text::new(label);
    if let Some(d) = default {
        p = p.with_default(d);
    }
    match p.prompt_skippable()? {
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => Ok(Some(s)),
        None => Ok(default.map(|s| s.to_string())),
    }
}

pub fn prompt_add(name: &str) -> Result<PersonFields> {
    println!("Adding {}. Press Esc to skip a field.\n", name);

    Ok(PersonFields {
        name: Some(name.to_string()),
        nickname: prompt_optional("Nickname:", None)?,
        email: prompt_optional("Email:", None)?,
        phone: prompt_optional("Phone:", None)?,
        company: prompt_optional("Company:", None)?,
        team: prompt_optional("Team:", None)?,
        department: prompt_optional("Department:", None)?,
        job_title: prompt_optional("Job title:", None)?,
        birthday: prompt_optional("Birthday (YYYY-MM-DD):", None)?,
        employment_date: prompt_optional("Employment date (YYYY-MM-DD):", None)?,
    })
}

pub fn prompt_edit(person: &Person) -> Result<PersonFields> {
    println!("Editing {}. Press Esc to keep current value.\n", person.name);

    Ok(PersonFields {
        name: prompt_optional("Name:", Some(&person.name))?,
        nickname: prompt_optional("Nickname:", person.nickname.as_deref())?,
        email: prompt_optional("Email:", person.email.as_deref())?,
        phone: prompt_optional("Phone:", person.phone.as_deref())?,
        company: prompt_optional("Company:", person.company.as_deref())?,
        team: prompt_optional("Team:", person.team.as_deref())?,
        department: prompt_optional("Department:", person.department.as_deref())?,
        job_title: prompt_optional("Job title:", person.job_title.as_deref())?,
        birthday: prompt_optional("Birthday (YYYY-MM-DD):", person.birthday.as_deref())?,
        employment_date: prompt_optional(
            "Employment date (YYYY-MM-DD):",
            person.employment_date.as_deref(),
        )?,
    })
}

pub fn pick_person(conn: &rusqlite::Connection) -> Result<(i64, String)> {
    let people = db::people::list_all(conn, "name")?;
    if people.is_empty() {
        bail!("No people in the database.");
    }
    let options: Vec<String> = people.iter().map(|p| p.name.clone()).collect();
    let ids: Vec<i64> = people.iter().map(|p| p.id).collect();
    let selection = Select::new("Select a person:", options).prompt()?;
    let idx = ids
        .iter()
        .zip(people.iter())
        .position(|(_, p)| p.name == selection)
        .unwrap();
    Ok((ids[idx], selection))
}

pub fn disambiguate(matches: &[(i64, String)]) -> Result<(i64, String)> {
    let options: Vec<String> = matches.iter().map(|(_, n)| n.clone()).collect();
    let selection = inquire::Select::new("Multiple matches found. Select one:", options).prompt()?;
    let idx = matches.iter().position(|(_, n)| *n == selection).unwrap();
    Ok(matches[idx].clone())
}
