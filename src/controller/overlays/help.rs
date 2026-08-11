use crate::domain::Action;

use crate::controller::{Controller, Overlay};

impl Controller {
    pub(super) fn update_help(&mut self, action: Action) {
        match action {
            Action::Confirm | Action::Cancel => {
                self.state.overlay = None;
            }
            _ => {
                self.state.overlay = Some(Overlay::Help);
            }
        }
    }
}
