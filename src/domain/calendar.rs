use chrono::NaiveDate;

use super::note::normalize_note;
use super::Activity;

pub const HOURS_PER_DAY: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Day {
    date: NaiveDate,
    hours: [Option<Activity>; HOURS_PER_DAY],
    note: Option<String>,
}

impl Day {
    pub fn new(date: NaiveDate) -> Self {
        Self { date, hours: std::array::from_fn(|_| None), note: None }
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn activity(&self, hour: u8) -> Option<&Activity> {
        self.hours.get(usize::from(hour)).and_then(Option::as_ref)
    }

    pub fn set_hour(&mut self, hour: u8, activity: Activity) -> Option<Activity> {
        self.hours
            .get_mut(usize::from(hour))
            .map(|slot| slot.replace(activity))
            .unwrap_or_else(|| panic!("hour out of range: {hour}"))
    }

    pub fn clear_hour(&mut self, hour: u8) -> Option<Activity> {
        self.hours
            .get_mut(usize::from(hour))
            .map(Option::take)
            .unwrap_or_else(|| panic!("hour out of range: {hour}"))
    }

    pub fn set_note(&mut self, note: impl Into<String>) {
        self.note = normalize_note(note.into());
    }

    pub fn clear_note(&mut self) {
        self.note = None;
    }

    pub fn is_empty(&self) -> bool {
        self.note.is_none() && self.hours.iter().all(Option::is_none)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::Day;
    use crate::domain::{Activity, Category};

    #[test]
    fn empty_day_detects_empty_after_clearing() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 2).unwrap();
        let mut day = Day::new(date);
        day.set_note("note");
        day.set_hour(8, Activity::new(Category::Work));
        day.clear_note();
        day.clear_hour(8);

        assert!(day.is_empty());
    }
}
