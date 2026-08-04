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

    let mut lines = draft_lines_with_cursor(draft, cursor, inner.width as usize);
    while lines.len() < inner.height.saturating_sub(2) as usize {
        lines.push(Line::raw(""));
    }
    lines.push(Line::raw(separator_line(inner.width)));
    lines.push(Line::raw("⏎ save   Esc cancel"));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn separator_line(width: u16) -> String {
    "─".repeat(width as usize)
}

fn title(state: &State, target: &NoteTarget) -> String {
    match target {
        NoteTarget::Day { date } => format!(" Note - {} ", date.format("%a, %b %-d %Y")),
        NoteTarget::Hour { date, hour } => match state.activity(*date, *hour) {
            Some(activity) => match activity.category() {
                Some(category) => format!(" Note - {hour:02}:00 {} ", category.label()),
                None => format!(" Note - {hour:02}:00 "),
            },
            None => format!(" Note - {hour:02}:00 "),
        },
    }
}

fn draft_lines_with_cursor(draft: &str, cursor: usize, width: usize) -> Vec<Line<'static>> {
    if width == 0 {
        return vec![];
    }

    let chars: Vec<char> = draft.chars().collect();
    let cursor = cursor.min(chars.len());
    let mut lines = vec![Vec::<Span<'static>>::new()];
    let mut line_width = 0usize;

    for (index, ch) in chars.iter().enumerate() {
        if index == cursor {
            push_wrapped_span(&mut lines, &mut line_width, cursor_span(), width);
        }

        if *ch == '\n' {
            lines.push(Vec::new());
            line_width = 0;
            continue;
        }

        if line_width >= width {
            lines.push(Vec::new());
            line_width = 0;
        }

        push_wrapped_span(&mut lines, &mut line_width, Span::raw(ch.to_string()), width);
    }

    if cursor == chars.len() {
        push_wrapped_span(&mut lines, &mut line_width, cursor_span(), width);
    }

    lines.into_iter().map(Line::from).collect()
}

fn push_wrapped_span(
    lines: &mut Vec<Vec<Span<'static>>>,
    line_width: &mut usize,
    span: Span<'static>,
    width: usize,
) {
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    if *line_width >= width {
        lines.push(Vec::new());
        *line_width = 0;
    }

    *line_width += 1;
    lines.last_mut().expect("at least one line").push(span);
}

fn cursor_span() -> Span<'static> {
    Span::styled("|", cursor_style())
}

fn cursor_style() -> Style {
    theme::selected(Style::default())
}
