mod controller;
mod domain;
mod input;
mod storage;
mod view;

use std::io;

use anyhow::Context;
use controller::Controller;
use crossterm::{
    execute,
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> anyhow::Result<()> {
    let (mut terminal, keyboard_enhancement_enabled) =
        setup_terminal().context("failed to initialize terminal")?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal, keyboard_enhancement_enabled)
        .context("failed to restore terminal")?;
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

        if let Some(action) = input::next_action(std::time::Duration::from_millis(250))? {
            controller.update(action)?;
        }
    }

    Ok(())
}

fn setup_terminal() -> anyhow::Result<(Terminal<CrosstermBackend<io::Stdout>>, bool)> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let keyboard_enhancement_enabled = enable_keyboard_enhancement(&mut stdout);
    let backend = CrosstermBackend::new(stdout);
    Ok((Terminal::new(backend)?, keyboard_enhancement_enabled))
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    keyboard_enhancement_enabled: bool,
) -> anyhow::Result<()> {
    if keyboard_enhancement_enabled {
        let _ = execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags);
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn enable_keyboard_enhancement(stdout: &mut io::Stdout) -> bool {
    match crossterm::terminal::supports_keyboard_enhancement() {
        Ok(true) => execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )
        .is_ok(),
        _ => false,
    }
}
