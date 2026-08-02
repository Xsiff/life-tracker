pub mod calendar_view;
pub mod category_picker;
pub mod note_editor;
pub mod status_bar;
pub mod theme;

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Local};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::controller::{Overlay, State, ViewMode};
use crate::{
    controller::{Cursor, NoteTarget},
    domain::{Activity, Category, Day},
};

#[derive(Debug, Clone)]
pub struct PreviewScene {
    pub name: &'static str,
    pub state: State,
}

pub fn render(frame: &mut Frame, state: &State) {
    let now = Local::now();
    render_with_now(frame, state, &now);
}

pub fn preview_scenes() -> Vec<PreviewScene> {
    vec![
        PreviewScene {
            name: "Calendar",
            state: calendar_preview_state(),
        },
        PreviewScene {
            name: "Category Picker",
            state: category_picker_preview_state(),
        },
        PreviewScene {
            name: "Note Editor",
            state: note_editor_preview_state(),
        },
    ]
}

fn render_with_now(frame: &mut Frame, state: &State, now: &DateTime<Local>) {
    let title = format!(
        " life-tracker ───────────────────────── {} {} ",
        month(now.month0() as usize),
        now.year()
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);
    let area = frame.size();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    calendar_view::render(frame, sections[0], state, now);
    status_bar::render(frame, sections[1], state, now);

    if let Some(overlay) = &state.overlay {
        render_overlay(frame, state, overlay, overlay_rect(area, overlay));
    }
}

fn render_overlay(frame: &mut Frame, state: &State, overlay: &Overlay, area: Rect) {
    match overlay {
        Overlay::CategoryPicker { hour, selected, .. } => {
            category_picker::render(frame, area, *hour, *selected);
        }
        Overlay::NoteEditor { target, draft, .. } => {
            note_editor::render(frame, area, state, target, draft);
        }
    }
}

fn overlay_rect(area: Rect, overlay: &Overlay) -> Rect {
    let (width, height) = match overlay {
        Overlay::CategoryPicker { .. } => (30, 15),
        Overlay::NoteEditor { .. } => (30, 8),
    };
    centered_rect(area, width.min(area.width), height.min(area.height))
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

fn month(index: usize) -> &'static str {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][index]
}

fn calendar_preview_state() -> State {
    let mut days = BTreeMap::new();

    let mut monday = Day::new(date(2026, 7, 27));
    for hour in 0..16 {
        monday.set_hour(hour, Activity::new(Category::Sleep));
    }
    days.insert(monday.date(), monday);

    let mut tuesday = Day::new(date(2026, 7, 28));
    for hour in 0..24 {
        tuesday.set_hour(hour, Activity::new(Category::Work));
    }
    days.insert(tuesday.date(), tuesday);

    let mut wednesday = Day::new(date(2026, 7, 29));
    for hour in 0..3 {
        wednesday.set_hour(hour, Activity::new(Category::Health));
    }
    days.insert(wednesday.date(), wednesday);

    let mut thursday = Day::new(date(2026, 7, 30));
    for hour in 0..16 {
        thursday.set_hour(hour, Activity::new(Category::Sleep));
    }
    days.insert(thursday.date(), thursday);

    let mut friday = Day::new(date(2026, 7, 31));
    for hour in 0..8 {
        friday.set_hour(hour, Activity::new(Category::Travel));
    }
    days.insert(friday.date(), friday);

    let mut sunday = Day::new(date(2026, 8, 2));
    for hour in 0..7 {
        sunday.set_hour(hour, Activity::new(Category::Work));
    }
    days.insert(sunday.date(), sunday);

    let mut monday_next = Day::new(date(2026, 8, 3));
    for hour in 0..8 {
        monday_next.set_hour(hour, Activity::new(Category::Sleep));
    }
    days.insert(monday_next.date(), monday_next);

    let mut tuesday_next = Day::new(date(2026, 8, 4));
    for hour in 0..16 {
        tuesday_next.set_hour(hour, Activity::new(Category::Work));
    }
    days.insert(tuesday_next.date(), tuesday_next);

    let mut wednesday_next = Day::new(date(2026, 8, 5));
    for hour in 0..8 {
        wednesday_next.set_hour(hour, Activity::new(Category::Health));
    }
    days.insert(wednesday_next.date(), wednesday_next);

    State {
        view: ViewMode::Calendar,
        cursor: Cursor {
            date: date(2026, 8, 2),
            hour: None,
        },
        overlay: None,
        days,
        last_error: Some("Preview: Calendar  ←/→ switch scene  q quit".to_string()),
        quit: false,
    }
}

fn matrix_preview_state(overlay: Option<Overlay>) -> State {
    let mut days = BTreeMap::new();
    let mut day = Day::new(date(2026, 8, 2));
    for hour in 0..=6 {
        day.set_hour(hour, Activity::new(Category::Sleep));
    }
    day.set_hour(7, Activity::new(Category::Health));
    day.set_hour(8, Activity::new(Category::Travel));
    for hour in 9..=11 {
        day.set_hour(hour, Activity::new(Category::Work));
    }
    day.set_hour(12, Activity::new(Category::Health));
    day.set_hour(13, Activity::with_note(Category::Work, "Sprint planning, blocked"));
    day.set_hour(14, Activity::new(Category::Work));
    day.set_hour(16, Activity::new(Category::Relaxation));
    day.set_hour(17, Activity::new(Category::HobbiesSkills));
    days.insert(day.date(), day);

    State {
        view: crate::controller::ViewMode::Calendar,
        cursor: Cursor {
            date: date(2026, 8, 2),
            hour: Some(13),
        },
        overlay,
        days,
        last_error: Some("Preview: Matrix  ←/→ switch scene  q quit".to_string()),
        quit: false,
    }
}

