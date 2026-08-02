use chrono::{Datelike, Local, NaiveDate};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{controller::State, domain::week_window_centered};

use super::theme;

const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const CELL_WIDTH: usize = 7;
const ROW_LABEL_WIDTH: usize = 5;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &State,
    now: &chrono::DateTime<Local>,
) {
    let weeks = week_window_centered(state.cursor.date);
    let mut lines = Vec::with_capacity(8);

    let header = weeks
        .iter()
        .map(|week| format!("{:^CELL_WIDTH$}", format!("W{:02}", week.iso_week().week())))
        .collect::<Vec<_>>()
        .join("");
    lines.push(Line::from(format!("{:>ROW_LABEL_WIDTH$}  {header}", "")));

    for (weekday_idx, weekday_label) in WEEKDAY_LABELS.iter().enumerate() {
        let mut spans = vec![Span::raw(format!("{weekday_label:>ROW_LABEL_WIDTH$}  "))];
        for week_start in weeks {
            let date = week_start + chrono::Duration::days(weekday_idx as i64);
            let marker = if date == now.date_naive() { "●" } else { " " };
            let (cell, style) = render_day_cell(state, date);
            spans.push(Span::styled(
                marker,
                marker_style(date, state.cursor.date, now.date_naive()),
            ));
            spans.push(Span::styled(cell, style));
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn marker_style(date: NaiveDate, selected: NaiveDate, today: NaiveDate) -> Style {
    if date == selected {
        theme::selected(Style::default())
    } else if date == today {
        theme::now_style()
    } else {
        Style::default()
    }
}

fn render_day_cell(state: &State, date: NaiveDate) -> (String, Style) {
    match state.days.get(&date) {
        Some(day) => {
            let fill = match day.filled_hours() {
                0 => "[░░░]".to_string(),
                1..=7 => "[░░░]".to_string(),
                8..=15 => "[▓░░]".to_string(),
                16..=23 => "[▓▓░]".to_string(),
                _ => "[▓▓▓]".to_string(),
            };
            let style = day
                .dominant_category()
                .map(|category| Style::default().fg(theme::color(category)))
                .unwrap_or_else(theme::empty_style);
            if date == state.cursor.date {
                (fill, theme::selected(style))
            } else {
                (fill, style)
            }
        }
        None => {
            let style = if date == state.cursor.date {
                theme::selected(theme::empty_style())
            } else {
                theme::empty_style()
            };
            ("[   ]".to_string(), style)
        }
    }
}
