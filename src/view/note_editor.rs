use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::controller::{NoteTarget, State};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &State,
    target: &NoteTarget,
    draft: &str,
    cursor: usize,
) {
    frame.render_widget(Clear, area);
    let block = Block::default().title(title(state, target)).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let footer_height = 2u16.min(inner.height);
    let text_height = inner.height.saturating_sub(footer_height);
    let text_area = Rect { x: inner.x, y: inner.y, width: inner.width, height: text_height };
    let footer_area =
        Rect { x: inner.x, y: inner.y + text_height, width: inner.width, height: footer_height };

    let (lines, cursor_line) = draft_lines_with_cursor(draft, cursor, inner.width as usize);
    if text_area.height > 0 {
        let max_scroll = lines.len().saturating_sub(text_area.height as usize);
        let scroll = cursor_line.saturating_sub(text_area.height as usize / 2).min(max_scroll);
        frame.render_widget(Paragraph::new(lines).scroll((scroll as u16, 0)), text_area);
    }

    if footer_area.height > 0 {
        let mut footer_lines = vec![Line::raw(separator_line(footer_area.width))];
        if footer_area.height > 1 {
            footer_lines.push(Line::raw("⏎ save   Esc cancel"));
        }
        frame.render_widget(Paragraph::new(footer_lines), footer_area);
    }
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

fn draft_lines_with_cursor(
    draft: &str,
    cursor: usize,
    width: usize,
) -> (Vec<Line<'static>>, usize) {
    if width == 0 {
        return (vec![], 0);
    }

    let chars: Vec<char> = draft.chars().collect();
    let cursor = cursor.min(chars.len());
    let mut lines = vec![Vec::<Span<'static>>::new()];
    let mut line_width = 0usize;
    let mut cursor_line = 0usize;

    for (index, ch) in chars.iter().enumerate() {
        if index == cursor {
            cursor_line = lines.len().saturating_sub(1);
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
        cursor_line = lines.len().saturating_sub(1);
        push_wrapped_span(&mut lines, &mut line_width, cursor_span(), width);
    }

    (lines.into_iter().map(Line::from).collect(), cursor_line)
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
    Span::raw("█")
}
