use chrono::{Datelike, NaiveDate};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::controller::NoteTarget;

use super::calendar_view::{
    DATE_WIDTH, HOUR_CONTENT_WIDTH, MIN_VISIBLE_DATE_ROWS, MIN_VISIBLE_HOURS,
};

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

    Some(Rect { x: grid_area.x + x, y: grid_area.y + row_y, width, height: 1 })
}

pub(crate) fn grid_area(area: Rect) -> Rect {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(24)])
        .split(area);
    sections[0]
}

pub(crate) fn visible_dates(center: NaiveDate, count: usize) -> Vec<NaiveDate> {
    let start = center - chrono::Duration::days((count / 2) as i64);
    (0..count).map(|offset| start + chrono::Duration::days(offset as i64)).collect()
}

pub(crate) fn visible_date_rows(max_lines: usize) -> usize {
    let reserved_lines = 4usize;
    let rows = max_lines.saturating_sub(reserved_lines) / 2;
    rows.max(MIN_VISIBLE_DATE_ROWS)
}

pub(crate) fn visible_hour_cols(max_width: usize) -> usize {
    let reserved = DATE_WIDTH + 1;
    let hour_col_width = HOUR_CONTENT_WIDTH + 1;
    let cols = max_width.saturating_sub(reserved) / hour_col_width;
    cols.clamp(MIN_VISIBLE_HOURS, 24)
}

pub(crate) fn visible_hours(center: u8, count: usize) -> Vec<u8> {
    let count = count.min(24);
    let radius = count / 2;
    let max_start = 24usize.saturating_sub(count);
    let start = usize::from(center).saturating_sub(radius).min(max_start);
    (start..start + count).map(|hour| hour as u8).collect()
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
