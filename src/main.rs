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
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

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

        if let Some(action) = input::next_action(std::time::Duration::from_millis(250))? {
            controller.update(action)?;
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
