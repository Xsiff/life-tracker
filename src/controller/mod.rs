mod base;
mod note_buffer;
mod overlay;
mod persistence;
mod picker;
mod state;

use chrono::{Local, NaiveDate};

use crate::{domain::Action, storage::Store};

#[cfg(test)]
pub use state::Cursor;
pub use state::{CategoryPickerSelection, NoteTarget, Overlay, State};

pub struct Controller {
    state: State,
    store: Store,
}

impl Controller {
    pub fn new(store: Store) -> anyhow::Result<Self> {
        let days = store.load_all()?;
        Ok(Self { state: State::new(today(), days), store })
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
}

fn today() -> NaiveDate {
    Local::now().date_naive()
}
