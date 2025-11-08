use ratatui::{layout::Rect, Frame};

use crate::{
    ui::component::{game::GameComponent, *},
    ui::error::Result,
    ui::event::Event,
};

pub struct AppComponent<RNG: rand::Rng> {
    game: GameComponent<RNG>,
}

impl<RNG: rand::Rng> Component for AppComponent<RNG> {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        self.game.handle_event(event)
    }

    fn handle_tick(&mut self, dt: &web_time::Duration) -> Result<()> {
        self.game.handle_tick(dt)
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
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
