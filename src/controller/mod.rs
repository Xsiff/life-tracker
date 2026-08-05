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
            (Overlay::Help, Action::Confirm) | (Overlay::Help, Action::Cancel) => {
                self.state.overlay = None;
            }
            (Overlay::Help, _) => {
                self.state.overlay = Some(Overlay::Help);
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::Char(c)) => {
                let mut draft = draft;
                let mut cursor = cursor;
                insert_char(&mut draft, &mut cursor, c);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::Digit(n)) => {
                let mut draft = draft;
                let mut cursor = cursor;
                insert_char(&mut draft, &mut cursor, char::from_digit(n as u32, 10).unwrap());
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
            (Overlay::NoteEditor { target, draft, cursor }, Action::DeleteWord) => {
                let mut draft = draft;
                let mut cursor = cursor;
                erase_word(&mut draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::MoveLeft) => {
                let mut cursor = cursor;
                move_note_cursor_left(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::MoveRight) => {
                let mut cursor = cursor;
                move_note_cursor_right(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::MoveWordLeft) => {
                let mut cursor = cursor;
                move_note_cursor_word_left(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::MoveWordRight) => {
                let mut cursor = cursor;
                move_note_cursor_word_right(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::MoveUp) => {
                let mut cursor = cursor;
                move_note_cursor_vertical(&draft, &mut cursor, -1);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            (Overlay::NoteEditor { target, draft, cursor }, Action::MoveDown) => {
                let mut cursor = cursor;
                move_note_cursor_vertical(&draft, &mut cursor, 1);
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
            Action::Char('?') => {
                self.state.overlay = Some(Overlay::Help);
            }
            Action::Char('q') | Action::Char('Q') => self.state.quit = true,
            Action::Tick
            | Action::Cancel
            | Action::InsertNewline
            | Action::Erase
            | Action::DeleteWord
            | Action::MoveWordLeft
            | Action::MoveWordRight
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

fn erase_word(draft: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let mut pos = *cursor;
    while let Some((start, ch)) = prev_char_at(draft, pos) {
        if !ch.is_whitespace() {
            break;
        }
        pos = start;
        if pos == 0 {
            draft.drain(0..*cursor);
            *cursor = 0;
            return;
        }
    }

    while let Some((start, ch)) = prev_char_at(draft, pos) {
        if ch.is_whitespace() {
            break;
        }
        pos = start;
        if pos == 0 {
            break;
        }
    }

    draft.drain(pos..*cursor);
    *cursor = pos;
}

fn move_note_cursor_left(draft: &str, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let new_cursor = draft[..*cursor]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    *cursor = new_cursor;
}

fn move_note_cursor_right(draft: &str, cursor: &mut usize) {
    if *cursor >= draft.len() {
        return;
    }

    let next = draft[*cursor..]
        .chars()
        .next()
        .map(|ch| *cursor + ch.len_utf8())
        .unwrap_or(*cursor);
    *cursor = next;
}

fn move_note_cursor_word_left(draft: &str, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let mut pos = *cursor;
    while let Some((start, ch)) = prev_char_at(draft, pos) {
        if !ch.is_whitespace() {
            break;
        }
        pos = start;
        if pos == 0 {
            *cursor = 0;
            return;
        }
    }

    while let Some((start, ch)) = prev_char_at(draft, pos) {
        if ch.is_whitespace() {
            break;
        }
        pos = start;
        if pos == 0 {
            break;
        }
    }

    *cursor = pos;
}

fn move_note_cursor_word_right(draft: &str, cursor: &mut usize) {
    let mut pos = *cursor;
    if pos >= draft.len() {
        return;
    }

    match next_char_at(draft, pos) {
        Some((_, ch)) if ch.is_whitespace() => {
            while let Some((start, ch)) = next_char_at(draft, pos) {
                if !ch.is_whitespace() {
                    break;
                }
                pos = start + ch.len_utf8();
                if pos >= draft.len() {
                    *cursor = draft.len();
                    return;
                }
            }
        }
        Some(_) => {
            while let Some((start, ch)) = next_char_at(draft, pos) {
                if ch.is_whitespace() {
                    break;
                }
                pos = start + ch.len_utf8();
                if pos >= draft.len() {
                    *cursor = draft.len();
                    return;
                }
            }
        }
        None => return,
    }

    *cursor = pos;
}

fn move_note_cursor_vertical(draft: &str, cursor: &mut usize, delta: i8) {
    if delta == 0 {
        return;
    }

    let chars: Vec<char> = draft.chars().collect();
    let cursor_char_idx = byte_to_char_index(draft, *cursor);
    let (line_idx, col_idx) = char_index_to_line_col(&chars, cursor_char_idx);
    let target_line = if delta.is_negative() {
        line_idx.checked_sub(1)
    } else {
        line_idx.checked_add(1)
    };

    let Some(target_line) = target_line else {
        return;
    };

    let Some(target_char_idx) = line_col_to_char_index(&chars, target_line, col_idx) else {
        return;
    };

    *cursor = char_to_byte_index(&chars, target_char_idx);
}

fn byte_to_char_index(draft: &str, byte_idx: usize) -> usize {
    draft[..byte_idx.min(draft.len())].chars().count()
}

fn prev_char_at(draft: &str, idx: usize) -> Option<(usize, char)> {
    if idx == 0 {
        return None;
    }

    draft[..idx].char_indices().last()
}

fn next_char_at(draft: &str, idx: usize) -> Option<(usize, char)> {
    if idx >= draft.len() {
        return None;
    }

    draft[idx..]
        .char_indices()
        .next()
        .map(|(rel_idx, ch)| (idx + rel_idx, ch))
}

fn char_index_to_line_col(chars: &[char], char_idx: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;

    for (idx, ch) in chars.iter().enumerate() {
        if idx == char_idx {
            return (line, col);
        }
        if *ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

fn line_col_to_char_index(chars: &[char], target_line: usize, target_col: usize) -> Option<usize> {
    let mut line = 0usize;
    let mut col = 0usize;

    for (idx, ch) in chars.iter().enumerate() {
        if line == target_line && col == target_col {
            return Some(idx);
        }

        if *ch == '\n' {
            if line == target_line {
                return Some(idx);
            }
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    if line == target_line && col >= target_col {
        Some(chars.len())
    } else {
        None
    }
}

fn char_to_byte_index(chars: &[char], char_idx: usize) -> usize {
    chars.iter().take(char_idx).map(|ch| ch.len_utf8()).sum()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{domain::Action, storage::Store};

    use super::{
        erase_word, insert_char, move_cursor_hour, move_note_cursor_left,
        move_note_cursor_right, move_note_cursor_vertical, move_note_cursor_word_left,
        move_note_cursor_word_right, Controller, Overlay,
    };

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

    #[test]
    fn note_cursor_moves_by_character_left_and_right() {
        let draft = "abçd";
        let mut cursor = draft.len();

        move_note_cursor_left(draft, &mut cursor);
        assert_eq!(cursor, "abç".len());

        move_note_cursor_left(draft, &mut cursor);
        assert_eq!(cursor, "ab".len());

        move_note_cursor_right(draft, &mut cursor);
        assert_eq!(cursor, "abç".len());
    }

    #[test]
    fn note_cursor_moves_between_lines_with_column_clamping() {
        let draft = "ab\ncde\nf";

        let mut cursor = "ab\ncd".len();
        move_note_cursor_vertical(draft, &mut cursor, -1);
        assert_eq!(cursor, "ab".len());

        let mut cursor = 1usize;
        move_note_cursor_vertical(draft, &mut cursor, 1);
        assert_eq!(cursor, "ab\nc".len());
    }

    #[test]
    fn note_cursor_moves_by_word_left_and_right() {
        let draft = "alpha beta  gamma";

        let mut cursor = draft.len();
        move_note_cursor_word_left(draft, &mut cursor);
        assert_eq!(cursor, "alpha beta  ".len());

        move_note_cursor_word_left(draft, &mut cursor);
        assert_eq!(cursor, "alpha ".len());

        let mut cursor = "alpha".len();
        move_note_cursor_word_right(draft, &mut cursor);
        assert_eq!(cursor, "alpha ".len());

        move_note_cursor_word_right(draft, &mut cursor);
        assert_eq!(cursor, "alpha beta".len());
    }

    #[test]
    fn delete_word_removes_previous_chunk_and_leaves_cursor_at_boundary() {
        let mut draft = String::from("alpha beta  gamma");
        let mut cursor = draft.len();

        erase_word(&mut draft, &mut cursor);
        assert_eq!(draft, "alpha beta  ");
        assert_eq!(cursor, "alpha beta  ".len());

        erase_word(&mut draft, &mut cursor);
        assert_eq!(draft, "alpha ");
        assert_eq!(cursor, "alpha ".len());
    }

    #[test]
    fn question_mark_opens_and_closes_help_overlay() {
        let store = Store::in_memory().expect("in-memory store");
        let mut controller = Controller::new(store).expect("controller");

        controller.update(Action::Char('?')).expect("open help");
        assert!(matches!(controller.state().overlay, Some(Overlay::Help)));

        controller.update(Action::Cancel).expect("close help");
        assert!(controller.state().overlay.is_none());
    }
}
