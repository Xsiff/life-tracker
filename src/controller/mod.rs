mod state;

use std::collections::BTreeMap;

use chrono::{Local, NaiveDate};

use crate::{
    domain::{Action, Activity, Category, Day},
    storage::Store,
};

pub use state::{CategoryPickerSelection, Cursor, NoteTarget, Overlay, State, ViewMode};

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
                        let activity = Activity::new(category);
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
            Action::Char('n') | Action::Char('N') => {
                self.open_note_editor_for_focus();
            }
            Action::Char('x') | Action::Char('X') => {
                self.clear_focused_value()?;
            }
            Action::Char('q') | Action::Char('Q') => self.state.quit = true,
            Action::Tick
            | Action::Cancel
            | Action::Erase
            | Action::Digit(_)
            | Action::Char(_) => {}
        }
        Ok(())
    }

    fn move_cursor_hour(&mut self, delta: i8) {
        if self.state.cursor.hour.is_none() {
            if delta > 0 {
                self.state.cursor.hour = Some(0);
            }
            return;
        }

        let hour = self.state.cursor.hour.unwrap_or(0) as i16 + i16::from(delta);
        if hour < 0 {
            self.state.cursor.hour = None;
        } else if hour > 23 {
            self.state.cursor.hour = Some(0);
            self.state.cursor.date += chrono::Duration::days(1);
        } else {
            self.state.cursor.hour = Some(hour as u8);
        }
    }

    fn open_note_editor_for_focus(&mut self) {
        let target = match self.state.cursor.hour {
            Some(hour) => NoteTarget::Hour {
                date: self.state.cursor.date,
                hour,
            },
            None => NoteTarget::Day {
                date: self.state.cursor.date,
            },
        };
        self.open_note_editor(target);
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
                    self.store.clear_hour(date, hour)?;
                    if let Some(day) = self.state.days.get_mut(&date) {
                        day.clear_hour(hour);
                    }
                    cleanup_day(&mut self.state.days, date);
                } else {
                    let category = self
                        .state
                        .activity(date, hour)
                        .map(|activity| activity.category())
                        .unwrap_or(Category::Other);
                    let mut activity = Activity::new(category);
                    activity.set_note(draft);
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

    fn clear_day_note(&mut self, date: NaiveDate) -> anyhow::Result<()> {
        self.store.clear_day_note(date)?;
        if let Some(day) = self.state.days.get_mut(&date) {
            day.clear_note();
        }
        cleanup_day(&mut self.state.days, date);
        Ok(())
    }

    fn clear_focused_value(&mut self) -> anyhow::Result<()> {
        match self.state.cursor.hour {
            Some(hour) => self.clear_hour(self.state.cursor.date, hour),
            None => self.clear_day_note(self.state.cursor.date),
        }
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

fn picker_default_selection(state: &State, target: &NoteTarget) -> CategoryPickerSelection {
    match *target {
        NoteTarget::Day { .. } => CategoryPickerSelection::AddNote,
        NoteTarget::Hour { date, hour } => state
            .activity(date, hour)
            .map(|act| CategoryPickerSelection::Category(act.category()))
            .unwrap_or(CategoryPickerSelection::Category(Category::Other)),
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
    match *target {
        NoteTarget::Day { .. } => CategoryPickerSelection::AddNote,
        NoteTarget::Hour { .. } => match selected {
            CategoryPickerSelection::Category(category) => {
                CategoryPickerSelection::Category(prev_category(category))
            }
            CategoryPickerSelection::AddNote => CategoryPickerSelection::Category(Category::Other),
        },
    }
}

fn move_picker_down(
    target: &NoteTarget,
    selected: CategoryPickerSelection,
) -> CategoryPickerSelection {
    match *target {
        NoteTarget::Day { .. } => CategoryPickerSelection::AddNote,
        NoteTarget::Hour { .. } => match selected {
            CategoryPickerSelection::Category(Category::Other) => {
                CategoryPickerSelection::AddNote
            }
            CategoryPickerSelection::Category(category) => {
                CategoryPickerSelection::Category(next_category(category))
            }
            CategoryPickerSelection::AddNote => CategoryPickerSelection::AddNote,
        },
    }
}

fn prev_category(category: Category) -> Category {
    Category::from_u8((category.as_u8() + 9) % 10).unwrap_or(Category::Other)
}

fn next_category(category: Category) -> Category {
    Category::from_u8((category.as_u8() + 1) % 10).unwrap_or(Category::Sleep)
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
