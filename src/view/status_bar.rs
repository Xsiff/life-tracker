use chrono::{DateTime, Datelike, Local, Timelike};
use ratatui::{
    layout::Rect,
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::controller::{Overlay, State, ViewMode};

pub fn render(frame: &mut Frame, area: Rect, state: &State, now: &DateTime<Local>) {
    let focus_or_error = state
        .last_error
        .as_deref()
        .map(|error| format!("Error: {error}"))
        .unwrap_or_else(|| focus_text(state));

    let commands = match state.overlay.as_ref() {
        Some(Overlay::CategoryPicker { .. }) => "↑↓ move  0-9 select  ⏎ confirm  Esc cancel",
        Some(Overlay::NoteEditor { .. }) => "type text  ⏎ save  Esc cancel  Backspace erase",
        None => match state.view {
            ViewMode::Calendar => "←↑↓→ move  ⏎ open  N note  v view  q quit",
            ViewMode::Day => "↑↓ move  ⏎ set  x clear  n note  v view  Esc back",
        },
    };

    let lines = vec![
        Line::raw(format!(
            "now {} {} {} · {:02}:{:02}   {}",
            weekday(now.weekday().num_days_from_monday() as usize),
            now.day(),
            month(now.month0() as usize),
            now.hour(),
            now.minute(),
            focus_or_error
        )),
        Line::raw(commands),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn focus_text(state: &State) -> String {
    match state.view {
        ViewMode::Calendar => {
            let filled = state.day(state.cursor.date).map(|day| day.filled_hours()).unwrap_or(0);
            format!(
                "Focus: {} {} {} ({}h)",
                weekday(state.cursor.date.weekday().num_days_from_monday() as usize),
                state.cursor.date.day(),
                month(state.cursor.date.month0() as usize),
                filled
            )
        }
        ViewMode::Day => {
            let hour = state.cursor.hour.unwrap_or(0);
            match state.activity(state.cursor.date, hour) {
                Some(activity) => {
                    let note = if activity.has_note() { " *" } else { "" };
                    format!("Focus: {hour:02}:00 {}{note}", activity.category().label())
                }
                None => format!("Focus: {hour:02}:00 Empty"),
            }
        }
    }
}

fn weekday(index: usize) -> &'static str {
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][index]
}

fn month(index: usize) -> &'static str {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][index]
}