fn category_picker_preview_state() -> State {
    let mut state = matrix_preview_state(Some(Overlay::CategoryPicker {
        date: date(2026, 8, 2),
        hour: 13,
        selected: Category::Sleep,
    }));
    state.last_error = Some("Preview: Category Picker  ←/→ switch scene  q quit".to_string());
    state
}

fn note_editor_preview_state() -> State {
    let mut state = matrix_preview_state(Some(Overlay::NoteEditor {
        target: NoteTarget::Hour {
            date: date(2026, 8, 2),
            hour: 13,
        },
        draft: "Sprint planning, blocked\non API keys.".to_string(),
        cursor: 40,
    }));
    state.last_error = Some("Preview: Note Editor  ←/→ switch scene  q quit".to_string());
    state
}

fn date(year: i32, month: u32, day: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::{Local, NaiveDate, TimeZone};
    use ratatui::{backend::TestBackend, Terminal};

    use crate::{controller::{Cursor, NoteTarget, Overlay, State, ViewMode}, domain::{Activity, Category, Day}};

    use super::{calendar_preview_state, matrix_preview_state, render_with_now};

    #[test]
    fn renders_calendar_view_scaffold() {
        let mut days = BTreeMap::new();
        let mut monday = Day::new(date(2026, 7, 27));
        for hour in 0..16 {
            monday.set_hour(hour, Activity::new(Category::Sleep));
        }
        days.insert(monday.date(), monday);

        let mut sunday = Day::new(date(2026, 8, 2));
        for hour in 0..7 {
            sunday.set_hour(hour, Activity::new(Category::Work));
        }
        days.insert(sunday.date(), sunday);

        let state = State {
            view: crate::controller::ViewMode::Calendar,
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(0),
            },
            overlay: None,
            days,
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 160, 32);
        assert!(output.contains("life-tracker"));
        assert!(output.contains("00.00"));
        assert!(output.contains("23.00"));
        assert!(output.contains("**August 2026**"));
        assert!(output.contains("Focus: 02.08.2026 00.00 Work"));
    }

    #[test]
    fn renders_matrix_focus_line() {
        let mut state = matrix_preview_state(None);
        state.last_error = None;

        let output = render_to_string(&state, 160, 32);
        assert!(output.contains("13.00"));
        assert!(output.contains("Focus: 02.08.2026 13.00 Work *"));
    }

    #[test]
    fn renders_category_picker_overlay() {
        let state = State {
            view: crate::controller::ViewMode::Calendar,
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(13),
            },
            overlay: Some(Overlay::CategoryPicker {
                date: date(2026, 8, 2),
                hour: 13,
                selected: Category::Sleep,
            }),
            days: BTreeMap::new(),
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 80, 24);
        assert!(output.contains("Set activity - 13.00"));
        assert!(output.contains("> 0 Sleep"));
        assert!(output.contains("[+] add note"));
        assert!(output.contains("9 Other"));
    }

    #[test]
    fn renders_note_editor_overlay() {
        let mut days = BTreeMap::new();
        let mut day = Day::new(date(2026, 8, 2));
        day.set_hour(13, Activity::new(Category::Work));
        days.insert(day.date(), day);

        let state = State {
            view: crate::controller::ViewMode::Calendar,
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(13),
            },
            overlay: Some(Overlay::NoteEditor {
                target: NoteTarget::Hour {
                    date: date(2026, 8, 2),
                    hour: 13,
                },
                draft: "Sprint planning, blocked\non API keys.".to_string(),
                cursor: 40,
            }),
            days,
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 58, 18);
        assert!(output.contains("Note - 13:00 Work"));
        assert!(output.contains("Sprint planning, blocked"));
        assert!(output.contains("on API keys."));
    }

    #[test]
    #[ignore]
    fn print_calendar_example() {
        let mut state = calendar_preview_state();
        state.last_error = None;

        println!("{}", render_to_string(&state, 58, 12));
    }

    #[test]
    #[ignore]
    fn print_day_example() {
        let mut state = matrix_preview_state(None);
        state.last_error = None;

        println!("{}", render_to_string(&state, 160, 32));
    }

    fn render_to_string(state: &State, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let now = Local
            .with_ymd_and_hms(2026, 8, 2, 13, 47, 0)
            .single()
            .expect("fixed datetime");

        terminal
            .draw(|frame| render_with_now(frame, state, &now))
            .expect("draw");

        let buffer = terminal.backend().buffer().clone();
        let mut lines = Vec::new();
        for y in 0..height {
            let mut line = String::new();
            for x in 0..width {
                line.push_str(buffer.get(x, y).symbol());
            }
            lines.push(line);
        }
        lines.join("\n")
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }
}
