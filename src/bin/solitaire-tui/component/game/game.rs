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
            ui_state::{HoveringState, State, UIState},
        },
        Component,
    },
    error::Result,
    event::Event,
};

pub struct GameComponent {
    state: klondike::GameStateOption,
    ui_state: UIState,
}

impl<'a> Component for GameComponent {
    fn init(&mut self) -> Result<()> {
        self.reset();
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::KeyPress(KeyCode::Up, m)
            | Event::KeyPress(KeyCode::Char('w'), m)
            | Event::KeyPress(KeyCode::Char('W'), m) => self.handle_up(*m),
            Event::KeyPress(KeyCode::Down, m)
            | Event::KeyPress(KeyCode::Char('s'), m)
            | Event::KeyPress(KeyCode::Char('S'), m) => self.handle_down(*m),
            Event::KeyPress(KeyCode::Left, m)
            | Event::KeyPress(KeyCode::Char('a'), m)
            | Event::KeyPress(KeyCode::Char('A'), m) => self.handle_left(*m),
            Event::KeyPress(KeyCode::Right, m)
            | Event::KeyPress(KeyCode::Char('d'), m)
            | Event::KeyPress(KeyCode::Char('D'), m) => self.handle_right(*m),
            Event::KeyPress(KeyCode::Enter, m) | Event::KeyPress(KeyCode::Char(' '), m) => {
                self.handle_interact(*m)
            }
            _ => Ok(()),
        }
    }

    fn render(&self, f: &mut Frame, rect: Rect) {
        let outer = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Klondike")
            .title(
                Title::from(match self.ui_state {
                    UIState::Hovering(pile) => match pile {
                        HoveringState::Stock => "| navigate: ← ↑ ↓ → | draw: ␣ |",
                        HoveringState::Talon => "| navigate: ← ↑ ↓ → | move: ⇧ + ← ↑ ↓ → |",
                        HoveringState::Foundation(_) => "| navigate: ← ↑ ↓ → | move: ⇧ + ← ↑ ↓ → |",
                        HoveringState::Tableau(_) => {
                            "| navigate: ← ↑ ↓ → | move: ⇧ + ← → | take more: ⇧ + ↑ |"
                        }
                    },
                    UIState::Selecting(_) => "| take more: ⇧ + ↑ | take less: ↓ | move: ← → |",
                    UIState::Moving(_) => "| move: ← ↑ ↓ → | place: ␣ |",
                })
                .position(Position::Bottom)
                .alignment(Alignment::Left),
            );

        let inner_rect = outer.inner(rect);

        render::GameState::from((&self.state, &self.ui_state)).render(f, inner_rect);

        f.render_widget(outer, rect);
    }
}

impl GameComponent {
    pub fn new() -> GameComponent {
        GameComponent {
            state: klondike::GameStateOption::from(klondike::GameRules::new_and_deal()),
            ui_state: UIState::Hovering(HoveringState::Stock),
        }
    }

    fn handle_up(&mut self, modifier: KeyModifiers) -> Result<()> {
        self.ui_state =
            self.ui_state
                .handle_direction(ui_state::Direction::Up, modifier, &self.state);
        Ok(())
    }

    fn handle_down(&mut self, modifier: KeyModifiers) -> Result<()> {
        self.ui_state =
            self.ui_state
                .handle_direction(ui_state::Direction::Down, modifier, &self.state);
        Ok(())
    }

    fn handle_left(&mut self, modifier: KeyModifiers) -> Result<()> {
        self.ui_state =
            self.ui_state
                .handle_direction(ui_state::Direction::Left, modifier, &self.state);
        Ok(())
    }

    fn handle_right(&mut self, modifier: KeyModifiers) -> Result<()> {
        self.ui_state =
            self.ui_state
                .handle_direction(ui_state::Direction::Right, modifier, &self.state);
        Ok(())
    }

    fn handle_interact(&mut self, modifier: KeyModifiers) -> Result<()> {
        self.ui_state = self.ui_state.handle_interact(modifier, &mut self.state);
        Ok(())
    }

    fn reset(&mut self) {
        *self = GameComponent {
            state: klondike::GameStateOption::from(klondike::GameRules::new_and_deal()),
            ui_state: UIState::Hovering(HoveringState::Stock),
        }
    }
}
