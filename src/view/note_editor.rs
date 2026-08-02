use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::controller::{NoteTarget, State};

use super::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &State,
    target: &NoteTarget,
    draft: &str,
    cursor: usize,
) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(title(state, target))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = draft_lines_with_cursor(draft, cursor);
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
            Some(activity) => format!(" Note - {hour:02}:00 {} ", activity.category().label()),
            None => format!(" Note - {hour:02}:00 "),
        },
    }
}

fn draft_lines_with_cursor(draft: &str, cursor: usize) -> Vec<Line<'static>> {
    let chars: Vec<char> = draft.chars().collect();
    let cursor = cursor.min(chars.len());
    let mut lines = vec![Vec::<Span<'static>>::new()];

    for (index, ch) in chars.iter().enumerate() {
        if index == cursor {
            current_line(&mut lines).push(cursor_span());
        }

        if *ch == '\n' {
            lines.push(Vec::new());
            continue;
        }

        current_line(&mut lines).push(Span::raw(ch.to_string()));
    }

    if cursor == chars.len() {
        current_line(&mut lines).push(cursor_span());
    }

    lines.into_iter().map(Line::from).collect()
}

fn current_line<'a>(lines: &'a mut Vec<Vec<Span<'static>>>) -> &'a mut Vec<Span<'static>> {
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    lines.last_mut().expect("at least one line")
}

fn cursor_span() -> Span<'static> {
    Span::styled("|", cursor_style())
}

fn cursor_style() -> Style {
    theme::selected(Style::default())
}
