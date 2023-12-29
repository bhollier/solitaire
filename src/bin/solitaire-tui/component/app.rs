use ratatui::{layout::Rect, Frame};

use crate::{
    component::{game::GameComponent, *},
    error::Result,
    event::Event,
};

pub struct AppComponent<RNG: rand::Rng> {
    game: GameComponent<RNG>,
}

impl<RNG: rand::Rng> Component for AppComponent<RNG> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        self.game.handle_event(event)?;
        Ok(())
    }

    fn handle_tick(&mut self, dt: &std::time::Duration) -> Result<()> {
        self.game.handle_tick(dt)?;
        Ok(())
    }

    fn render(&self, f: &mut Frame, rect: Rect) {
        self.game.render(f, rect);
    }
}

impl<RNG: rand::Rng + Clone> AppComponent<RNG> {
    pub fn new(rng: &RNG) -> AppComponent<RNG> {
        AppComponent {
            game: GameComponent::new(rng.clone()),
        }
    }
}
