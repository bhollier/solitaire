mod event;

use std::io;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::prelude::{SmallRng, *};
use rand_seeder::Seeder;
use ratatui::prelude::{CrosstermBackend, Terminal};

use crate::event::*;
use solitaire::ui;
use solitaire::ui::component::Component;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    seed: Option<String>,
}

fn main() -> ui::error::Result<()> {
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

    let mut app = ui::component::app::AppComponent::new(&rng);
    let events = Events::new(web_time::Duration::from_millis(100));

    loop {
        terminal.draw(|f| app.render(f, f.area()))?;

        match events.next()? {
            Message::Event(ui::event::Event::KeyPress(ui::event::KeyCode::Char('q'), _))
            | Message::Event(ui::event::Event::KeyPress(
                ui::event::KeyCode::Char('c'),
                ui::event::Modifiers { ctrl: true, .. },
            )) => break,
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
