mod app;
mod controller;
mod domain;
mod event;
mod storage;
mod view;

use std::{io, time::Duration};

use anyhow::Context;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal().context("failed to initialize terminal preview")?;
    let result = run_preview(&mut terminal);
    restore_terminal(&mut terminal).context("failed to restore terminal")?;
    result
}

fn run_preview(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let scenes = view::preview_scenes();
    let mut selected = 0usize;

    loop {
        let scene = &scenes[selected];
        terminal.draw(|frame| view::render(frame, &scene.state))?;

        if !poll(Duration::from_millis(250))? {
            continue;
        }

        let Event::Key(key) = read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Left | KeyCode::Up => {
                selected = selected.checked_sub(1).unwrap_or(scenes.len() - 1);
            }
            KeyCode::Right | KeyCode::Down | KeyCode::Tab | KeyCode::Enter | KeyCode::Char(' ') => {
                selected = (selected + 1) % scenes.len();
            }
            _ => {}
        }
    }

    Ok(())
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
