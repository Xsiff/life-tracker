use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    controller::{CategoryPickerSelection, NoteTarget},
    domain::Category,
};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, target: &NoteTarget, selected: CategoryPickerSelection) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(title(target))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    if matches!(target, NoteTarget::Hour { .. }) {
        for category in Category::ALL {
            let is_selected = selected == CategoryPickerSelection::Category(category);
            let prefix = if is_selected { ">" } else { " " };
            let style = picker_style(is_selected, Style::default().fg(theme::color(category)));
            lines.push(Line::from(Span::styled(
                format!("{prefix} {} {}", category.digit(), category.label()),
                style,
            )));
        }
        lines.push(Line::raw("────────────────────────────"));
    }

    let note_selected = selected == CategoryPickerSelection::AddNote;
    let note_prefix = if note_selected { ">" } else { " " };
    lines.push(Line::from(Span::styled(
        format!("{note_prefix} [+] add note"),
        picker_style(note_selected, theme::header_style()),
    )));
    lines.push(Line::raw("⏎ confirm   Esc cancel"));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn title(target: &NoteTarget) -> String {
    match target {
        NoteTarget::Day { date } => format!(" Day - {} ", date.format("%d.%m.%Y")),
        NoteTarget::Hour { hour, .. } => format!(" Set activity - {hour:02}.00 "),
    }
}

fn picker_style(is_selected: bool, base: Style) -> Style {
    if is_selected {
        theme::selected(base)
    } else {
        base
    }
}
