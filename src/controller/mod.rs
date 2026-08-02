mod state;

use std::collections::BTreeMap;

use chrono::{Local, NaiveDate, Timelike};

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
            (Overlay::CategoryPicker { date, hour, selected }, Action::MoveUp) => {
                let selected = match selected {
                    CategoryPickerSelection::Category(category) => {
                        CategoryPickerSelection::Category(prev_category(category))
                    }
                    CategoryPickerSelection::AddNote => CategoryPickerSelection::AddNote,
                };
                self.state.overlay = Some(Overlay::CategoryPicker {
                    date,
                    hour,
                    selected,
                });
            }
            (Overlay::CategoryPicker { date, hour, selected }, Action::MoveDown) => {
                let selected = match selected {
                    CategoryPickerSelection::Category(category) => {
                        CategoryPickerSelection::Category(next_category(category))
                    }
                    CategoryPickerSelection::AddNote => CategoryPickerSelection::AddNote,
                };
                self.state.overlay = Some(Overlay::CategoryPicker {
                    date,
                    hour,
                    selected,
                });
            }
            (Overlay::CategoryPicker { date, hour, .. }, Action::Digit(n)) => {
                if let Some(category) = Category::from_digit(n) {
                    self.state.overlay = Some(Overlay::CategoryPicker {
                        date,
                        hour,
                        selected: CategoryPickerSelection::Category(category),
                    });
                }
            }
            (Overlay::CategoryPicker { date, hour, selected }, Action::Confirm) => {
                match selected {
                    CategoryPickerSelection::Category(category) => {
                        let activity = Activity::new(category);
                        self.store.set_hour(date, hour, &activity)?;
                        ensure_day(&mut self.state.days, date).set_hour(hour, activity);
                        self.state.overlay = None;
                        self.state.cursor.date = date;
                        self.state.cursor.hour = Some(hour);
                    }
                    CategoryPickerSelection::AddNote => {
                        self.state.overlay = Some(Overlay::NoteEditor {
                            target: NoteTarget::Hour { date, hour },
                            draft: self
                                .state
                                .activity(date, hour)
                                .and_then(|activity| activity.note())
                                .unwrap_or("")
                                .to_string(),
                            cursor: self
                                .state
                                .activity(date, hour)
                                .and_then(|activity| activity.note())
                                .unwrap_or("")
                                .len(),
                        });
                    }
                }
            }
            (Overlay::CategoryPicker { date, hour, .. }, Action::Cancel) => {
                self.state.overlay = None;
                self.state.cursor.date = date;
                self.state.cursor.hour = Some(hour);
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
                let hour = self.state.cursor.hour.unwrap_or_else(current_hour);
                let selected = self
                    .state
                    .activity(date, hour)
                    .map(|act| CategoryPickerSelection::Category(act.category()))
                    .unwrap_or(CategoryPickerSelection::Category(Category::Other));
                self.state.overlay = Some(Overlay::CategoryPicker {
                    date,
                    hour,
                    selected,
                });
            }
            Action::CycleView => {}
            Action::Char('n') | Action::Char('N') => {
                self.open_hour_note_editor();
            }
            Action::Char('x') | Action::Char('X') => {
                self.clear_hour(self.state.cursor.date, self.state.cursor.hour.unwrap_or(0))?;
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
        let hour = self.state.cursor.hour.unwrap_or(0) as i16 + i16::from(delta);
        if hour < 0 {
            self.state.cursor.hour = Some(23);
            self.state.cursor.date -= chrono::Duration::days(1);
        } else if hour > 23 {
            self.state.cursor.hour = Some(0);
            self.state.cursor.date += chrono::Duration::days(1);
        } else {
            self.state.cursor.hour = Some(hour as u8);
        }
    }

    fn open_hour_note_editor(&mut self) {
        let date = self.state.cursor.date;
        let hour = self.state.cursor.hour.unwrap_or(0);
        let draft = self
            .state
            .activity(date, hour)
            .and_then(|activity| activity.note())
            .unwrap_or("")
            .to_string();
        let cursor = draft.len();
        self.state.overlay = Some(Overlay::NoteEditor {
            target: NoteTarget::Hour { date, hour },
            draft,
            cursor,
        });
    }

    fn restore_focus(&mut self, target: NoteTarget) {
        match target {
            NoteTarget::Day { date } => {
                self.state.cursor.date = date;
                self.state.cursor.hour = Some(current_hour());
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

fn current_hour() -> u8 {
    Local::now().hour() as u8
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
