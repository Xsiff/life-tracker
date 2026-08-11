#[cfg(test)]
mod tests;

pub mod calendar_layout;
pub mod calendar_lines;
pub mod calendar_view;
pub mod category_picker;
pub mod help_popup;
pub mod note_editor;
pub mod overlay_layout;
pub mod status_bar;
pub mod theme;

use chrono::{DateTime, Datelike, Local};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::controller::{Overlay, State};

use self::overlay_layout::overlay_rect;

pub fn render(frame: &mut Frame, state: &State) {
    let now = Local::now();
    render_with_now(frame, state, &now);
}

pub(super) fn render_with_now(frame: &mut Frame, state: &State, now: &DateTime<Local>) {
    let title = format!(
        " life-tracker ───────────────────────── {} {} ",
        month(now.month0() as usize),
        now.year()
    );

    let block = Block::default().title(title).borders(Borders::ALL);
    let area = frame.size();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let matrix_area = sections[0];
    calendar_view::render(frame, matrix_area, state, now);
    status_bar::render(frame, sections[1], state, now);

    if let Some(overlay) = &state.overlay {
        render_overlay(frame, state, overlay, overlay_rect(matrix_area, overlay));
    }
}

fn render_overlay(frame: &mut Frame, state: &State, overlay: &Overlay, area: Rect) {
    match overlay {
        Overlay::CategoryPicker { target, selected } => {
            category_picker::render(frame, area, target, *selected);
        }
        Overlay::Help => {
            help_popup::render(frame, area);
        }
        Overlay::NoteEditor { target, draft, cursor } => {
            note_editor::render(frame, area, state, target, draft, *cursor);
        }
    }
}

fn month(index: usize) -> &'static str {
    ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][index]
}
