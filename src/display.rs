use chrono::{Datelike, Local, NaiveDate};
use comfy_table::{Cell, Table};
use owo_colors::OwoColorize;

use crate::models::{DateEvent, Person, PersonDetail};

pub struct DisplayConfig {
    pub color: bool,
}

impl DisplayConfig {
    pub fn new(no_color: bool) -> Self {
        let color = !no_color && std::env::var("NO_COLOR").is_err();
        Self { color }
    }
}

pub fn show_person(detail: &PersonDetail, cfg: &DisplayConfig) {
    let p = &detail.person;

    // Name header
    if cfg.color {
        println!("{}", p.name.bold());
    } else {
        println!("{}", p.name);
    }

    if let Some(nick) = &p.nickname {
        println!("  Nickname:    {nick}");
    }

    // Company line
    match (&p.company, &p.team, &p.department) {
        (Some(c), Some(t), _) => println!("  Company:     {c} ({t})"),
        (Some(c), None, Some(d)) => println!("  Company:     {c} ({d})"),
        (Some(c), None, None) => println!("  Company:     {c}"),
        _ => {}
    }

    if let Some(title) = &p.job_title {
        println!("  Title:       {title}");
    }

    if let Some(bday) = &p.birthday {
        let age_str = compute_age(bday);
        println!("  Birthday:    {bday}{age_str}");
    }

    if let Some(emp) = &p.employment_date {
        let years_str = compute_years(emp);
        println!("  Employed:    {emp}{years_str}");
    }

    if let Some(email) = &p.email {
        println!("  Email:       {email}");
    }
    if let Some(phone) = &p.phone {
        println!("  Phone:       {phone}");
    }

    // Relationships grouped by type
    if !detail.relationships.is_empty() {
        println!();
        let mut by_type: std::collections::BTreeMap<&str, Vec<String>> =
            std::collections::BTreeMap::new();
        for rel in &detail.relationships {
            let entry = by_type.entry(&rel.rel_type).or_default();
            let mut s = rel.name.clone();
            if let Some(b) = &rel.birthday {
                if let Ok(date) = NaiveDate::parse_from_str(b, "%Y-%m-%d") {
                    s.push_str(&format!(
                        " (birthday: {} {})",
                        date.format("%b"),
                        date.day()
                    ));
                }
            }
            entry.push(s);
        }
        for (rel_type, names) in &by_type {
            let label = capitalize(rel_type);
            println!("  {label}: {}", names.join(", "));
        }
    }

    // Tags
    if !detail.tags.is_empty() {
        println!();
        if cfg.color {
            println!(
                "  Tags: {}",
                detail
                    .tags
                    .iter()
                    .map(|t| t.cyan().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            println!("  Tags: {}", detail.tags.join(", "));
        }
    }

    // Notes
    if !detail.notes.is_empty() {
        println!();
        println!("  Notes:");
        for note in &detail.notes {
            let date = &note.created_at[..10];
            println!("    {date}  {}", note.content);
        }
    }

    // Custom fields
    if !detail.custom_fields.is_empty() {
        println!();
        println!("  Custom Fields:");
        for cf in &detail.custom_fields {
            println!("    {}: {}", cf.key, cf.value);
        }
    }
}

pub fn list_people(people: &[Person], cfg: &DisplayConfig) {
    if people.is_empty() {
        println!("No people found.");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec!["Name", "Company", "Title", "Email"]);

    for p in people {
        table.add_row(vec![
            Cell::new(&p.name),
            Cell::new(p.company.as_deref().unwrap_or("")),
            Cell::new(p.job_title.as_deref().unwrap_or("")),
            Cell::new(p.email.as_deref().unwrap_or("")),
        ]);
    }

    if !cfg.color {
        table.force_no_tty();
    }

    println!("{table}");
}

pub fn show_dates(events: &[DateEvent], range_label: &str, cfg: &DisplayConfig) {
    if events.is_empty() {
        println!("No events for {range_label}.");
        return;
    }

    if cfg.color {
        println!("{}", format!("Upcoming events ({range_label}):").bold());
    } else {
        println!("Upcoming events ({range_label}):");
    }

    let today = Local::now().date_naive();

    for ev in events {
        if let Ok(date) = NaiveDate::parse_from_str(&ev.event_date, "%Y-%m-%d") {
            let this_year_date =
                NaiveDate::from_ymd_opt(today.year(), date.month(), date.day()).unwrap_or(date);
            let days_away = (this_year_date - today).num_days();

            let countdown = if days_away == 0 {
                "today".to_string()
            } else if days_away == 1 {
                "in 1 day".to_string()
            } else if days_away > 0 {
                format!("in {} days", days_away)
            } else {
                format!("{} days ago", -days_away)
            };

            let event_label = format_event_type(&ev.event_type);
            let extra = match ev.event_type.as_str() {
                "birthday" => {
                    let age = today.year() - date.year();
                    format!(" (turns {age})")
                }
                "employment_anniversary" => {
                    let years = today.year() - date.year();
                    format!(" ({years} years)")
                }
                _ => String::new(),
            };

            let date_str = format!("{}", this_year_date.format("%b %-d"));
            let name_display = if ev.event_type.contains("_birthday")
                && ev.event_type != "birthday"
            {
                format!("{} ({})", ev.person_name, ev.subject_name)
            } else {
                ev.person_name.clone()
            };

            println!(
                "  {:<8} {:<20} {:<22} {}{}",
                date_str, name_display, event_label, countdown, extra
            );
        }
    }
}

fn format_event_type(t: &str) -> String {
    match t {
        "birthday" => "Birthday".to_string(),
        "employment_anniversary" => "Employment Anniv.".to_string(),
        other => {
            let s = other.replace('_', " ");
            capitalize(&s)
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

fn compute_age(birthday: &str) -> String {
    if let Ok(date) = NaiveDate::parse_from_str(birthday, "%Y-%m-%d") {
        let today = Local::now().date_naive();
        let mut age = today.year() - date.year();
        if (today.month(), today.day()) < (date.month(), date.day()) {
            age -= 1;
        }
        format!(" (age {age})")
    } else {
        String::new()
    }
}

fn compute_years(start: &str) -> String {
    if let Ok(date) = NaiveDate::parse_from_str(start, "%Y-%m-%d") {
        let today = Local::now().date_naive();
        let mut years = today.year() - date.year();
        if (today.month(), today.day()) < (date.month(), date.day()) {
            years -= 1;
        }
        if years == 1 {
            " (1 year)".to_string()
        } else {
            format!(" ({years} years)")
        }
    } else {
        String::new()
    }
}
