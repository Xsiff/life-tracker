mod controller;
mod domain;
mod event;
mod storage;
mod view;

use std::{io, time::Duration};

use anyhow::Context;
use controller::Controller;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::domain::Action;

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal().context("failed to initialize terminal")?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal).context("failed to restore terminal")?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let store = storage::Store::open()?;
    let mut controller = Controller::new(store)?;

    loop {
        terminal.draw(|frame| view::render(frame, controller.state()))?;
        if controller.should_quit() {
            break;
        }

        if let Some(action) = next_action(Duration::from_millis(250))? {
            controller.update(action)?;
        }
    }

    Ok(())
}

fn next_action(timeout: Duration) -> anyhow::Result<Option<Action>> {
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
        KeyCode::Left => Some(Action::MoveLeft),
        KeyCode::Right => Some(Action::MoveRight),
        KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Tab => Some(Action::CycleView),
        KeyCode::Backspace => Some(Action::Erase),
        KeyCode::Char(c) if c.is_ascii_digit() => Some(Action::Digit(c as u8 - b'0')),
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

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
