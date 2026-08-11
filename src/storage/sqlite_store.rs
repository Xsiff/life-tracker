use std::collections::BTreeMap;

use anyhow::Context;
use chrono::NaiveDate;
use directories::ProjectDirs;
use rusqlite::{params, Connection};

use crate::domain::{Activity, Category, Day};

use super::{
    sqlite_decode::{decode_activity_row, decode_day_note_row},
    sqlite_schema::INIT_SQL,
};

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open() -> anyhow::Result<Self> {
        let project_dirs = ProjectDirs::from("dev", "xsiff", "life-tracker")
            .context("failed to resolve data dir")?;
        std::fs::create_dir_all(project_dirs.data_dir()).context("failed to create data dir")?;
        let path = project_dirs.data_dir().join("life-tracker.db");
        let conn = Connection::open(path).context("failed to open database")?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    #[cfg(test)]
    pub fn in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory().context("failed to open in-memory database")?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    pub fn load_all(&self) -> anyhow::Result<BTreeMap<NaiveDate, Day>> {
        let mut days = BTreeMap::new();

        self.load_activities(&mut days)?;
        self.load_day_notes(&mut days)?;

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
        self.conn.execute("DELETE FROM day_notes WHERE date = ?1", params![date.to_string()])?;
        Ok(())
    }

    fn init(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(INIT_SQL)?;
        Ok(())
    }

    fn load_activities(&self, days: &mut BTreeMap<NaiveDate, Day>) -> anyhow::Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT date, hour, category, note FROM activities ORDER BY date, hour")?;
        let rows = stmt.query_map([], decode_activity_row)?;

        for row in rows {
            let row = row?;
            if row.activity.is_empty() {
                continue;
            }
            days.entry(row.date)
                .or_insert_with(|| Day::new(row.date))
                .set_hour(row.hour, row.activity);
        }

        Ok(())
    }

    fn load_day_notes(&self, days: &mut BTreeMap<NaiveDate, Day>) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare("SELECT date, note FROM day_notes ORDER BY date")?;
        let rows = stmt.query_map([], decode_day_note_row)?;

        for row in rows {
            let row = row?;
            days.entry(row.date).or_insert_with(|| Day::new(row.date)).set_note(row.note);
        }

        Ok(())
    }
}
