use chrono::{Datelike, Local, NaiveDate, Timelike};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    controller::{NoteTarget, State},
    domain::Category,
};

use super::theme;

const DATE_WIDTH: usize = 16;
const HOUR_CONTENT_WIDTH: usize = 5;
const MIN_VISIBLE_DATE_ROWS: usize = 4;
const MIN_VISIBLE_HOURS: usize = 4;

pub(crate) fn focused_cell_rect(area: Rect, target: &NoteTarget) -> Option<Rect> {
    let grid_area = grid_area(area);
    let visible_dates = visible_dates_for_target(target, grid_area.height as usize);
    let date = match target {
        NoteTarget::Day { date } => *date,
        NoteTarget::Hour { date, .. } => *date,
    };
    let row_y = focused_date_row_y(&visible_dates, date)?;

    let (x, width) = match target {
        NoteTarget::Day { .. } => (0u16, DATE_WIDTH as u16),
        NoteTarget::Hour { hour, .. } => {
            let visible_hours = visible_hours(*hour, visible_hour_cols(grid_area.width as usize));
            let hour_index = visible_hours.iter().position(|visible| visible == hour)?;
            let x = (DATE_WIDTH + 1 + hour_index * (HOUR_CONTENT_WIDTH + 1)) as u16;
            (x, HOUR_CONTENT_WIDTH as u16)
        }
    };

    Some(Rect {
        x: grid_area.x + x,
        y: grid_area.y + row_y,
        width,
        height: 1,
    })
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &State,
    now: &chrono::DateTime<Local>,
) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(24)])
        .split(area);

    frame.render_widget(
        Paragraph::new(build_grid_lines(
            state,
            now,
            sections[0].width as usize,
            sections[0].height as usize,
        )),
        sections[0],
    );
    if sections.len() > 1 && sections[1].width >= 12 {
        render_legend(frame, sections[1]);
    }
}

fn build_grid_lines(
    state: &State,
    now: &chrono::DateTime<Local>,
    max_width: usize,
    max_lines: usize,
) -> Vec<Line<'static>> {
    let visible_dates = visible_dates(state.cursor.date, visible_date_rows(max_lines));
    let visible_hours = visible_hours(state.cursor.hour.unwrap_or(0), visible_hour_cols(max_width));
    let mut lines = Vec::new();
    lines.push(build_header_line(&visible_hours));
    lines.push(Line::raw(build_rule_line('─', '┼', visible_hours.len())));

    let mut active_month = None;
    for (idx, date) in visible_dates.iter().enumerate() {
        if active_month != Some((date.year(), date.month())) {
            active_month = Some((date.year(), date.month()));
            lines.push(Line::from(Span::styled(
                format!("**{} {}**", date.format("%B"), date.year()),
                theme::month_header_style(),
            )));
            lines.push(Line::raw(build_rule_line('═', '╪', visible_hours.len())));
        }
        lines.push(build_day_line(state, *date, now, &visible_hours));

        let separator = match visible_dates.get(idx + 1) {
            Some(next) if (next.year(), next.month()) != (date.year(), date.month()) => {
                build_rule_line('═', '╪', visible_hours.len())
            }
            _ => build_rule_line('─', '┼', visible_hours.len()),
        };
        lines.push(Line::raw(separator));
    }

    lines
}

fn build_header_line(hours: &[u8]) -> Line<'static> {
    let mut spans = vec![Span::raw(format!("{:<DATE_WIDTH$}│", ""))];
    for hour in hours {
        spans.push(Span::styled(
            format!("{hour:>2}.00"),
            theme::header_style(),
        ));
        spans.push(Span::raw("│"));
    }
    Line::from(spans)
}

fn build_day_line(
    state: &State,
    date: NaiveDate,
    now: &chrono::DateTime<Local>,
    hours: &[u8],
) -> Line<'static> {
    let day_has_note = state.day(date).and_then(|d| d.note()).is_some();
    let date_format = date.format("%d.%m.%Y").to_string();
    let weekday = date.format("%a").to_string();
    let combined = if day_has_note {
        format!("{} {}*", date_format, weekday)
    } else {
        format!("{} {}", date_format, weekday)
    };
    let date_text = Span::styled(
        format!("{:^16}", combined),
        date_style(
            date,
            state.cursor.date,
            now.date_naive(),
            state.cursor.hour.is_none(),
        ),
    );
    let mut spans = vec![date_text, Span::raw("│")];

    for hour in hours {
        let activity = state.activity(date, *hour);
        let is_selected = state.cursor.date == date && state.cursor.hour == Some(*hour);
        let is_now = date == now.date_naive() && *hour == now.hour() as u8;
        let (text, style) = cell_content(activity, is_selected, is_now);
        spans.push(Span::styled(text, style));
        spans.push(Span::raw("│"));
    }

    Line::from(spans)
}

