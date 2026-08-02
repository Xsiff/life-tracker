use chrono::{DateTime, Local, Timelike};
use ratatui::{
    layout::Rect,
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::controller::{NoteTarget, Overlay, State};

pub fn render(frame: &mut Frame, area: Rect, state: &State, now: &DateTime<Local>) {
    let focus_or_error = state
        .last_error
        .as_deref()
        .map(|error| format!("Error: {error}"))
        .unwrap_or_else(|| focus_text(state));

    let commands = match state.overlay.as_ref() {
        Some(Overlay::CategoryPicker { target, .. }) => match target {
            NoteTarget::Day { .. } => "↑↓ move  ⏎ confirm  Esc cancel",
            NoteTarget::Hour { .. } => "↑↓ move  0-9 select  ⏎ confirm  Esc cancel",
        },
        Some(Overlay::NoteEditor { .. }) => {
            "type text  ⇧⏎/Ctrl+J newline  ⏎ save  Esc cancel  Backspace erase"
        }
        None => "←↑↓→ move  ⏎ open  n note  x clear  q quit",
    };

    let lines = vec![
        Line::raw(format!(
            "now {} · {:02}:{:02}   {}",
            now.format("%d.%m.%Y"),
            now.hour(),
            now.minute(),
            focus_or_error
        )),
        Line::raw(commands),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn focus_text(state: &State) -> String {
    match state.cursor.hour {
        Some(hour) => match state.activity(state.cursor.date, hour) {
            Some(activity) => {
                let note = if activity.has_note() { " *" } else { "" };
                let label = activity
                    .category()
                    .map(|category| category.label())
                    .unwrap_or("No activity");
                format!(
                    "Focus: {} {hour:02}.00 {}{note}",
                    state.cursor.date.format("%d.%m.%Y"),
                    label
                )
            }
            None => format!(
                "Focus: {} {hour:02}.00 Empty",
                state.cursor.date.format("%d.%m.%Y")
            ),
        },
        None => {
            let note = if state.day(state.cursor.date).and_then(|day| day.note()).is_some() {
                " *"
            } else {
                ""
            };
            format!("Focus: {} Day{note}", state.cursor.date.format("%d.%m.%Y"))
        }
    }
}
