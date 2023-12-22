use ratatui::{layout::Rect, Frame};

use crate::{
    component::{game::GameComponent, *},
    error::Result,
    event::Event,
};

pub struct AppComponent {
    game: GameComponent,
}

impl<'a> Component for AppComponent {
    fn init(&mut self) -> Result<()> {
        self.game.init()?;
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<()> {
        self.game.handle_event(event)?;
        Ok(())
    }

    fn render(&self, f: &mut Frame, rect: Rect) {
        self.game.render(f, rect);
    }
}

impl<'a> AppComponent {
    pub fn new() -> AppComponent {
        AppComponent {
            game: GameComponent::new(),
        }
    }
}
