use crate::domain::Action;

use crate::controller::{
    note_buffer::{
        erase_char, erase_word, insert_char, move_note_cursor_left, move_note_cursor_right,
        move_note_cursor_vertical, move_note_cursor_word_left, move_note_cursor_word_right,
        scroll_note_cursor_vertical,
    },
    Controller, NoteTarget, Overlay,
};

impl Controller {
    pub(super) fn update_note_editor(
        &mut self,
        target: NoteTarget,
        draft: String,
        cursor: usize,
        action: Action,
    ) -> anyhow::Result<()> {
        match action {
            Action::Char(c) => {
                let mut draft = draft;
                let mut cursor = cursor;
                insert_char(&mut draft, &mut cursor, c);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::Digit(n) => {
                let mut draft = draft;
                let mut cursor = cursor;
                insert_char(&mut draft, &mut cursor, char::from_digit(n as u32, 10).unwrap());
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::InsertNewline => {
                let mut draft = draft;
                let mut cursor = cursor;
                insert_char(&mut draft, &mut cursor, '\n');
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::Erase => {
                let mut draft = draft;
                let mut cursor = cursor;
                erase_char(&mut draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::DeleteWord => {
                let mut draft = draft;
                let mut cursor = cursor;
                erase_word(&mut draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::MoveLeft => {
                let mut cursor = cursor;
                move_note_cursor_left(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::MoveRight => {
                let mut cursor = cursor;
                move_note_cursor_right(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::MoveWordLeft => {
                let mut cursor = cursor;
                move_note_cursor_word_left(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::MoveWordRight => {
                let mut cursor = cursor;
                move_note_cursor_word_right(&draft, &mut cursor);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::MoveUp => {
                let mut cursor = cursor;
                move_note_cursor_vertical(&draft, &mut cursor, -1);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::MoveDown => {
                let mut cursor = cursor;
                move_note_cursor_vertical(&draft, &mut cursor, 1);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::Scroll(delta) => {
                let mut cursor = cursor;
                scroll_note_cursor_vertical(&draft, &mut cursor, delta);
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
            Action::Confirm => {
                self.save_note(target, draft)?;
            }
            Action::Cancel => {
                self.state.overlay = None;
                self.restore_focus(target);
            }
            _ => {
                self.state.overlay = Some(Overlay::NoteEditor { target, draft, cursor });
            }
        }

        Ok(())
    }
}
