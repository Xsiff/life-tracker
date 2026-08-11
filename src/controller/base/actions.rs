use crate::domain::Action;

use super::navigation::move_cursor_hour;
use crate::controller::{
    persistence::note_draft, picker::picker_default_selection, Controller, NoteTarget, Overlay,
};

impl Controller {
    pub(in crate::controller) fn update_base(&mut self, action: Action) -> anyhow::Result<()> {
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

    pub(in crate::controller) fn open_note_editor(&mut self, target: NoteTarget) {
        let draft = note_draft(&self.state, &target);
        let cursor = draft.len();
        self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
    }

    pub(in crate::controller) fn restore_focus(&mut self, target: NoteTarget) {
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
