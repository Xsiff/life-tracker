use chrono::{Datelike, Duration, NaiveDate};

use super::{Activity, Category};

pub const HOURS_PER_DAY: usize = 24;
pub const WINDOW_WEEKS: usize = 5;

#[derive(Debug, Clone)]
pub struct Day {
    pub date: NaiveDate,
    pub hours: [Option<Activity>; HOURS_PER_DAY],
    pub note: Option<String>,
}

impl Day {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            hours: std::array::from_fn(|_| None),
            note: None,
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn set_hour(&mut self, hour: u8, activity: Activity) {
        self.hours[hour as usize] = Some(activity);
    }

    pub fn activity(&self, hour: u8) -> Option<&Activity> {
        self.hours.get(hour as usize).and_then(Option::as_ref)
    }

    pub fn filled_hours(&self) -> usize {
        self.hours.iter().filter(|slot| slot.is_some()).count()
    }

    pub fn has_note(&self) -> bool {
        self.note.as_deref().is_some_and(|note| !note.is_empty())
    }

    pub fn is_empty(&self) -> bool {
        self.filled_hours() == 0 && !self.has_note()
    }

    pub fn dominant_category(&self) -> Option<Category> {
        let mut counts = [0usize; Category::ALL.len()];
        for activity in self.hours.iter().flatten() {
            counts[activity.category as usize] += 1;
        }

        counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .max_by(|(left_idx, left_count), (right_idx, right_count)| {
                left_count
                    .cmp(right_count)
                    .then_with(|| right_idx.cmp(left_idx))
            })
            .and_then(|(idx, _)| Category::from_digit(idx as u8))
    }
}

pub fn start_of_week(date: NaiveDate) -> NaiveDate {
    let days_from_monday = i64::from(date.weekday().num_days_from_monday());
    date - Duration::days(days_from_monday)
}

pub fn week_window_centered(date: NaiveDate) -> [NaiveDate; WINDOW_WEEKS] {
    let start = start_of_week(date);
    std::array::from_fn(|idx| start + Duration::weeks(idx as i64))
}
