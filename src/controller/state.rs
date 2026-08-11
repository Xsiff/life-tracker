use std::collections::BTreeMap;

use chrono::{Local, NaiveDate, Timelike};

use crate::domain::{Activity, Category, Day};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryPickerSelection {
    Category(Category),
    AddNote,
    DeleteNote,
    DeleteActivity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    CategoryPicker { target: NoteTarget, selected: CategoryPickerSelection },
    Help,
    NoteEditor { target: NoteTarget, draft: String, cursor: usize },
}

#[derive(Debug, Clone)]
pub struct State {
    pub cursor: Cursor,
    pub overlay: Option<Overlay>,
    pub days: BTreeMap<NaiveDate, Day>,
    pub last_error: Option<String>,
    pub quit: bool,
}

impl State {
    pub fn new(today: NaiveDate, days: BTreeMap<NaiveDate, Day>) -> Self {
        Self {
            cursor: Cursor { date: today, hour: Some(Local::now().hour() as u8) },
            overlay: None,
            days,
            last_error: None,
            quit: false,
        }
    }

    pub fn day(&self, date: NaiveDate) -> Option<&Day> {
        self.days.get(&date)
    }

    pub fn activity(&self, date: NaiveDate, hour: u8) -> Option<&Activity> {
        self.day(date).and_then(|day| day.activity(hour))
    }
}