fn build_rule_line(fill: char, cross: char, hour_count: usize) -> String {
    let mut line = String::new();
    line.push_str(&fill.to_string().repeat(DATE_WIDTH));
    line.push(cross);
    for _ in 0..hour_count {
        line.push_str(&fill.to_string().repeat(HOUR_CONTENT_WIDTH));
        line.push(cross);
    }
    line
}

fn grid_area(area: Rect) -> Rect {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(24)])
        .split(area);
    sections[0]
}

fn visible_dates_for_target(target: &NoteTarget, max_lines: usize) -> Vec<NaiveDate> {
    let center = match target {
        NoteTarget::Day { date } => *date,
        NoteTarget::Hour { date, .. } => *date,
    };
    visible_dates(center, visible_date_rows(max_lines))
}

fn focused_date_row_y(visible_dates: &[NaiveDate], date: NaiveDate) -> Option<u16> {
    let mut row_y = 2u16;
    let mut active_month = None;

    for current in visible_dates {
        if active_month != Some((current.year(), current.month())) {
            active_month = Some((current.year(), current.month()));
            row_y = row_y.saturating_add(2);
        }

        if *current == date {
            return Some(row_y);
        }

        row_y = row_y.saturating_add(2);
    }

    None
}

fn render_legend(frame: &mut Frame, area: Rect) {
    let legend_width = area.width.min(24);
    let legend_height = area.height.min(12);
    let legend_area = Rect {
        x: area.x,
        y: area.y,
        width: legend_width,
        height: legend_height,
    };
    let block = Block::default().title(" Palette ").borders(Borders::ALL);
    let inner = block.inner(legend_area);
    frame.render_widget(block, legend_area);
    frame.render_widget(Paragraph::new(build_legend_lines()), inner);
}

fn build_legend_lines() -> Vec<Line<'static>> {
    Category::ALL
        .iter()
        .map(|category| {
            Line::from(Span::styled(
                format!("{} = {}", category.digit(), category.label()),
                Style::default().fg(theme::color(*category)),
            ))
        })
        .collect()
}

fn visible_dates(center: NaiveDate, count: usize) -> Vec<NaiveDate> {
    let start = center - chrono::Duration::days((count / 2) as i64);
    (0..count)
        .map(|offset| start + chrono::Duration::days(offset as i64))
        .collect()
}

fn visible_date_rows(max_lines: usize) -> usize {
    let reserved_lines = 4usize;
    let rows = max_lines.saturating_sub(reserved_lines) / 2;
    rows.max(MIN_VISIBLE_DATE_ROWS)
}

fn visible_hour_cols(max_width: usize) -> usize {
    let reserved = DATE_WIDTH + 1;
    let hour_col_width = HOUR_CONTENT_WIDTH + 1;
    let cols = max_width.saturating_sub(reserved) / hour_col_width;
    cols.clamp(MIN_VISIBLE_HOURS, 24)
}

fn visible_hours(center: u8, count: usize) -> Vec<u8> {
    let count = count.min(24);
    let radius = count / 2;
    let max_start = 24usize.saturating_sub(count);
    let start = usize::from(center).saturating_sub(radius).min(max_start);
    (start..start + count).map(|hour| hour as u8).collect()
}

fn date_style(
    date: NaiveDate,
    selected: NaiveDate,
    today: NaiveDate,
    hour_focus: bool,
) -> Style {
    if date == selected && hour_focus {
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
            match activity.category() {
                Some(category) => {
                    let note = if activity.has_note() { "*" } else { " " };
                    format!(" {}{}", category.digit(), note)
                }
                None => " *".to_string(),
            }
        }
        None => String::new(),
    };

    let base = match activity {
        Some(activity) => match activity.category() {
            Some(category) => theme::cell_style(category),
            None => theme::empty_style(),
        },
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
