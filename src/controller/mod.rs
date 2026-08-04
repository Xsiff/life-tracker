mod state;

use std::collections::BTreeMap;

use chrono::{Local, NaiveDate};

use crate::{
    domain::{Action, Activity, Category, Day},
    storage::Store,
};

pub use state::{CategoryPickerSelection, NoteTarget, Overlay, State};
#[cfg(test)]
pub use state::Cursor;

pub struct Controller {
    state: State,
    store: Store,
}

impl Controller {
    pub fn new(store: Store) -> anyhow::Result<Self> {
        let days = store.load_all()?;
        Ok(Self {
            state: State::new(today(), days),
            store,
        })
    }

    pub fn update(&mut self, action: Action) -> anyhow::Result<()> {
        self.state.last_error = None;

        if self.state.overlay.is_some() {
            self.update_overlay(action)?;
        } else {
            self.update_base(action)?;
        }

        Ok(())
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn should_quit(&self) -> bool {
        self.state.quit
    }

    fn update_base(&mut self, action: Action) -> anyhow::Result<()> {
        self.update_calendar(action)
    }

    fn update_overlay(&mut self, action: Action) -> anyhow::Result<()> {
        let Some(overlay) = self.state.overlay.take() else {
            return Ok(());
        };

        match (overlay, action) {
            (Overlay::CategoryPicker { target, selected }, Action::MoveUp) => {
                let selected = move_picker_up(&target, selected);
                self.state.overlay = Some(Overlay::CategoryPicker { target, selected });
            }
            (Overlay::CategoryPicker { target, selected }, Action::MoveDown) => {
                let selected = move_picker_down(&target, selected);
                self.state.overlay = Some(Overlay::CategoryPicker { target, selected });
            }
            (Overlay::CategoryPicker { target, .. }, Action::Digit(n)) => {
                if matches!(target, NoteTarget::Hour { .. }) {
                    if let Some(category) = Category::from_digit(n) {
                        self.state.overlay = Some(Overlay::CategoryPicker {
                            target,
                            selected: CategoryPickerSelection::Category(category),
                        });
                    }
                } else {
                    self.state.overlay = Some(Overlay::CategoryPicker {
                        target,
                        selected: CategoryPickerSelection::AddNote,
                    });
                }
            }
            (Overlay::CategoryPicker { target, selected }, Action::Confirm) => match selected {
                CategoryPickerSelection::Category(category) => {
                    if let NoteTarget::Hour { date, hour } = target {
                        let activity = match self.state.activity(date, hour) {
                            Some(existing) => {
                                let mut activity = existing.clone();
                                activity.set_category(category);
                                activity
                            }
                            None => Activity::new(category),
                        };
                        self.store.set_hour(date, hour, &activity)?;
                        ensure_day(&mut self.state.days, date).set_hour(hour, activity);
                        self.state.overlay = None;
                        self.restore_focus(target);
                    } else {
                        self.state.overlay = Some(Overlay::CategoryPicker {
                            target,
                            selected: CategoryPickerSelection::AddNote,
                        });
                    }
                }
                CategoryPickerSelection::AddNote => {
                    self.open_note_editor(target);
                }
                CategoryPickerSelection::DeleteNote => {
                    self.delete_note(target)?;
                }
                CategoryPickerSelection::DeleteActivity => {
                    if let NoteTarget::Hour { date, hour } = target {
                        self.clear_hour_if_present(date, hour)?;
                        self.state.overlay = None;
                        self.restore_focus(target);
                    } else {
                        self.state.overlay = Some(Overlay::CategoryPicker {
                            target,
                            selected: CategoryPickerSelection::DeleteNote,
                        });
                    }
                }
            },
            (Overlay::CategoryPicker { target, .. }, Action::Cancel) => {
                self.state.overlay = None;
                self.restore_focus(target);
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::Char(c)) => {
                let mut draft = draft;
                let mut cursor = cursor;
                insert_char(&mut draft, &mut cursor, c);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::InsertNewline) => {
                let mut draft = draft;
                let mut cursor = cursor;
                insert_char(&mut draft, &mut cursor, '\n');
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::Erase) => {
                let mut draft = draft;
                let mut cursor = cursor;
                erase_char(&mut draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, .. }, Action::Confirm) => {
                self.save_note(target, draft)?;
            }
            (Overlay::NoteEditor { target, .. }, Action::Cancel) => {
                self.state.overlay = None;
                self.restore_focus(target);
            }
            (overlay, _) => {
                self.state.overlay = Some(overlay);
            }
        }

        Ok(())
    }

    fn update_calendar(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::MoveLeft => {
                self.move_cursor_hour(-1);
            }
            Action::MoveRight => {
                self.move_cursor_hour(1);
            }
            Action::MoveUp => {
                self.state.cursor.date -= chrono::Duration::days(1);
            }
            Action::MoveDown => {
                self.state.cursor.date += chrono::Duration::days(1);
            }
            Action::Confirm => {
                let date = self.state.cursor.date;
                let target = match self.state.cursor.hour {
                    Some(hour) => NoteTarget::Hour { date, hour },
                    None => NoteTarget::Day { date },
                };
                self.state.overlay = Some(Overlay::CategoryPicker {
                    selected: picker_default_selection(&self.state, &target),
                    target,
                });
            }
            Action::CycleView => {}
            Action::Char('q') | Action::Char('Q') => self.state.quit = true,
            Action::Tick
            | Action::Cancel
            | Action::InsertNewline
            | Action::Erase
            | Action::Digit(_)
            | Action::Char(_) => {}
        }
        Ok(())
    }

