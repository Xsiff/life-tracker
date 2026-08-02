use chrono::{Local, Timelike};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::controller::State;

use super::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &State,
    now: &chrono::DateTime<Local>,
) {
    let date = state.cursor.date;
    let day = state.day(date);
    let mut lines = Vec::new();

    for row in 0..8 {
        let mut spans = Vec::new();
        for col in 0..3 {
            let hour = (row + col * 8) as u8;
            spans.push(render_hour_span(state, day, hour, now));
            if col < 2 {
                spans.push(Span::raw("  "));
            }
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_hour_span(
    state: &State,
    day: Option<&crate::domain::Day>,
    hour: u8,
    now: &chrono::DateTime<Local>,
) -> Span<'static> {
    let activity = day.and_then(|entry| entry.activity(hour));
    let is_selected = state.cursor.hour == Some(hour);
    let is_now_hour = state.cursor.date == now.date_naive() && hour == now.hour() as u8;
    let marker = if is_selected {
        "◀"
    } else if is_now_hour {
        "●"
    } else {
        " "
    };

    let content = match activity {
        Some(activity) => {
            let note = if activity.has_note() { "*" } else { " " };
            format!(
                "{hour:02} {:<16}{note}{marker}",
                activity.category().label()
            )
        }
        None => format!("{hour:02} {:<16} {marker}", ""),
    };

    let style = match activity {
        Some(activity) => {
            let base = Style::default().fg(theme::color(activity.category()));
            if is_selected {
                theme::selected(base)
            } else if is_now_hour {
                base.bg(ratatui::style::Color::Indexed(236))
            } else {
                base
            }
        }
        None => {
            let base = theme::empty_style();
            if is_selected {
                theme::selected(base)
            } else if is_now_hour {
                base.bg(ratatui::style::Color::Indexed(236))
            } else {
                base
            }
        }
    };

    Span::styled(content, style)
}
