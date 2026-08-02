use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::controller::{NoteTarget, State};

pub fn render(frame: &mut Frame, area: Rect, state: &State, target: &NoteTarget, draft: &str) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(title(state, target))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = draft.lines().map(Line::raw).collect::<Vec<_>>();
    while lines.len() < inner.height.saturating_sub(2) as usize {
        lines.push(Line::raw(""));
    }
    lines.push(Line::raw("────────────────────────────"));
    lines.push(Line::raw("⏎ save   Esc cancel"));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn title(state: &State, target: &NoteTarget) -> String {
    match target {
        NoteTarget::Day { date } => format!(" Note - {} ", date.format("%a, %b %-d %Y")),
        NoteTarget::Hour { date, hour } => match state.activity(*date, *hour) {
            Some(activity) => format!(" Note - {hour:02}:00 {} ", activity.category.label()),
            None => format!(" Note - {hour:02}:00 "),
        },
    }
}
