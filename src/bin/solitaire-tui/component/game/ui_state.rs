use std::time::Duration;

use crossterm::event::KeyModifiers;
use solitaire::{
    variant::{
        klondike,
        klondike::{DealResult, GameStateOption},
    },
    GameState,
};

#[derive(Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Enum describing the various states the UI can be in
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum UIState {
    /// Animated dealing state at the start of a game
    /// while the cards are being dealt onto the tableau
    Dealing(DealingState),
    /// Default state, the user is hovering over a pile
    Hovering(HoveringState),
    /// The user is in the process of selecting one (or many) cards
    Selecting(SelectingState),
    /// The user is moving their selected cards to another pile
    Moving(MovingState),
}

pub trait State: Sized {
    fn handle_tick(self, dt: &Duration, game_state: &mut GameStateOption) -> UIState;

    fn handle_direction(
        self,
        dir: Direction,
        modifier: KeyModifiers,
        game_state: &GameStateOption,
    ) -> UIState;

    fn handle_interact(self, game_state: &mut GameStateOption) -> UIState;

    fn handle_goto(self, i: u8) -> UIState;

    fn handle_cancel(self) -> UIState;
}

impl State for UIState {
    fn handle_tick(self, dt: &Duration, game_state: &mut GameStateOption) -> UIState {
        match self {
            UIState::Dealing(s) => s.handle_tick(dt, game_state),
            UIState::Hovering(s) => s.handle_tick(dt, game_state),
            UIState::Selecting(s) => s.handle_tick(dt, game_state),
            UIState::Moving(s) => s.handle_tick(dt, game_state),
        }
    }

    fn handle_direction(
        self,
        dir: Direction,
        modifier: KeyModifiers,
        game_state: &GameStateOption,
    ) -> UIState {
        match self {
            UIState::Dealing(s) => s.handle_direction(dir, modifier, game_state),
            UIState::Hovering(s) => s.handle_direction(dir, modifier, game_state),
            UIState::Selecting(s) => s.handle_direction(dir, modifier, game_state),
            UIState::Moving(s) => s.handle_direction(dir, modifier, game_state),
        }
    }

    fn handle_interact(self, game_state: &mut GameStateOption) -> UIState {
        match self {
            UIState::Dealing(s) => s.handle_interact(game_state),
            UIState::Hovering(s) => s.handle_interact(game_state),
            UIState::Selecting(s) => s.handle_interact(game_state),
            UIState::Moving(s) => s.handle_interact(game_state),
        }
    }

    fn handle_goto(self, i: u8) -> UIState {
        match self {
            UIState::Dealing(s) => s.handle_goto(i),
            UIState::Hovering(s) => s.handle_goto(i),
            UIState::Selecting(s) => s.handle_goto(i),
            UIState::Moving(s) => s.handle_goto(i),
        }
    }

