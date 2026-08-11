use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::domain::{Activity, Day};

use super::{Controller, NoteTarget, State};

impl Controller {
    pub(in crate::controller) fn save_note(
        &mut self,
        target: NoteTarget,
        draft: String,
    ) -> anyhow::Result<()> {
        match target {
            NoteTarget::Day { date } => {
                if draft.trim().is_empty() {
                    self.store.clear_day_note(date)?;
                    if let Some(day) = self.state.days.get_mut(&date) {
                        day.clear_note();
                    }
                    cleanup_day(&mut self.state.days, date);
                } else {
                    self.store.set_day_note(date, &draft)?;
                    ensure_day(&mut self.state.days, date).set_note(draft);
                }
            }
            NoteTarget::Hour { date, hour } => {
                if draft.trim().is_empty() {
                    self.clear_hour_note(date, hour)?;
                } else {
                    let activity = match self.state.activity(date, hour) {
                        Some(existing) => {
                            let mut activity = existing.clone();
                            activity.set_note(draft);
                            activity
                        }
                        None => Activity::note_only(draft),
                    };
                    self.store.set_hour(date, hour, &activity)?;
                    self.write_hour_state(date, hour, activity);
                }
            }
        }

        self.state.overlay = None;
        self.restore_focus(target);
        Ok(())
    }

    pub(in crate::controller) fn write_hour_state(
        &mut self,
        date: NaiveDate,
        hour: u8,
        activity: Activity,
    ) {
        ensure_day(&mut self.state.days, date).set_hour(hour, activity);
    }

    fn clear_hour(&mut self, date: NaiveDate, hour: u8) -> anyhow::Result<()> {
        self.store.clear_hour(date, hour)?;
        if let Some(day) = self.state.days.get_mut(&date) {
            day.clear_hour(hour);
        }
        cleanup_day(&mut self.state.days, date);
        Ok(())
    }

    pub(in crate::controller) fn clear_hour_if_present(
        &mut self,
        date: NaiveDate,
        hour: u8,
    ) -> anyhow::Result<()> {
        let Some(existing) = self.state.activity(date, hour).cloned() else {
            return Ok(());
        };
        if !existing.has_category() {
            return Ok(());
        }

        let mut activity = existing;
        activity.clear_category();
        if activity.is_empty() {
            self.clear_hour(date, hour)
        } else {
            self.store.set_hour(date, hour, &activity)?;
            self.write_hour_state(date, hour, activity);
            Ok(())
        }
    }

    fn clear_hour_note(&mut self, date: NaiveDate, hour: u8) -> anyhow::Result<()> {
        let Some(existing) = self.state.activity(date, hour).cloned() else {
            return Ok(());
        };
        if !existing.has_note() {
            return Ok(());
        }

        let mut activity = existing;
        activity.clear_note();
        if activity.is_empty() {
            self.clear_hour(date, hour)?;
        } else {
            self.store.set_hour(date, hour, &activity)?;
            self.write_hour_state(date, hour, activity);
        }
        Ok(())
    }

    fn clear_day_note(&mut self, date: NaiveDate) -> anyhow::Result<()> {
        self.store.clear_day_note(date)?;
        if let Some(day) = self.state.days.get_mut(&date) {
            day.clear_note();
        }
        cleanup_day(&mut self.state.days, date);
        Ok(())
    }

    fn clear_day_note_if_present(&mut self, date: NaiveDate) -> anyhow::Result<()> {
        let has_note = self.state.day(date).and_then(Day::note).is_some();
        if !has_note {
            return Ok(());
        }
        self.clear_day_note(date)
    }

    pub(in crate::controller) fn delete_note(&mut self, target: NoteTarget) -> anyhow::Result<()> {
        match target {
            NoteTarget::Day { date } => self.clear_day_note_if_present(date)?,
            NoteTarget::Hour { date, hour } => self.clear_hour_note(date, hour)?,
        }
        self.state.overlay = None;
        self.restore_focus(target);
        Ok(())
    }
}

pub(in crate::controller) fn note_draft(state: &State, target: &NoteTarget) -> String {
    match *target {
        NoteTarget::Day { date } => state.day(date).and_then(Day::note).unwrap_or("").to_string(),
        NoteTarget::Hour { date, hour } => state
            .activity(date, hour)
            .and_then(|activity| activity.note())
            .unwrap_or("")
            .to_string(),
    }
}

fn ensure_day(days: &mut BTreeMap<NaiveDate, Day>, date: NaiveDate) -> &mut Day {
    days.entry(date).or_insert_with(|| Day::new(date))
}

fn cleanup_day(days: &mut BTreeMap<NaiveDate, Day>, date: NaiveDate) {
    let should_remove = days.get(&date).map(Day::is_empty).unwrap_or(false);
    if should_remove {
        days.remove(&date);
    }
}
