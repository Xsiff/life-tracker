use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Frame,
};

use crate::controller::State;

use super::calendar_lines::{build_grid_lines, render_legend};

pub(crate) const DATE_WIDTH: usize = 16;
pub(crate) const HOUR_CONTENT_WIDTH: usize = 5;
pub(crate) const MIN_VISIBLE_DATE_ROWS: usize = 4;
pub(crate) const MIN_VISIBLE_HOURS: usize = 4;

pub fn render(frame: &mut Frame, area: Rect, state: &State, now: &chrono::DateTime<Local>) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(24)])
        .split(area);

    frame.render_widget(
        Paragraph::new(build_grid_lines(
            state,
            now,
            sections[0].width as usize,
            sections[0].height as usize,
        )),
        sections[0],
    );
    if sections.len() > 1 && sections[1].width >= 12 {
        render_legend(frame, sections[1]);
    }
}
