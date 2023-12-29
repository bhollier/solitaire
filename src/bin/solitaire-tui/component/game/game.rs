use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders,
    },
    Frame,
};
use solitaire::variant::klondike;

use crate::{
    component::{
        game::{
            render, ui_state,
            ui_state::{DealingState, HoveringState, State, UIState},
        },
        Component,
    },
    error::Result,
    event::{Event, EventResult, EventState},
};

pub struct GameComponent<RNG: rand::Rng> {
    rng: RNG,
    state: klondike::GameStateOption,
    ui_state: UIState,
}

impl<RNG: rand::Rng> Component for GameComponent<RNG> {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::KeyPress(KeyCode::Up, m)
            | Event::KeyPress(KeyCode::Char('w'), m)
            | Event::KeyPress(KeyCode::Char('W'), m) => {
                self.handle_direction(ui_state::Direction::Up, *m)
            }
            Event::KeyPress(KeyCode::Down, m)
            | Event::KeyPress(KeyCode::Char('s'), m)
            | Event::KeyPress(KeyCode::Char('S'), m) => {
                self.handle_direction(ui_state::Direction::Down, *m)
            }
            Event::KeyPress(KeyCode::Left, m)
            | Event::KeyPress(KeyCode::Char('a'), m)
            | Event::KeyPress(KeyCode::Char('A'), m) => {
                self.handle_direction(ui_state::Direction::Left, *m)
            }
            Event::KeyPress(KeyCode::Right, m)
            | Event::KeyPress(KeyCode::Char('d'), m)
            | Event::KeyPress(KeyCode::Char('D'), m) => {
                self.handle_direction(ui_state::Direction::Right, *m)
            }
            Event::KeyPress(KeyCode::Enter, _) | Event::KeyPress(KeyCode::Char(' '), _) => {
                self.handle_interact()
            }
            Event::KeyPress(KeyCode::Char(c @ '1'..='9'), _) => {
                self.handle_goto(c.to_digit(10).unwrap())
            }
            Event::KeyPress(KeyCode::Char('c'), _) | Event::KeyPress(KeyCode::Char('C'), _) => {
                self.handle_cancel()
            }
            Event::KeyPress(KeyCode::Char('r'), _) | Event::KeyPress(KeyCode::Char('R'), _) => {
                self.handle_reset()
            }
            _ => Ok(EventState::NotConsumed),
        }
    }

    fn handle_tick(&mut self, dt: &Duration) -> Result<()> {
        self.ui_state = self.ui_state.handle_tick(dt, &mut self.state);
        Ok(())
    }

    fn render(&self, f: &mut Frame, rect: Rect) {
        let outer = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Klondike")
            .title(
                Title::from(format!(
                    "┤ {} ├",
                    match self.ui_state {
                        UIState::Dealing(_) => "skip: ␣",
                        UIState::Hovering(pile) => match pile {
                            HoveringState::Stock => "navigate: ← ↑ ↓ → | draw: ␣ | [r]estart",
                            HoveringState::Talon => {
                                "navigate: ← ↑ ↓ → | move: ⇧ + ← ↑ ↓ → | [r]estart"
                            }
                            HoveringState::Foundation(_) => {
                                "navigate: ← ↑ ↓ → | move: ⇧ + ← ↑ ↓ → | [r]estart"
                            }
                            HoveringState::Tableau(_) => {
                                "navigate: ← ↑ ↓ → | move: ⇧ + ← → | take more: ⇧ + ↑ | [r]estart"
                            }
                        },
                        UIState::Selecting(_) => {
                            "take more: ⇧ + ↑ | take less: ↓ | move: ← → | [c]ancel | [r]estart"
                        }
                        UIState::Moving(_) => "move: ← ↑ ↓ → | place: ␣ | [c]ancel | [r]estart",
                    }
                ))
                .position(Position::Bottom)
                .alignment(Alignment::Left),
            );

        let inner_rect = outer.inner(rect);

        render::GameState::from((&self.state, &self.ui_state)).render(f, inner_rect);

        f.render_widget(outer, rect);
    }
}

impl<RNG: rand::Rng> GameComponent<RNG> {
    pub fn new(rng: RNG) -> GameComponent<RNG> {
        let mut rng = rng;
        let state = klondike::InitialGameState::new_with_rng(&mut rng);
        GameComponent {
            rng,
            state: klondike::GameStateOption::from(state),
            ui_state: UIState::Dealing(DealingState::new()),
        }
    }

    fn handle_direction(
        &mut self,
        dir: ui_state::Direction,
        modifier: KeyModifiers,
    ) -> EventResult {
        self.ui_state = self.ui_state.handle_direction(dir, modifier, &self.state);
        Ok(EventState::Consumed)
    }

    fn handle_interact(&mut self) -> EventResult {
        self.ui_state = self.ui_state.handle_interact(&mut self.state);
        Ok(EventState::Consumed)
    }

    fn handle_goto(&mut self, c: u32) -> EventResult {
        self.ui_state = self.ui_state.handle_goto(c as u8);
        Ok(EventState::Consumed)
    }

    fn handle_cancel(&mut self) -> EventResult {
        self.ui_state = self.ui_state.handle_cancel();
        Ok(EventState::Consumed)
    }

    fn handle_reset(&mut self) -> EventResult {
        self.state = klondike::GameStateOption::from(klondike::InitialGameState::new_with_rng(
            &mut self.rng,
        ));
        self.ui_state = UIState::Dealing(DealingState::new());
        Ok(EventState::Consumed)
    }
}
