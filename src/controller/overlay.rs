use crate::domain::{Action, Activity, Category};

use super::{
    note_buffer::{
        erase_char, erase_word, insert_char, move_note_cursor_left, move_note_cursor_right,
        move_note_cursor_vertical, move_note_cursor_word_left, move_note_cursor_word_right,
        scroll_note_cursor_vertical,
    },
    picker::{move_picker_down, move_picker_up, scroll_picker},
    CategoryPickerSelection, Controller, NoteTarget, Overlay,
};

impl Controller {
    pub(super) fn update_overlay(&mut self, action: Action) -> anyhow::Result<()> {
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
            (Overlay::CategoryPicker { target, selected }, Action::Scroll(delta)) => {
                let selected = scroll_picker(&target, selected, delta);
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
                        self.write_hour_state(date, hour, activity);
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
            (Overlay::NoteEditor { target, draft, cursor }, Action::Scroll(delta)) => {
                let mut cursor = cursor;
                scroll_note_cursor_vertical(&draft, &mut cursor, delta);
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
}

#[cfg(test)]
mod tests {
    use crate::{domain::Action, storage::Store};

    use super::{Controller, Overlay};

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
