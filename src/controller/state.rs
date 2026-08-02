use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::domain::{Activity, Category, Day};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Calendar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub date: NaiveDate,
    pub hour: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteTarget {
    Day { date: NaiveDate },
    Hour { date: NaiveDate, hour: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    CategoryPicker {
        date: NaiveDate,
        hour: u8,
        selected: Category,
    },
    NoteEditor {
        target: NoteTarget,
        draft: String,
        cursor: usize,
    },
}

#[derive(Debug, Clone)]
pub struct State {
    pub view: ViewMode,
    pub cursor: Cursor,
    pub overlay: Option<Overlay>,
    pub days: BTreeMap<NaiveDate, Day>,
    pub last_error: Option<String>,
    pub quit: bool,
}

impl State {
    pub fn day(&self, date: NaiveDate) -> Option<&Day> {
        self.days.get(&date)
    }

    pub fn activity(&self, date: NaiveDate, hour: u8) -> Option<&Activity> {
        self.day(date).and_then(|day| day.activity(hour))
    }
}
