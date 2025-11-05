mod component;
mod error;
mod event;

use std::io;

use clap::Parser;
use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::prelude::{SmallRng, *};
use rand_seeder::Seeder;
use ratatui::prelude::{CrosstermBackend, Terminal};

use crate::{
    component::{app::AppComponent, Component},
    error::Result,
    event::*,
};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    seed: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rng = match args.seed.as_deref() {
        Some(seed) => Seeder::from(seed).make_rng(),
        None => SmallRng::from_rng(thread_rng()).unwrap(),
    };

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mouse_events = io::stdout()
        .execute(crossterm::event::EnableMouseCapture)
        .is_ok();

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let mut app = AppComponent::new(&rng);
    let events = Events::new(std::time::Duration::from_millis(100));

    loop {
        terminal.draw(|f| app.render(f, f.size()))?;

        match events.next()? {
            Message::Event(Event::KeyPress(KeyCode::Char('q'), _))
            | Message::Event(Event::KeyPress(KeyCode::Char('c'), KeyModifiers::CONTROL)) => break,
            Message::Event(event) => {
                app.handle_event(&event)?;
            }
            Message::Tick(dt) => {
                app.handle_tick(&dt)?;
            }
        }
    }

    if mouse_events {
        io::stdout().execute(crossterm::event::DisableMouseCapture)?;
    }
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
