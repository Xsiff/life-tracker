use std::time::Duration;

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers},
};

use crate::domain::Action;

pub fn next_action(timeout: Duration) -> anyhow::Result<Option<Action>> {
    if !poll(timeout)? {
        return Ok(Some(Action::Tick));
    }

    let Event::Key(key) = read()? else {
        return Ok(None);
    };
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    let action = match key.code {
        KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
            Some(Action::MoveWordLeft)
        }
        KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
            Some(Action::MoveWordRight)
        }
        KeyCode::Delete if key.modifiers.contains(KeyModifiers::ALT) => {
            Some(Action::DeleteWord)
        }
        KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
            Some(Action::DeleteWord)
        }
        KeyCode::Left => Some(Action::MoveLeft),
        KeyCode::Right => Some(Action::MoveRight),
        KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                Some(Action::InsertNewline)
            } else {
                Some(Action::Confirm)
            }
        }
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Tab => Some(Action::CycleView),
        KeyCode::Backspace => Some(Action::Erase),
        KeyCode::Char(c) if c.is_ascii_digit() => Some(Action::Digit(c as u8 - b'0')),
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::InsertNewline)
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                None
            } else {
                Some(Action::Char(c))
            }
        }
        _ => None,
    };

    Ok(action)
}
