use crossterm::event::KeyModifiers;
use solitaire::{
    variant::{klondike, klondike::GameStateOption},
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
    /// Default state, the user is hovering over a pile
    Hovering(HoveringState),
    /// The user is in the process of selecting one (or many) cards
    Selecting(SelectingState),
    /// The user is moving their selected cards to another pile
    Moving(MovingState),
}

pub trait State {
    fn handle_direction(
        self,
        dir: Direction,
        modifier: KeyModifiers,
        game_state: &GameStateOption,
    ) -> UIState;

    fn handle_interact(self, modifier: KeyModifiers, game_state: &mut GameStateOption) -> UIState;
}

impl State for UIState {
    fn handle_direction(
        self,
        dir: Direction,
        modifier: KeyModifiers,
        game_state: &GameStateOption,
    ) -> UIState {
        match self {
            UIState::Hovering(s) => s.handle_direction(dir, modifier, game_state),
            UIState::Selecting(s) => s.handle_direction(dir, modifier, game_state),
            UIState::Moving(s) => s.handle_direction(dir, modifier, game_state),
        }
    }

    fn handle_interact(self, modifier: KeyModifiers, game_state: &mut GameStateOption) -> UIState {
        match self {
            UIState::Hovering(s) => s.handle_interact(modifier, game_state),
            UIState::Selecting(s) => s.handle_interact(modifier, game_state),
            UIState::Moving(s) => s.handle_interact(modifier, game_state),
        }
    }
}

pub type HoveringState = klondike::PileRef;

impl State for HoveringState {
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
                            _ => UIState::Selecting(SelectingState::Tableau { pile_n, take_n: 2 }),
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
                        2 => UIState::Hovering(self),
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

    fn handle_interact(self, _: KeyModifiers, game_state: &mut GameStateOption) -> UIState {
        match game_state {
            GameStateOption::Playing(play) => match self {
                HoveringState::Stock => match klondike::GameRules::draw_stock(play.clone(), 1) {
                    Ok(new_state) => *game_state = GameStateOption::Playing(new_state),
                    Err(_) => return UIState::Hovering(self),
                },
                // todo auto move
                _ => {}
            },
            _ => {}
        }
        UIState::Hovering(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SelectingState {
    Tableau { pile_n: usize, take_n: usize },
}

impl State for SelectingState {
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
                        let pile_len = game_state
                            .get_stack(klondike::PileRef::Tableau(pile_n))
                            .unwrap()
                            .len();
                        // No more can be selected
                        if pile_len == take_n {
                            return UIState::Selecting(self);
                        }
                        // todo validate the sequence
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
                Direction::Down => match modifier {
                    KeyModifiers::SHIFT => match take_n {
                        // If only 2 cards are currently selected, move out of selecting mode
                        2 => UIState::Hovering(HoveringState::Tableau(pile_n)),
                        // Otherwise decrease take_n by 1
                        _ => UIState::Selecting(SelectingState::Tableau {
                            pile_n,
                            take_n: take_n - 1,
                        }),
                    },
                    // Cannot move down so do nothing
                    _ => UIState::Selecting(self),
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

    fn handle_interact(self, _: KeyModifiers, _: &mut GameStateOption) -> UIState {
        // no-op
        UIState::Selecting(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MovingState {
    pub src: klondike::PileRef,
    pub take_n: usize,
    pub dst: klondike::PileRef,
}

impl State for MovingState {
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

    fn handle_interact(self, _: KeyModifiers, game_state: &mut GameStateOption) -> UIState {
        match game_state {
            GameStateOption::Playing(play_state) => {
                match klondike::GameRules::move_cards(
                    play_state.clone(),
                    self.src,
                    self.take_n,
                    self.dst,
                ) {
                    Ok(result) => {
                        *game_state = GameStateOption::from(result);
                        return UIState::Hovering(self.dst);
                    }
                    Err(_) => return UIState::Hovering(self.src),
                }
            }
            _ => {}
        }
        UIState::Hovering(self.src)
    }
}
