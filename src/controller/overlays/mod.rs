mod category_picker;
mod help;
mod note_editor;

use crate::domain::Action;

use crate::controller::{Controller, Overlay};

impl Controller {
    pub(super) fn update_overlay(&mut self, action: Action) -> anyhow::Result<()> {
        let Some(overlay) = self.state.overlay.take() else {
            return Ok(());
        };

        match overlay {
            Overlay::CategoryPicker { target, selected } => {
                self.update_category_picker(target, selected, action)?;
            }
            Overlay::Help => {
                self.update_help(action);
            }
            Overlay::NoteEditor { target, draft, cursor } => {
                self.update_note_editor(target, draft, cursor, action)?;
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
