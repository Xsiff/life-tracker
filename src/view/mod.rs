pub mod calendar_view;
pub mod category_picker;
pub mod help_popup;
pub mod note_editor;
pub mod status_bar;
pub mod theme;

use chrono::{DateTime, Datelike, Local};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::controller::{NoteTarget, Overlay, State};

pub fn render(frame: &mut Frame, state: &State) {
    let now = Local::now();
    render_with_now(frame, state, &now);
}

fn render_with_now(frame: &mut Frame, state: &State, now: &DateTime<Local>) {
    let title = format!(
        " life-tracker ───────────────────────── {} {} ",
        month(now.month0() as usize),
        now.year()
    );

    let block = Block::default().title(title).borders(Borders::ALL);
    let area = frame.size();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let matrix_area = sections[0];
    calendar_view::render(frame, matrix_area, state, now);
    status_bar::render(frame, sections[1], state, now);

    if let Some(overlay) = &state.overlay {
        render_overlay(frame, state, overlay, overlay_rect(matrix_area, overlay));
    }
}

fn render_overlay(frame: &mut Frame, state: &State, overlay: &Overlay, area: Rect) {
    match overlay {
        Overlay::CategoryPicker { target, selected } => {
            category_picker::render(frame, area, target, *selected);
        }
        Overlay::Help => {
            help_popup::render(frame, area);
        }
        Overlay::NoteEditor {
            target,
            draft,
            cursor,
        } => {
            note_editor::render(frame, area, state, target, draft, *cursor);
        }
    }
}

