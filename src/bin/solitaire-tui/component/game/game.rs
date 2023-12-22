use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders,
    },
    Frame,
};
use solitaire::{variant::klondike, GameState};

use crate::{
    component::{
        game::{
            render,
            render::{CARD_HEIGHT, CARD_WIDTH},
            ui_state,
            ui_state::{HoveringState, MovingState, SelectingState, State, UIState},
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

        let width = CARD_WIDTH * klondike::NUM_TABLEAU as u16;
        let horizontal_pad = inner_rect.width.checked_sub(width).unwrap_or(0);

        let inner_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(width), Constraint::Min(horizontal_pad)])
            .split(inner_rect)[0];

        let vstack = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(CARD_HEIGHT),
                Constraint::Length(CARD_HEIGHT * 3),
            ])
            .split(inner_rect);

        // Render the top row
        {
            let top = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(CARD_WIDTH),
                    // Talon is two widths wide
                    Constraint::Length(CARD_WIDTH * 2),
                    Constraint::Length(CARD_WIDTH),
                    Constraint::Length(CARD_WIDTH),
                    Constraint::Length(CARD_WIDTH),
                    Constraint::Length(CARD_WIDTH),
                ])
                .split(vstack[0]);

            render::stock(
                self.state
                    .get_stack(klondike::PileRef::Stock)
                    .map_or(&[], |s| s.as_slice()),
                self.ui_state == UIState::Hovering(HoveringState::Stock),
                f,
                top[0],
            );

            render::talon(
                self.get_cards_during_move(klondike::PileRef::Talon)
                    .as_slice(),
                self.ui_state == UIState::Hovering(HoveringState::Talon),
                f,
                top[1],
            );

            for (i, foundation_rect) in top[2..6].iter().enumerate() {
                let cards = self.get_cards_during_move(klondike::PileRef::Foundation(i));
                render::foundation(
                    cards.as_slice(),
                    self.ui_state == UIState::Hovering(HoveringState::Foundation(i)),
                    f,
                    *foundation_rect,
                );
            }
        }

        // Render the tableau
        let tableau = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([(); klondike::NUM_TABLEAU].map(|_| Constraint::Length(CARD_WIDTH)))
            .split(vstack[1]);

        for (i, tableau_rect) in tableau.iter().enumerate() {
            let cards = self.get_cards_during_move(klondike::PileRef::Tableau(i));

            let selected = match self.ui_state {
                UIState::Hovering(HoveringState::Tableau(pile_n)) => {
                    if pile_n == i {
                        render::TableauSelected::Selected(1)
                    } else {
                        render::TableauSelected::Unselected
                    }
                }
                UIState::Selecting(SelectingState::Tableau { pile_n, take_n }) => {
                    if pile_n == i {
                        render::TableauSelected::Selected(take_n)
                    } else {
                        render::TableauSelected::Unselected
                    }
                }
                _ => render::TableauSelected::Unselected,
            };

            render::tableau(cards.as_slice(), selected, f, *tableau_rect);
        }

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

    fn get_cards_during_move(&self, p: klondike::PileRef) -> klondike::Stack {
        // Get the pile's stack
        let mut cards: klondike::Stack = self
            .state
            .get_stack(p)
            .map_or_else(|| Vec::new(), |s| s.clone());

        // Match the moving state
        match self.ui_state {
            UIState::Moving(MovingState { src, take_n, dst }) => {
                // If the src == dst is a no op
                if src == dst {

                    // This is the source of the move
                } else if src == p {
                    // Remove take_n cards from the pile
                    cards.truncate(cards.len() - take_n);

                    // This is the destination of the move
                } else if dst == p {
                    // Take cards from the source
                    let src_cards: &[klondike::Card] = &self
                        .state
                        .get_stack(src)
                        .map_or(&[] as &[klondike::Card], |s| s.as_slice());

                    let src_cards = &src_cards[src_cards.len() - take_n..];

                    // Concatenate this pile and the source cards
                    cards = cards
                        .iter()
                        .chain(src_cards.iter())
                        .cloned()
                        .collect::<Vec<klondike::Card>>()
                }
            }
            _ => {}
        };

        cards
    }

    fn reset(&mut self) {
        *self = GameComponent {
            state: klondike::GameStateOption::from(klondike::GameRules::new_and_deal()),
            ui_state: UIState::Hovering(HoveringState::Stock),
        }
    }
}
