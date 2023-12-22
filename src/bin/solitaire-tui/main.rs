mod component;
mod error;
mod event;

use std::io;

use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};

use crate::{
    component::{app::AppComponent, Component},
    error::Result,
    event::*,
};

fn main() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let mut app = AppComponent::new();
    let events = Events::new(250);

    loop {
        terminal.draw(|f| app.render(f, f.size()))?;

        match events.next()? {
            Message::Event(Event::KeyPress(KeyCode::Char('q'), _))
            | Message::Event(Event::KeyPress(KeyCode::Char('c'), KeyModifiers::CONTROL)) => break,
            Message::Event(event) => app.handle_event(&event)?,
            Message::Tick => {}
        }
    }

    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
