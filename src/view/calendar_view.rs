use chrono::{Datelike, Local, NaiveDate, Timelike};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{controller::State, domain::Category};

use super::theme;

const DATE_WIDTH: usize = 12;
const HOUR_CONTENT_WIDTH: usize = 5;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &State,
    now: &chrono::DateTime<Local>,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    frame.render_widget(Paragraph::new(build_grid_lines(state, now)), sections[0]);
    frame.render_widget(Paragraph::new(build_legend_lines()), sections[1]);
}

fn build_grid_lines(state: &State, now: &chrono::DateTime<Local>) -> Vec<Line<'static>> {
    let visible_dates = visible_dates(state.cursor.date, 10);
    let mut lines = Vec::new();
    lines.push(build_header_line());
    lines.push(Line::raw(build_rule_line('─', '┼')));

    let mut active_month = None;
    for date in visible_dates {
        if active_month != Some((date.year(), date.month())) {
            if active_month.is_some() {
                lines.push(Line::raw(build_rule_line('═', '╪')));
            }
            active_month = Some((date.year(), date.month()));
            lines.push(Line::from(Span::styled(
                format!("**{} {}**", date.format("%B"), date.year()),
                theme::month_header_style(),
            )));
            lines.push(Line::raw(build_rule_line('═', '╪')));
        }
        lines.push(build_day_line(state, date, now));
        lines.push(Line::raw(build_rule_line('─', '┼')));
    }

    lines
}

fn build_header_line() -> Line<'static> {
    let mut spans = vec![Span::raw(format!("{:<DATE_WIDTH$}│", ""))];
    for hour in 0..24u8 {
        spans.push(Span::styled(
            format!("{hour:>2}.00"),
            theme::header_style(),
        ));
        spans.push(Span::raw("│"));
    }
    Line::from(spans)
}

fn build_day_line(state: &State, date: NaiveDate, now: &chrono::DateTime<Local>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("{:<DATE_WIDTH$}│", date.format("%d.%m.%Y")),
        date_style(date, state.cursor.date, now.date_naive()),
    )];

    for hour in 0..24u8 {
        let activity = state.activity(date, hour);
        let is_selected = state.cursor.date == date && state.cursor.hour == Some(hour);
        let is_now = date == now.date_naive() && hour == now.hour() as u8;
        let (text, style) = cell_content(activity, is_selected, is_now);
        spans.push(Span::styled(text, style));
        spans.push(Span::raw("│"));
    }

    Line::from(spans)
}

fn build_rule_line(fill: char, cross: char) -> String {
    let mut line = String::new();
    line.push_str(&fill.to_string().repeat(DATE_WIDTH));
    line.push(cross);
    for _ in 0..24 {
        line.push_str(&fill.to_string().repeat(HOUR_CONTENT_WIDTH));
        line.push(cross);
    }
    line
}

fn build_legend_lines() -> Vec<Line<'static>> {
    let first = Category::ALL[..5]
        .iter()
        .map(|category| legend_span(*category))
        .collect::<Vec<_>>();
    let second = Category::ALL[5..]
        .iter()
        .map(|category| legend_span(*category))
        .collect::<Vec<_>>();
    vec![Line::from(first), Line::from(second)]
}

fn legend_span(category: Category) -> Span<'static> {
    Span::styled(
        format!(" {}={}  ", category.digit(), category.label()),
        Style::default().fg(theme::color(category)),
    )
}

fn visible_dates(center: NaiveDate, count: usize) -> Vec<NaiveDate> {
    let start = center - chrono::Duration::days((count / 2) as i64);
    (0..count)
        .map(|offset| start + chrono::Duration::days(offset as i64))
        .collect()
}

fn date_style(date: NaiveDate, selected: NaiveDate, today: NaiveDate) -> Style {
    if date == selected {
        theme::selected(Style::default())
    } else if date == today {
        theme::now_cell(Style::default())
    } else {
        theme::header_style()
    }
}

fn cell_content(
    activity: Option<&crate::domain::Activity>,
    is_selected: bool,
    is_now: bool,
) -> (String, Style) {
    let label = match activity {
        Some(activity) => {
            let note = if activity.has_note() { "*" } else { " " };
            format!(" {}{}", activity.category().digit(), note)
        }
        None => " ·".to_string(),
    };

    let base = match activity {
        Some(activity) => Style::default().fg(theme::color(activity.category())),
        None => theme::empty_style(),
    };

    let style = if is_selected {
        theme::selected(base)
    } else if is_now {
        theme::now_cell(base)
    } else {
        base
    };

    (pad_cell(label), style)
}

fn pad_cell(text: String) -> String {
    format!("{text:<HOUR_CONTENT_WIDTH$}")
}
