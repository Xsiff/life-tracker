use anyhow::Context;
use chrono::NaiveDate;
use rusqlite::Row;

use crate::domain::{Activity, Category};

pub(super) struct ActivityRow {
    pub(super) date: NaiveDate,
    pub(super) hour: u8,
    pub(super) activity: Activity,
}

pub(super) struct DayNoteRow {
    pub(super) date: NaiveDate,
    pub(super) note: String,
}

pub(super) fn decode_activity_row(row: &Row<'_>) -> rusqlite::Result<ActivityRow> {
    let date: String = row.get(0)?;
    let hour: i64 = row.get(1)?;
    let category: String = row.get(2)?;
    let note: Option<String> = row.get(3)?;

    Ok(ActivityRow {
        date: parse_date(&date).map_err(into_sql_err)?,
        hour: u8::try_from(hour).context("invalid hour").map_err(into_sql_err)?,
        activity: decode_activity(&category, note).map_err(into_sql_err)?,
    })
}

pub(super) fn decode_day_note_row(row: &Row<'_>) -> rusqlite::Result<DayNoteRow> {
    let date: String = row.get(0)?;
    let note: String = row.get(1)?;

    Ok(DayNoteRow { date: parse_date(&date).map_err(into_sql_err)?, note })
}

fn decode_activity(category: &str, note: Option<String>) -> anyhow::Result<Activity> {
    if category.is_empty() {
        Ok(Activity::note_only(note.unwrap_or_default()))
    } else {
        let category = category.parse::<Category>().map_err(|err| anyhow::anyhow!(err))?;
        Ok(match note {
            Some(note) => Activity::with_note(category, note),
            None => Activity::new(category),
        })
    }
}

fn parse_date(value: &str) -> anyhow::Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(value, "%Y-%m-%d")?)
}

fn into_sql_err(error: anyhow::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())),
    )
}
