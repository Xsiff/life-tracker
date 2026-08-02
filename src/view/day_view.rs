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
    let mut lines = vec![Line::from(Span::styled(
        format!("Focused day: {}", date.format("%d.%m.%Y")),
        theme::header_style(),
    ))];

    for hour in 0..24u8 {
        lines.push(Line::from(render_hour_span(state, day, hour, now)));
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
    let day_has_note = day.and_then(|d| d.note()).is_some();
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
            let note = if activity.has_note() || day_has_note { "*" } else { " " };
            let label = activity
                .category()
                .map(|category| category.label())
                .unwrap_or("");
            format!("{hour:02}.00 {label:<16}{note}{marker}")
        }
        None => {
            let note = if day_has_note { "*" } else { " " };
            format!("{hour:02}.00 {:<16}{note} {marker}", "")
        }
    };

    let style = match activity {
        Some(activity) => {
            let base = activity
                .category()
                .map(theme::cell_style)
                .unwrap_or_else(theme::empty_style);
            if is_selected {
                theme::selected(base)
            } else if is_now_hour {
                theme::now_cell(base)
            } else {
                base
            }
        }
        None => {
            let base = theme::empty_style();
            if is_selected {
                theme::selected(base)
            } else if is_now_hour {
                theme::now_cell(base)
            } else {
                base
            }
        }
    };

    Span::styled(content, style)
}
