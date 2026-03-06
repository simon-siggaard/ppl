use anyhow::{bail, Result};
use chrono::{Datelike, Local, NaiveDate};
use rusqlite::Connection;

use crate::models::DateEvent;

pub enum DateRange {
    Today,
    Tomorrow,
    ThisWeek,
    ThisMonth,
    ThisYear,
    Next(i64),  // next N days
    Last(i64),  // last N days
}

impl DateRange {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "today" => Ok(Self::Today),
            "tomorrow" => Ok(Self::Tomorrow),
            "this-week" => Ok(Self::ThisWeek),
            "this-month" => Ok(Self::ThisMonth),
            "this-year" => Ok(Self::ThisYear),
            "next-7d" => Ok(Self::Next(7)),
            "next-30d" => Ok(Self::Next(30)),
            "next-90d" => Ok(Self::Next(90)),
            "last-7d" => Ok(Self::Last(7)),
            "last-30d" => Ok(Self::Last(30)),
            _ => bail!("Unknown date range: {s}. Use: today, tomorrow, this-week, this-month, this-year, next-7d, next-30d, next-90d, last-7d, last-30d"),
        }
    }

    pub fn to_date_bounds(&self) -> (NaiveDate, NaiveDate) {
        let today = Local::now().date_naive();
        match self {
            Self::Today => (today, today),
            Self::Tomorrow => {
                let t = today + chrono::Duration::days(1);
                (t, t)
            }
            Self::ThisWeek => {
                let weekday = today.weekday().num_days_from_monday();
                let start = today - chrono::Duration::days(weekday as i64);
                let end = start + chrono::Duration::days(6);
                (start, end)
            }
            Self::ThisMonth => {
                let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
                let end = if today.month() == 12 {
                    NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1).unwrap()
                } - chrono::Duration::days(1);
                (start, end)
            }
            Self::ThisYear => {
                let start = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
                let end = NaiveDate::from_ymd_opt(today.year(), 12, 31).unwrap();
                (start, end)
            }
            Self::Next(days) => (today, today + chrono::Duration::days(*days)),
            Self::Last(days) => (today - chrono::Duration::days(*days), today),
        }
    }
}

pub fn query_events(conn: &Connection, range: &DateRange) -> Result<Vec<DateEvent>> {
    let (start, end) = range.to_date_bounds();
    let start_md = format!("{:02}-{:02}", start.month(), start.day());
    let end_md = format!("{:02}-{:02}", end.month(), end.day());

    let wraps_year = start_md > end_md;

    let mut stmt = conn.prepare(
        "SELECT person_id, person_name, event_type, event_date, subject_name FROM date_events",
    )?;

    let all_events: Vec<DateEvent> = stmt
        .query_map([], |row| {
            Ok(DateEvent {
                person_id: row.get(0)?,
                person_name: row.get(1)?,
                event_type: row.get(2)?,
                event_date: row.get(3)?,
                subject_name: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let filtered: Vec<DateEvent> = all_events
        .into_iter()
        .filter(|ev| {
            if let Some(date) = NaiveDate::parse_from_str(&ev.event_date, "%Y-%m-%d").ok() {
                let md = format!("{:02}-{:02}", date.month(), date.day());
                if wraps_year {
                    md >= start_md || md <= end_md
                } else {
                    md >= start_md && md <= end_md
                }
            } else {
                false
            }
        })
        .collect();

    // Sort by month-day distance from today
    let today = Local::now().date_naive();
    let mut sorted = filtered;
    sorted.sort_by_key(|ev| {
        let date = NaiveDate::parse_from_str(&ev.event_date, "%Y-%m-%d").unwrap();
        let md_today = today.ordinal();
        let md_event = NaiveDate::from_ymd_opt(today.year(), date.month(), date.day())
            .unwrap_or(date)
            .ordinal();
        if md_event >= md_today {
            md_event - md_today
        } else {
            365 + md_event - md_today
        }
    });

    Ok(sorted)
}