    fn handle_cancel(self) -> UIState {
        match self {
            UIState::Dealing(s) => s.handle_cancel(),
            UIState::Hovering(s) => s.handle_cancel(),
            UIState::Selecting(s) => s.handle_cancel(),
            UIState::Moving(s) => s.handle_cancel(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DealingState {
    since_last_deal: Duration,
}

impl DealingState {
    const DEAL_INTERVAL: Duration = Duration::from_millis(100);

    pub fn new() -> Self {
        DealingState {
            since_last_deal: Duration::from_secs(0),
        }
    }
}

impl State for DealingState {
    fn handle_tick(self, dt: &Duration, game_state: &mut GameStateOption) -> UIState {
        let mut since_last_deal = self.since_last_deal + *dt;
        // Keep dealing until all the expected cards have been dealt,
        // so that slow downs don't cause fewer cards to be dealt
        while since_last_deal >= Self::DEAL_INTERVAL {
            since_last_deal = since_last_deal - Self::DEAL_INTERVAL;
            match game_state {
                GameStateOption::Initial(initial) => {
                    match klondike::GameRules::deal_one(initial.clone()) {
                        DealResult::Dealing(new_state) => {
                            *game_state = GameStateOption::from(new_state);
                        }
                        DealResult::Complete(new_state) => {
                            *game_state = GameStateOption::from(new_state);
                            // Deal complete, so move to hovering state
                            return UIState::Hovering(HoveringState::Stock);
                        }
                    }
                }
                // Idiot check
                _ => return UIState::Hovering(HoveringState::Stock),
            }
        }
        UIState::Dealing(DealingState { since_last_deal })
    }

    fn handle_direction(self, _: Direction, _: KeyModifiers, _: &GameStateOption) -> UIState {
        UIState::Dealing(self)
    }

    fn handle_interact(self, game_state: &mut GameStateOption) -> UIState {
        // Interact skips dealing
        match game_state {
            GameStateOption::Initial(initial) => {
                *game_state = GameStateOption::from(klondike::GameRules::deal_all(initial.clone()));
            }
            _ => {}
        }
        UIState::Hovering(HoveringState::Stock)
    }

    fn handle_goto(self, _: u8) -> UIState {
        UIState::Dealing(self)
    }

    fn handle_cancel(self) -> UIState {
        UIState::Dealing(self)
    }
}

pub type HoveringState = klondike::PileRef;

impl State for HoveringState {
    fn handle_tick(self, _: &Duration, _: &mut GameStateOption) -> UIState {
        // no-op
        UIState::Hovering(self)
    }

    fn handle_direction(
        self,
        dir: Direction,
        modifier: KeyModifiers,
        game_state: &GameStateOption,
    ) -> UIState {
        match modifier {
            // Selection mode
            KeyModifiers::SHIFT => {
                let pile_len = game_state.get_stack(self).unwrap().len();

                // Do nothing on an empty pile
                if pile_len == 0 {
                    return UIState::Hovering(self);
                }

                match self {
                    // Not applicable to the stock
                    HoveringState::Stock => UIState::Hovering(self),
                    // Moving talon
                    HoveringState::Talon => match dir {
                        // Up and left aren't possible
                        Direction::Up | Direction::Left => UIState::Hovering(self),
                        // Down is the second tableau
                        Direction::Down => UIState::Moving(MovingState {
                            src: klondike::PileRef::Talon,
                            take_n: 1,
                            dst: klondike::PileRef::Tableau(1),
                        }),
                        // Right is the first foundation
                        Direction::Right => UIState::Moving(MovingState {
                            src: klondike::PileRef::Talon,
                            take_n: 1,
                            dst: klondike::PileRef::Foundation(0),
                        }),
                    },
                    // Moving foundation
                    HoveringState::Foundation(pile_n) => match dir {
                        // Up isn't possible
                        Direction::Up => UIState::Hovering(self),
                        // Down is tableau
                        Direction::Down => UIState::Moving(MovingState {
                            src: klondike::PileRef::Foundation(pile_n),
                            take_n: 1,
                            dst: klondike::PileRef::Tableau(pile_n + 3),
                        }),
                        // Left and right are foundations
                        Direction::Left => match pile_n {
                            // Cannot move to talon
                            0 => UIState::Hovering(self),
                            _ => UIState::Moving(MovingState {
                                src: klondike::PileRef::Foundation(pile_n),
                                take_n: 1,
                                dst: klondike::PileRef::Foundation(pile_n - 1),
                            }),
                        },
                        Direction::Right => match pile_n {
                            3 => UIState::Hovering(self),
                            _ => UIState::Moving(MovingState {
                                src: klondike::PileRef::Foundation(pile_n),
                                take_n: 1,
                                dst: klondike::PileRef::Foundation(pile_n + 1),
                            }),
                        },
                    },
                    HoveringState::Tableau(pile_n) => match dir {
                        // Down isn't possible
                        Direction::Down => UIState::Hovering(self),
                        // Up can either be selecting or moving, depending on pile_len
                        Direction::Up => match pile_len {
                            // Only one element, so moving
                            1 => match pile_n {
                                // Cannot move to stock, talon or empty space
                                0..=2 => UIState::Hovering(self),
                                // Move to foundation
                                _ => UIState::Moving(MovingState {
                                    src: klondike::PileRef::Tableau(pile_n),
                                    take_n: 1,
                                    dst: klondike::PileRef::Foundation(pile_n - 3),
                                }),
                            },
                            // Selecting tableau
                            _ => {
                                let pile = game_state
                                    .get_stack(klondike::PileRef::Tableau(pile_n))
                                    .unwrap();
                                // Cannot select a face down card
                                if !pile.get(pile.len() - 2).unwrap().face_up {
                                    return UIState::Hovering(self);
                                }
                                UIState::Selecting(SelectingState::Tableau { pile_n, take_n: 2 })
                            }
                        },
                        // Left and right are tableaus
                        Direction::Left => match pile_n {
                            0 => UIState::Hovering(self),
                            _ => UIState::Moving(MovingState {
                                src: klondike::PileRef::Tableau(pile_n),
                                take_n: 1,
                                dst: klondike::PileRef::Tableau(pile_n - 1),
                            }),
                        },
                        Direction::Right => match pile_n {
                            6 => UIState::Hovering(self),
                            _ => UIState::Moving(MovingState {
                                src: klondike::PileRef::Tableau(pile_n),
                                take_n: 1,
                                dst: klondike::PileRef::Tableau(pile_n + 1),
                            }),
                        },
                    },
                }
            }
            // Hovering
            _ => match self {
                HoveringState::Stock => match dir {
                    // Up and left aren't possible
                    Direction::Up | Direction::Left => UIState::Hovering(self),
                    // Right is talon
                    Direction::Right => UIState::Hovering(HoveringState::Talon),
                    // Down is tableau
                    Direction::Down => UIState::Hovering(HoveringState::Tableau(0)),
                },
                HoveringState::Talon => match dir {
                    // Up isn't possible
                    Direction::Up => UIState::Hovering(self),
                    // Left is stock
                    Direction::Left => UIState::Hovering(HoveringState::Stock),
                    // Right is foundation
                    Direction::Right => UIState::Hovering(HoveringState::Foundation(0)),
                    // Down is tableau
                    Direction::Down => UIState::Hovering(HoveringState::Tableau(1)),
                },
                HoveringState::Foundation(pile_n) => match dir {
                    // Up isn't possible
                    Direction::Up => UIState::Hovering(self),
                    Direction::Left => match pile_n {
                        0 => UIState::Hovering(HoveringState::Talon),
                        _ => UIState::Hovering(HoveringState::Foundation(pile_n - 1)),
                    },
                    Direction::Right => match pile_n {
                        3 => UIState::Hovering(self),
                        _ => UIState::Hovering(HoveringState::Foundation(pile_n + 1)),
                    },
                    // Down is tableau
                    Direction::Down => UIState::Hovering(HoveringState::Tableau(pile_n + 3)),
                },
                HoveringState::Tableau(pile_n) => match dir {
                    // Down isn't possible
                    Direction::Down => UIState::Hovering(self),
                    // Up is top row
                    Direction::Up => match pile_n {
                        0 => UIState::Hovering(HoveringState::Stock),
                        1 => UIState::Hovering(HoveringState::Talon),
                        2 => UIState::Hovering(HoveringState::Talon),
                        _ => UIState::Hovering(HoveringState::Foundation(pile_n - 3)),
                    },
                    Direction::Left => match pile_n {
                        0 => UIState::Hovering(self),
                        _ => UIState::Hovering(HoveringState::Tableau(pile_n - 1)),
                    },
                    Direction::Right => match pile_n {
                        6 => UIState::Hovering(self),
                        _ => UIState::Hovering(HoveringState::Tableau(pile_n + 1)),
                    },
                },
            },
        }
    }

    fn handle_interact(self, game_state: &mut GameStateOption) -> UIState {
        match game_state {
            GameStateOption::Playing(play) => match self {
                HoveringState::Stock => match klondike::GameRules::draw_stock(play.clone(), 1) {
                    Ok(new_state) => *game_state = GameStateOption::Playing(new_state),
                    Err(_) => return UIState::Hovering(self),
                },
                p => match klondike::GameRules::auto_move_card(play.clone(), p, 1) {
                    Ok(new_state) => *game_state = GameStateOption::from(new_state),
                    Err(_) => return UIState::Hovering(self),
                },
            },
            _ => {}
        }
        UIState::Hovering(self)
    }

    fn handle_goto(self, i: u8) -> UIState {
        match i {
            1 => UIState::Hovering(HoveringState::Stock),
            2 => UIState::Hovering(HoveringState::Talon),
            i @ 3..=6 => UIState::Hovering(HoveringState::Foundation(i as usize - 3)),
            _ => UIState::Hovering(self),
        }
    }

    fn handle_cancel(self) -> UIState {
        // no-op
        UIState::Hovering(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SelectingState {
    Tableau { pile_n: usize, take_n: usize },
}

impl State for SelectingState {
    fn handle_tick(self, _: &Duration, _: &mut GameStateOption) -> UIState {
        // no-op
        UIState::Selecting(self)
    }

    fn handle_direction(
        self,
        dir: Direction,
        modifier: KeyModifiers,
        game_state: &GameStateOption,
    ) -> UIState {
        match self {
            SelectingState::Tableau { pile_n, take_n } => match dir {
                Direction::Up => match modifier {
                    // Increase the cards selected
                    KeyModifiers::SHIFT => {
                        let pile = game_state
                            .get_stack(klondike::PileRef::Tableau(pile_n))
                            .unwrap();
                        // No more can be selected
                        if pile.len() == take_n {
                            return UIState::Selecting(self);
                        }
                        // Cannot select a face down card
                        if !pile.get(pile.len() - (take_n + 1)).unwrap().face_up {
                            return UIState::Selecting(self);
                        }
                        // Otherwise increase take_n by 1
                        UIState::Selecting(SelectingState::Tableau {
                            pile_n,
                            take_n: take_n + 1,
                        })
                    }
                    // Move up
                    _ => {
                        let dst = match pile_n {
                            // Cannot move to stock, talon or empty space
                            0..=2 => return UIState::Selecting(self),
                            _ => klondike::PileRef::Foundation(pile_n - 3),
                        };
                        UIState::Moving(MovingState {
                            src: klondike::PileRef::Tableau(pile_n),
                            take_n,
                            dst,
                        })
                    }
                },
                // Decrease
                Direction::Down => match take_n {
                    // If only 2 cards are currently selected, move out of selecting mode
                    2 => UIState::Hovering(HoveringState::Tableau(pile_n)),
                    // Otherwise decrease take_n by 1
                    _ => UIState::Selecting(SelectingState::Tableau {
                        pile_n,
                        take_n: take_n - 1,
                    }),
                },
                // Move the cards left/right
                Direction::Left => match pile_n {
                    0 => UIState::Selecting(self),
                    _ => UIState::Moving(MovingState {
                        src: klondike::PileRef::Tableau(pile_n),
                        take_n,
                        dst: klondike::PileRef::Tableau(pile_n - 1),
                    }),
                },
                Direction::Right => match pile_n {
                    6 => UIState::Selecting(self),
                    _ => UIState::Moving(MovingState {
                        src: klondike::PileRef::Tableau(pile_n),
                        take_n,
                        dst: klondike::PileRef::Tableau(pile_n + 1),
                    }),
                },
            },
        }
    }

    fn handle_interact(self, game_state: &mut GameStateOption) -> UIState {
        match game_state {
            GameStateOption::Playing(play) => match self {
                SelectingState::Tableau { pile_n, take_n } => {
                    match klondike::GameRules::auto_move_card(
                        play.clone(),
                        klondike::PileRef::Tableau(pile_n),
                        take_n,
                    ) {
                        Ok(new_state) => {
                            *game_state = GameStateOption::from(new_state);
                            UIState::Hovering(HoveringState::Tableau(pile_n))
                        }
                        Err(_) => UIState::Selecting(self),
                    }
                }
            },
            _ => UIState::Selecting(self),
        }
    }

    fn handle_goto(self, i: u8) -> UIState {
        match self {
            SelectingState::Tableau { pile_n, take_n } => UIState::Moving(MovingState {
                src: klondike::PileRef::Tableau(pile_n),
                take_n,
                dst: match i {
                    1 => klondike::PileRef::Stock,
                    2 => klondike::PileRef::Talon,
                    i @ 3..=6 => klondike::PileRef::Foundation(i as usize - 3),
                    _ => return UIState::Selecting(self),
                },
            }),
        }
    }

    fn handle_cancel(self) -> UIState {
        match self {
            SelectingState::Tableau { pile_n, .. } => {
                UIState::Hovering(klondike::PileRef::Tableau(pile_n))
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MovingState {
    pub src: klondike::PileRef,
    pub take_n: usize,
    pub dst: klondike::PileRef,
}

impl State for MovingState {
    fn handle_tick(self, _: &Duration, _: &mut GameStateOption) -> UIState {
        // no-op
        UIState::Moving(self)
    }

    fn handle_direction(self, dir: Direction, _: KeyModifiers, _: &GameStateOption) -> UIState {
        let dst = match self.dst {
            // Shouldn't be possible, but handle it anyway
            klondike::PileRef::Stock => match dir {
                // Up and left isn't possible
                Direction::Up | Direction::Left => return UIState::Moving(self),
                Direction::Right => klondike::PileRef::Foundation(0),
                Direction::Down => klondike::PileRef::Tableau(0),
            },
            klondike::PileRef::Talon => match dir {
                // Up and left isn't possible
                Direction::Up | Direction::Left => return UIState::Moving(self),
                // Right is foundation
                Direction::Right => klondike::PileRef::Foundation(0),
                // Down is tableau
                Direction::Down => klondike::PileRef::Tableau(1),
            },
            klondike::PileRef::Foundation(pile_n) => match dir {
                // Up isn't possible
                Direction::Up => return UIState::Moving(self),
                Direction::Left => match pile_n {
                    0 => return UIState::Moving(self),
                    _ => klondike::PileRef::Foundation(pile_n - 1),
                },
                Direction::Right => match pile_n {
                    3 => return UIState::Moving(self),
                    _ => klondike::PileRef::Foundation(pile_n + 1),
                },
                // Down is tableau
                Direction::Down => klondike::PileRef::Tableau(pile_n + 3),
            },
            klondike::PileRef::Tableau(pile_n) => match dir {
                // Down isn't possible
                Direction::Down => return UIState::Moving(self),
                // Up is top row
                Direction::Up => match self.take_n {
                    // Can only move to foundations if the stack is 1
                    1 => match pile_n {
                        0..=2 => return UIState::Moving(self),
                        _ => klondike::PileRef::Foundation(pile_n - 3),
                    },
                    _ => return UIState::Moving(self),
                },
                Direction::Left => match pile_n {
                    0 => return UIState::Moving(self),
                    _ => klondike::PileRef::Tableau(pile_n - 1),
                },
                Direction::Right => match pile_n {
                    6 => return UIState::Moving(self),
                    _ => klondike::PileRef::Tableau(pile_n + 1),
                },
            },
        };
        UIState::Moving(MovingState { dst, ..self })
    }

    fn handle_interact(self, game_state: &mut GameStateOption) -> UIState {
        match game_state {
            GameStateOption::Playing(play_state) => {
                return match klondike::GameRules::move_cards(
                    play_state.clone(),
                    self.src,
                    self.take_n,
                    self.dst,
                ) {
                    Ok(result) => {
                        *game_state = GameStateOption::from(result);
                        UIState::Hovering(self.dst)
                    }
                    Err(_) => UIState::Hovering(self.src),
                }
            }
            _ => {}
        }
        UIState::Hovering(self.src)
    }

    fn handle_goto(self, i: u8) -> UIState {
        UIState::Moving(MovingState {
            src: self.src,
            take_n: self.take_n,
            dst: match i {
                i @ 3..=6 => klondike::PileRef::Foundation(i as usize - 3),
                _ => return UIState::Moving(self),
            },
        })
    }

    fn handle_cancel(self) -> UIState {
        UIState::Hovering(self.src)
    }
}