fn overlay_rect(area: Rect, overlay: &Overlay) -> Rect {
    let (width, height) = match overlay {
        Overlay::CategoryPicker { target, .. } => match target {
            NoteTarget::Day { .. } => (30, 6),
            NoteTarget::Hour { .. } => (30, 17),
        },
        Overlay::Help => (74, 16),
        Overlay::NoteEditor { .. } => (42, 12),
    };

    let width = width.min(area.width);
    let height = height.min(area.height);
    let anchor = match overlay {
        Overlay::CategoryPicker { target, .. } => {
            calendar_view::focused_cell_rect(area, target)
        }
        Overlay::NoteEditor { target, .. } => {
            calendar_view::focused_cell_rect(area, target)
        }
        Overlay::Help => None,
    };

    match anchor {
        Some(anchor) => anchored_rect(area, anchor, width, height),
        None => centered_rect(area, width, height),
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

fn anchored_rect(area: Rect, anchor: Rect, width: u16, height: u16) -> Rect {
    let right_space = area.x.saturating_add(area.width);
    let anchor_right = anchor.x.saturating_add(anchor.width);
    let x = if anchor_right.saturating_add(1).saturating_add(width) <= right_space {
        anchor_right.saturating_add(1)
    } else if anchor.x >= area.x.saturating_add(width).saturating_add(1) {
        anchor.x.saturating_sub(width.saturating_add(1))
    } else {
        area.x + (area.width.saturating_sub(width)) / 2
    };

    let bottom_space = area.y.saturating_add(area.height);
    let y = if anchor.y.saturating_add(height) <= bottom_space {
        anchor.y
    } else {
        bottom_space.saturating_sub(height)
    };

    Rect { x, y, width, height }
}

fn month(index: usize) -> &'static str {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][index]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::{Local, NaiveDate, TimeZone};
    use ratatui::{backend::TestBackend, Terminal};

    use crate::controller::{CategoryPickerSelection, Cursor, NoteTarget, Overlay, State};
    use crate::domain::{Activity, Category, Day};

    use super::render_with_now;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn matrix_state(overlay: Option<Overlay>) -> State {
        let mut days = BTreeMap::new();
        let day_date = date(2026, 8, 2);
        let mut day = Day::new(day_date);
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
        days.insert(day_date, day);

        State {
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

    #[test]
    fn renders_calendar_view_scaffold() {
        let mut days = BTreeMap::new();
        let monday_date = date(2026, 7, 27);
        let mut monday = Day::new(monday_date);
        for hour in 0..16 {
            monday.set_hour(hour, Activity::new(Category::Sleep));
        }
        days.insert(monday_date, monday);

        let sunday_date = date(2026, 8, 2);
        let mut sunday = Day::new(sunday_date);
        for hour in 0..7 {
            sunday.set_hour(hour, Activity::new(Category::Work));
        }
        days.insert(sunday_date, sunday);

        let state = State {
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
        assert!(output.contains("13.00"));
        assert!(output.contains("August 2026"));
        assert!(output.contains("Focus: 02.08.2026 Sun 00.00 Work"));
    }

    #[test]
    fn renders_strong_separator_at_month_boundary() {
        let mut days = BTreeMap::new();
        days.insert(date(2026, 7, 31), Day::new(date(2026, 7, 31)));
        days.insert(date(2026, 8, 1), Day::new(date(2026, 8, 1)));

        let state = State {
            cursor: Cursor {
                date: date(2026, 8, 1),
                hour: Some(0),
            },
            overlay: None,
            days,
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 160, 32);
        let lines: Vec<_> = output.lines().collect();
        let july_row = lines
            .iter()
            .position(|line| line.contains("31.07.2026"))
            .expect("july row");
        let separator = lines.get(july_row + 1).expect("separator after july row");
        let august_header = lines
            .iter()
            .position(|line| line.contains("August 2026"))
            .expect("august header");
        let after_header = lines.get(august_header + 1).expect("line after header");

        assert!(separator.contains('═'));
        assert!(!separator.contains('─'));
        assert!(after_header.contains('═'));
    }

    #[test]
    fn renders_matrix_focus_line() {
        let mut state = matrix_state(None);
        state.last_error = None;

        let output = render_to_string(&state, 160, 32);
        assert!(output.contains("13.00"));
        assert!(output.contains("Focus: 02.08.2026 Sun 13.00 Work *"));
    }

    #[test]
    fn renders_category_picker_overlay() {
        let state = State {
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(13),
            },
            overlay: Some(Overlay::CategoryPicker {
                target: NoteTarget::Hour {
                    date: date(2026, 8, 2),
                    hour: 13,
                },
                selected: CategoryPickerSelection::Category(Category::Sleep),
            }),
            days: BTreeMap::new(),
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 80, 24);
        assert!(output.contains("Set activity - 13.00"));
        assert!(output.contains("> 0 Sleep"));
        assert!(output.contains("  [+] add note"));
        assert!(output.contains("  [x] delete note"));
        assert!(output.contains("  [x] delete activity"));
        assert!(output.contains("9 Other"));
    }

    #[test]
    fn renders_category_picker_add_note_selected() {
        let state = State {
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(13),
            },
            overlay: Some(Overlay::CategoryPicker {
                target: NoteTarget::Hour {
                    date: date(2026, 8, 2),
                    hour: 13,
                },
                selected: CategoryPickerSelection::AddNote,
            }),
            days: BTreeMap::new(),
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 80, 24);
        assert!(output.contains("> [+] add note"));
    }

    #[test]
    fn renders_day_category_picker_overlay() {
        let state = State {
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: None,
            },
            overlay: Some(Overlay::CategoryPicker {
                target: NoteTarget::Day {
                    date: date(2026, 8, 2),
                },
                selected: CategoryPickerSelection::AddNote,
            }),
            days: BTreeMap::new(),
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 80, 24);
        assert!(output.contains("Day - 02.08.2026"));
        assert!(output.contains("> [+] add note"));
        assert!(output.contains("  [x] delete note"));
        assert!(output.contains("Focus: 02.08.2026 Sun Day"));
    }

    #[test]
    fn renders_note_only_hour_focus() {
        let mut days = BTreeMap::new();
        let day_date = date(2026, 8, 2);
        let mut day = Day::new(day_date);
        day.set_hour(13, Activity::note_only("kept after deleting activity"));
        days.insert(day_date, day);

        let state = State {
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(13),
            },
            overlay: None,
            days,
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 120, 24);
        assert!(output.contains("Focus: 02.08.2026 Sun 13.00 No activity *"));
    }

    #[test]
    fn renders_note_editor_overlay() {
        let mut days = BTreeMap::new();
        let day_date = date(2026, 8, 2);
        let mut day = Day::new(day_date);
        day.set_hour(13, Activity::new(Category::Work));
        days.insert(day_date, day);

        let state = State {
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
        assert!(output.lines().any(|line| line.contains("on API keys.")));
        assert!(output.contains("|"));
    }

    #[test]
    fn wraps_long_note_editor_text_within_popup() {
        let mut days = BTreeMap::new();
        let day_date = date(2026, 8, 2);
        let mut day = Day::new(day_date);
        day.set_hour(13, Activity::new(Category::Work));
        days.insert(day_date, day);
        let state = State {
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(13),
            },
            overlay: Some(Overlay::NoteEditor {
                target: NoteTarget::Hour {
                    date: date(2026, 8, 2),
                    hour: 13,
                },
                draft: "This is a deliberately long note without spaces to force wrapping".to_string(),
                cursor: 65,
            }),
            days,
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 46, 16);
        let long_note = "This is a deliberately long note without spaces to force wrapping";
        assert!(!output.contains(long_note));
        assert!(output.lines().any(|line| line.contains("This is a deliberately")));
        assert!(output.lines().any(|line| line.contains("wrapping")));
    }

    #[test]
    fn renders_help_popup_overlay() {
        let state = State {
            cursor: Cursor {
                date: date(2026, 8, 2),
                hour: Some(13),
            },
            overlay: Some(Overlay::Help),
            days: BTreeMap::new(),
            last_error: None,
            quit: false,
        };

        let output = render_to_string(&state, 96, 28);
        assert!(output.contains("Category help"));
        assert!(output.contains("Sleep"));
        assert!(output.contains("Rest, recovery, and sleep"));
        assert!(output.contains("Travel"));
        assert!(output.contains("Commuting, transit, or trips"));
        assert!(output.contains("Esc cancel"));
        assert!(output.contains("Enter close"));
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
}
