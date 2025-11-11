use web_time::Duration;

use crate::variant::klondike;
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders,
    },
    Frame,
};

use crate::{
    ui::component::{
        game::{
            render::RenderState,
            ui_state,
            ui_state::{DealingState, HoveringState, State, UIState},
        },
        Component,
    },
    ui::error::Result,
    ui::event::{Event, EventResult, EventState, KeyCode, Modifiers},
};

pub struct GameComponent<RNG: rand::Rng> {
    rng: RNG,
    state: klondike::GameStateOption,
    ui_state: UIState,
    last_render_state: Option<RenderState>,
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
                self.handle_event(ui_state::Event::Interact)
            }
            Event::KeyPress(KeyCode::Char(c @ '1'..='9'), _) => {
                self.handle_event(ui_state::Event::Goto(*c as u8))
            }
            Event::KeyPress(KeyCode::Char('c'), _) | Event::KeyPress(KeyCode::Char('C'), _) => {
                self.handle_event(ui_state::Event::Cancel)
            }
            Event::KeyPress(KeyCode::Char('r'), _) | Event::KeyPress(KeyCode::Char('R'), _) => {
                self.handle_reset()
            }
            Event::MousePress(col, row, _) => self.handle_click(*col, *row),
            _ => Ok(EventState::NotConsumed),
        }
    }

    fn handle_tick(&mut self, dt: &Duration) -> Result<()> {
        self.handle_event(ui_state::Event::Tick(*dt))?;
        Ok(())
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
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
                        UIState::AutoMoving(_) => "auto moving...",
                    }
                ))
                .position(Position::Bottom)
                .alignment(Alignment::Left),
            );

        let inner_rect = outer.inner(rect);

        let render_state = RenderState::new(&self.state, &self.ui_state, inner_rect);
        render_state.render(f);
        self.last_render_state = Some(render_state);

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
            last_render_state: None,
        }
    }

    fn handle_direction(&mut self, dir: ui_state::Direction, modifier: Modifiers) -> EventResult {
        self.handle_event(ui_state::Event::Direction { dir, modifier })
    }

    fn handle_reset(&mut self) -> EventResult {
        self.state = klondike::GameStateOption::from(klondike::InitialGameState::new_with_rng(
            &mut self.rng,
        ));
        self.ui_state = UIState::Dealing(DealingState::new());
        Ok(EventState::Consumed)
    }

    fn handle_click(&mut self, col: u16, row: u16) -> EventResult {
        let clicked_location = self
            .last_render_state
            .as_ref()
            .and_then(|render_state| render_state.find_card_at(col, row))
            .map(|(_, card_info)| card_info.location);
        self.handle_event(ui_state::Event::Click(clicked_location))
    }

    fn handle_event(&mut self, event: ui_state::Event) -> EventResult {
        self.ui_state = self.ui_state.on(event, &mut self.state);
        Ok(EventState::Consumed)
    }
}
