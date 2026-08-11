use chrono::NaiveDate;

use crate::domain::Action;

use super::{
    persistence::note_draft, picker::picker_default_selection, Controller, NoteTarget, Overlay,
};

impl Controller {
    pub(super) fn update_base(&mut self, action: Action) -> anyhow::Result<()> {
        self.update_calendar(action)
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
            Action::Scroll(delta) => {
                self.scroll_cursor_days(delta);
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
        let (date, hour) = move_cursor_hour(self.state.cursor.date, self.state.cursor.hour, delta);
        self.state.cursor.date = date;
        self.state.cursor.hour = hour;
    }

    fn scroll_cursor_days(&mut self, delta: i32) {
        self.state.cursor.date += chrono::Duration::days(delta as i64);
    }

    pub(super) fn open_note_editor(&mut self, target: NoteTarget) {
        let draft = note_draft(&self.state, &target);
        let cursor = draft.len();
        self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
    }

    pub(super) fn restore_focus(&mut self, target: NoteTarget) {
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::move_cursor_hour;

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
