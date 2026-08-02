use std::{collections::BTreeMap, str::FromStr};

use anyhow::Context;
use chrono::NaiveDate;
use directories::ProjectDirs;
use rusqlite::{params, Connection};

use crate::domain::{Activity, Category, Day};

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open() -> anyhow::Result<Self> {
        let project_dirs = ProjectDirs::from("dev", "openai", "life-tracker")
            .context("failed to resolve data dir")?;
        std::fs::create_dir_all(project_dirs.data_dir())
            .context("failed to create data dir")?;
        let path = project_dirs.data_dir().join("life-tracker.db");
        let conn = Connection::open(path).context("failed to open database")?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    pub fn load_all(&self) -> anyhow::Result<BTreeMap<NaiveDate, Day>> {
        let mut days = BTreeMap::new();

        let mut stmt = self
            .conn
            .prepare("SELECT date, hour, category, note FROM activities ORDER BY date, hour")?;
        let rows = stmt.query_map([], |row| {
            let date: String = row.get(0)?;
            let hour: i64 = row.get(1)?;
            let category: String = row.get(2)?;
            let note: Option<String> = row.get(3)?;
            Ok((date, hour, category, note))
        })?;
        for row in rows {
            let (date, hour, category, note) = row?;
            let date = parse_date(&date)?;
            let hour = u8::try_from(hour).context("invalid hour")?;
            let activity = if category.is_empty() {
                Activity::note_only(note.unwrap_or_default())
            } else {
                let category = category.parse::<Category>().map_err(|err| anyhow::anyhow!(err))?;
                match note {
                    Some(note) => Activity::with_note(category, note),
                    None => Activity::new(category),
                }
            };
            if activity.is_empty() {
                continue;
            }
            days.entry(date).or_insert_with(|| Day::new(date)).set_hour(hour, activity);
        }

        let mut stmt = self
            .conn
            .prepare("SELECT date, note FROM day_notes ORDER BY date")?;
        let rows = stmt.query_map([], |row| {
            let date: String = row.get(0)?;
            let note: String = row.get(1)?;
            Ok((date, note))
        })?;
        for row in rows {
            let (date, note) = row?;
            let date = parse_date(&date)?;
            days.entry(date).or_insert_with(|| Day::new(date)).set_note(note);
        }

        days.retain(|_, day| !day.is_empty());
        Ok(days)
    }

    pub fn set_hour(&self, date: NaiveDate, hour: u8, act: &Activity) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO activities(date, hour, category, note) VALUES (?1, ?2, ?3, ?4)",
            params![
                date.to_string(),
                i64::from(hour),
                act.category().map(Category::label).unwrap_or(""),
                act.note()
            ],
        )?;
        Ok(())
    }

    pub fn clear_hour(&self, date: NaiveDate, hour: u8) -> anyhow::Result<()> {
        self.conn.execute(
            "DELETE FROM activities WHERE date = ?1 AND hour = ?2",
            params![date.to_string(), i64::from(hour)],
        )?;
        Ok(())
    }

    pub fn set_day_note(&self, date: NaiveDate, note: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO day_notes(date, note) VALUES (?1, ?2)",
            params![date.to_string(), note],
        )?;
        Ok(())
    }

    pub fn clear_day_note(&self, date: NaiveDate) -> anyhow::Result<()> {
        self.conn.execute(
            "DELETE FROM day_notes WHERE date = ?1",
            params![date.to_string()],
        )?;
        Ok(())
    }

    fn init(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS activities (
                date     TEXT    NOT NULL,
                hour     INTEGER NOT NULL,
                category TEXT    NOT NULL,
                note     TEXT,
                PRIMARY KEY (date, hour)
            );

            CREATE TABLE IF NOT EXISTS day_notes (
                date     TEXT    NOT NULL,
                note     TEXT    NOT NULL,
                PRIMARY KEY (date)
            );
            "#,
        )?;
        Ok(())
    }
}

fn parse_date(value: &str) -> anyhow::Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(value, "%Y-%m-%d")?)
}