    fn move_cursor_hour(&mut self, delta: i8) {
        let (date, hour) =
            move_cursor_hour(self.state.cursor.date, self.state.cursor.hour, delta);
        self.state.cursor.date = date;
        self.state.cursor.hour = hour;
    }

    fn open_note_editor(&mut self, target: NoteTarget) {
        let draft = note_draft(&self.state, &target);
        let cursor = draft.len();
        self.state.overlay = Some(Overlay::NoteEditor {
            target,
            draft,
            cursor,
        });
    }

    fn restore_focus(&mut self, target: NoteTarget) {
        match target {
            NoteTarget::Day { date } => {
                self.state.cursor.date = date;
                self.state.cursor.hour = None;
            }
            NoteTarget::Hour { date, hour } => {
                self.state.cursor.date = date;
                self.state.cursor.hour = Some(hour);
            }
        }
    }

    fn save_note(&mut self, target: NoteTarget, draft: String) -> anyhow::Result<()> {
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
                    ensure_day(&mut self.state.days, date).set_hour(hour, activity);
                }
            }
        }

        self.state.overlay = None;
        self.restore_focus(target);
        Ok(())
    }

    fn clear_hour(&mut self, date: NaiveDate, hour: u8) -> anyhow::Result<()> {
        self.store.clear_hour(date, hour)?;
        if let Some(day) = self.state.days.get_mut(&date) {
            day.clear_hour(hour);
        }
        cleanup_day(&mut self.state.days, date);
        Ok(())
    }

    fn clear_hour_if_present(&mut self, date: NaiveDate, hour: u8) -> anyhow::Result<()> {
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
            ensure_day(&mut self.state.days, date).set_hour(hour, activity);
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
            ensure_day(&mut self.state.days, date).set_hour(hour, activity);
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

    fn delete_note(&mut self, target: NoteTarget) -> anyhow::Result<()> {
        match target {
            NoteTarget::Day { date } => self.clear_day_note_if_present(date)?,
            NoteTarget::Hour { date, hour } => self.clear_hour_note(date, hour)?,
        }
        self.state.overlay = None;
        self.restore_focus(target);
        Ok(())
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

fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn move_cursor_hour(date: NaiveDate, hour: Option<u8>, delta: i8) -> (NaiveDate, Option<u8>) {
    if delta == 0 {
        return (date, hour);
    }

    match (hour, delta.is_negative()) {
        (None, true) => (date - chrono::Duration::days(1), Some(23)),
        (None, false) => (date, Some(0)),
        (Some(0), true) => (date, None),
        (Some(23), false) => (date + chrono::Duration::days(1), None),
        (Some(hour), true) => (date, Some(hour - 1)),
        (Some(hour), false) => (date, Some(hour + 1)),
    }
}

fn picker_default_selection(state: &State, target: &NoteTarget) -> CategoryPickerSelection {
    match *target {
        NoteTarget::Day { .. } => CategoryPickerSelection::AddNote,
        NoteTarget::Hour { date, hour } => match state.activity(date, hour).and_then(Activity::category)
        {
            Some(category) => CategoryPickerSelection::Category(category),
            None => CategoryPickerSelection::AddNote,
        },
    }
}

fn note_draft(state: &State, target: &NoteTarget) -> String {
    match *target {
        NoteTarget::Day { date } => state
            .day(date)
            .and_then(Day::note)
            .unwrap_or("")
            .to_string(),
        NoteTarget::Hour { date, hour } => state
            .activity(date, hour)
            .and_then(|activity| activity.note())
            .unwrap_or("")
            .to_string(),
    }
}

fn move_picker_up(target: &NoteTarget, selected: CategoryPickerSelection) -> CategoryPickerSelection {
    move_picker_by(target, selected, -1)
}

fn move_picker_down(
    target: &NoteTarget,
    selected: CategoryPickerSelection,
) -> CategoryPickerSelection {
    move_picker_by(target, selected, 1)
}

fn move_picker_by(
    target: &NoteTarget,
    selected: CategoryPickerSelection,
    delta: isize,
) -> CategoryPickerSelection {
    let current = picker_selection_index(target, selected);
    let max = picker_selection_count(target).saturating_sub(1) as isize;
    let next = (current as isize + delta).clamp(0, max) as usize;
    picker_selection_at(target, next)
}

fn picker_selection_count(target: &NoteTarget) -> usize {
    match target {
        NoteTarget::Day { .. } => 2,
        NoteTarget::Hour { .. } => Category::ALL.len() + 3,
    }
}

fn picker_selection_index(target: &NoteTarget, selected: CategoryPickerSelection) -> usize {
    match (target, selected) {
        (NoteTarget::Day { .. }, CategoryPickerSelection::AddNote) => 0,
        (NoteTarget::Day { .. }, CategoryPickerSelection::DeleteNote) => 1,
        (NoteTarget::Day { .. }, _) => 0,
        (NoteTarget::Hour { .. }, CategoryPickerSelection::Category(category)) => {
            usize::from(category.as_u8())
        }
        (NoteTarget::Hour { .. }, CategoryPickerSelection::AddNote) => Category::ALL.len(),
        (NoteTarget::Hour { .. }, CategoryPickerSelection::DeleteNote) => Category::ALL.len() + 1,
        (NoteTarget::Hour { .. }, CategoryPickerSelection::DeleteActivity) => {
            Category::ALL.len() + 2
        }
    }
}

fn picker_selection_at(target: &NoteTarget, index: usize) -> CategoryPickerSelection {
    match target {
        NoteTarget::Day { .. } => match index {
            0 => CategoryPickerSelection::AddNote,
            _ => CategoryPickerSelection::DeleteNote,
        },
        NoteTarget::Hour { .. } => {
            if index < Category::ALL.len() {
                CategoryPickerSelection::Category(Category::ALL[index])
            } else {
                match index - Category::ALL.len() {
                    0 => CategoryPickerSelection::AddNote,
                    1 => CategoryPickerSelection::DeleteNote,
                    _ => CategoryPickerSelection::DeleteActivity,
                }
            }
        }
    }
}

fn insert_char(draft: &mut String, cursor: &mut usize, c: char) {
    draft.insert(*cursor, c);
    *cursor += c.len_utf8();
}

fn erase_char(draft: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let new_cursor = draft[..*cursor]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    draft.drain(new_cursor..*cursor);
    *cursor = new_cursor;
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{insert_char, move_cursor_hour};

    #[test]
    fn insert_char_supports_newlines() {
        let mut draft = String::from("abc");
        let mut cursor = 1usize;

        insert_char(&mut draft, &mut cursor, '\n');

        assert_eq!(draft, "a\nbc");
        assert_eq!(cursor, 2);
    }

    #[test]
    fn move_cursor_hour_wraps_left_from_date_column_to_previous_day() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();

        assert_eq!(
            move_cursor_hour(date, None, -1),
            (NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(), Some(23))
        );
    }

    #[test]
    fn move_cursor_hour_enters_first_hour_from_date_column_when_moving_right() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();

        assert_eq!(move_cursor_hour(date, None, 1), (date, Some(0)));
    }

    #[test]
    fn move_cursor_hour_wraps_right_from_last_hour_to_next_day_column() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();

        assert_eq!(
            move_cursor_hour(date, Some(23), 1),
            (NaiveDate::from_ymd_opt(2026, 8, 5).unwrap(), None)
        );
    }
}
