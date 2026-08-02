use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::domain::Category;

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, hour: u8, selected: Category) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" Set activity - {hour:02}.00 "))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    for category in Category::ALL {
        let prefix = if category == selected { ">" } else { " " };
        let style = if category == selected {
            theme::selected(ratatui::style::Style::default().fg(theme::color(category)))
        } else {
            ratatui::style::Style::default().fg(theme::color(category))
        };
        lines.push(Line::from(Span::styled(
            format!("{prefix} {} {}", category.digit(), category.label()),
            style,
        )));
    }
    lines.push(Line::raw("────────────────────────────"));
    lines.push(Line::from(Span::styled(
        "[+] add note",
        theme::header_style(),
    )));
    lines.push(Line::raw("⏎ confirm   Esc cancel"));

    frame.render_widget(Paragraph::new(lines), inner);
}
