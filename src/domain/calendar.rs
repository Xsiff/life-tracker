use chrono::{Datelike, Duration, NaiveDate, Weekday};

use super::{Activity, Category};

pub const HOURS_PER_DAY: usize = 24;
#[allow(dead_code)]
pub const WINDOW_WEEKS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Day {
    date: NaiveDate,
    hours: [Option<Activity>; HOURS_PER_DAY],
    note: Option<String>,
}

#[allow(dead_code)]
impl Day {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            hours: std::array::from_fn(|_| None),
            note: None,
        }
    }

    pub const fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn hours(&self) -> &[Option<Activity>; HOURS_PER_DAY] {
        &self.hours
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn activity(&self, hour: u8) -> Option<&Activity> {
        self.hours.get(usize::from(hour)).and_then(Option::as_ref)
    }

    pub fn get_hour(&self, hour: u8) -> Option<&Activity> {
        self.activity(hour)
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

    pub fn has_note(&self) -> bool {
        self.note.is_some()
    }

    pub fn filled_hours(&self) -> usize {
        self.hours
            .iter()
            .flatten()
            .filter(|activity| activity.has_category())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.note.is_none() && self.hours.iter().all(Option::is_none)
    }

    pub fn dominant_category(&self) -> Option<Category> {
        let mut counts = [0usize; Category::ALL.len()];
        for activity in self.hours.iter().flatten() {
            if let Some(category) = activity.category() {
                counts[usize::from(category.as_u8())] += 1;
            }
        }

        let mut best = None;
        for category in Category::ALL {
            let count = counts[usize::from(category.as_u8())];
            if count == 0 {
                continue;
            }

            match best {
                Some((_, best_count)) if best_count >= count => {}
                _ => best = Some((category, count)),
            }
        }

        best.map(|(category, _)| category)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Week {
    iso_year: i32,
    iso_week: u32,
}

#[allow(dead_code)]
impl Week {
    pub fn from_date(date: NaiveDate) -> Self {
        let iso = date.iso_week();
        Self {
            iso_year: iso.year(),
            iso_week: iso.week(),
        }
    }

    pub const fn iso_year(self) -> i32 {
        self.iso_year
    }

    pub const fn iso_week(self) -> u32 {
        self.iso_week
    }

    pub fn start_date(self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.iso_year, self.iso_week, Weekday::Mon)
            .expect("valid ISO week")
    }

    pub fn contains(self, date: NaiveDate) -> bool {
        Self::from_date(date) == self
    }

    pub fn offset(self, weeks: i64) -> Self {
        Self::from_date(self.start_date() + Duration::weeks(weeks))
    }

    pub fn centered_window(self) -> [Self; WINDOW_WEEKS] {
        let radius = (WINDOW_WEEKS / 2) as i64;
        std::array::from_fn(|idx| self.offset(idx as i64 - radius))
    }
}

#[allow(dead_code)]
pub fn start_of_week(date: NaiveDate) -> NaiveDate {
    let days_from_monday = i64::from(date.weekday().num_days_from_monday());
    date - Duration::days(days_from_monday)
}

#[allow(dead_code)]
pub fn week_window_centered(date: NaiveDate) -> [NaiveDate; WINDOW_WEEKS] {
    let start = start_of_week(date);
    std::array::from_fn(|idx| start + Duration::weeks(idx as i64))
}

fn normalize_note(note: String) -> Option<String> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.len() == note.len() {
        Some(note)
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{Day, Week, WINDOW_WEEKS};
    use crate::domain::{Activity, Category};

    #[test]
    fn dominant_category_prefers_most_frequent() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 2).unwrap();
        let mut day = Day::new(date);
        day.set_hour(8, Activity::new(Category::Work));
        day.set_hour(9, Activity::new(Category::Work));
        day.set_hour(10, Activity::new(Category::Health));

        assert_eq!(day.dominant_category(), Some(Category::Work));
    }

    #[test]
    fn dominant_category_breaks_ties_by_lower_discriminant() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 2).unwrap();
        let mut day = Day::new(date);
        day.set_hour(8, Activity::new(Category::Health));
        day.set_hour(9, Activity::new(Category::Sleep));

        assert_eq!(day.dominant_category(), Some(Category::Sleep));
    }

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

    #[test]
    fn centered_week_window_keeps_selected_week_in_middle() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 2).unwrap();
        let week = Week::from_date(date);
        let window = week.centered_window();

        assert_eq!(window.len(), WINDOW_WEEKS);
        assert_eq!(window[WINDOW_WEEKS / 2], week);
    }
}
