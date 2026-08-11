use crate::domain::{Action, Activity, Category};

use crate::controller::{
    picker::{move_picker_down, move_picker_up, scroll_picker},
    CategoryPickerSelection, Controller, NoteTarget, Overlay,
};

impl Controller {
    pub(super) fn update_category_picker(
        &mut self,
        target: NoteTarget,
        selected: CategoryPickerSelection,
        action: Action,
    ) -> anyhow::Result<()> {
        match action {
            Action::MoveUp => {
                let selected = move_picker_up(&target, selected);
                self.state.overlay = Some(Overlay::CategoryPicker { target, selected });
            }
            Action::MoveDown => {
                let selected = move_picker_down(&target, selected);
                self.state.overlay = Some(Overlay::CategoryPicker { target, selected });
            }
            Action::Scroll(delta) => {
                let selected = scroll_picker(&target, selected, delta);
                self.state.overlay = Some(Overlay::CategoryPicker { target, selected });
            }
            Action::Digit(n) => {
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
            Action::Confirm => match selected {
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
            Action::Cancel => {
                self.state.overlay = None;
                self.restore_focus(target);
            }
            _ => {
                self.state.overlay = Some(Overlay::CategoryPicker { target, selected });
            }
        }

        Ok(())
    }
}
