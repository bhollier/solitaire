use std::cmp::max;
use web_time::Duration;

use crate::ui::component::game::render::CardLocation;
use crate::ui::event::Modifiers;
use crate::{
    variant::{
        klondike,
        klondike::{DealResult, GameStateOption},
    },
    GameState,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Enum of all the events that the UI listens to
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Event {
    Tick(Duration),
    Direction { dir: Direction, modifier: Modifiers },
    Interact,
    Goto(u8),
    Cancel,
    Click(Option<CardLocation>),
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
    /// Animated auto move state when the game is moving
    /// safe cards to the foundation automatically
    AutoMoving(AutoMovingState),
}

pub trait State: Sized {
    fn on(self, event: Event, game_state: &mut GameStateOption) -> UIState;
}

impl State for UIState {
    fn on(self, event: Event, game_state: &mut GameStateOption) -> UIState {
        match self {
            UIState::Dealing(s) => s.on(event, game_state),
            UIState::Hovering(s) => s.on(event, game_state),
            UIState::Selecting(s) => s.on(event, game_state),
            UIState::Moving(s) => s.on(event, game_state),
            UIState::AutoMoving(s) => s.on(event, game_state),
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
    fn on(self, event: Event, game_state: &mut GameStateOption) -> UIState {
        match event {
            Event::Tick(dt) => {
                let mut since_last_deal = self.since_last_deal + dt;
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
                                    // Deal complete so move to hovering state,
                                    // unless there's already an ace available to move
                                    if AutoMovingState::can_auto_move(game_state) {
                                        return UIState::AutoMoving(AutoMovingState::new(
                                            HoveringState::Stock,
                                        ));
                                    }
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
            Event::Interact => {
                // Interact skips dealing
                match game_state {
                    GameStateOption::Initial(initial) => {
                        *game_state =
                            GameStateOption::from(klondike::GameRules::deal_all(initial.clone()));
                        if AutoMovingState::can_auto_move(game_state) {
                            return UIState::AutoMoving(AutoMovingState::new(HoveringState::Stock));
                        }
                    }
                    _ => {}
                }
                UIState::Hovering(HoveringState::Stock)
            }
            // Clicking while dealing skips it, like with interacting
            Event::Click(_) => self.on(Event::Interact, game_state),
            // All other events are a no-op
            _ => UIState::Dealing(self),
        }
    }
}

pub type HoveringState = klondike::PileRef;

impl State for HoveringState {
    fn on(self, event: Event, game_state: &mut GameStateOption) -> UIState {
        match event {
            Event::Direction { dir, modifier } => {
                match modifier {
                    // Selection mode
                    Modifiers { shift: true, .. } => {
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
                            // Moving foundation isn't possible
                            HoveringState::Foundation(_) => UIState::Hovering(self),
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
                                        UIState::Selecting(SelectingState::Tableau {
                                            pile_n,
                                            take_n: 2,
                                        })
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
                            Direction::Down => {
                                UIState::Hovering(HoveringState::Tableau(pile_n + 3))
                            }
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
            Event::Interact => {
                match game_state {
                    GameStateOption::Playing(play) => match self {
                        HoveringState::Stock => {
                            match klondike::GameRules::draw_stock(play.clone(), 1) {
                                Ok(new_state) => {
                                    *game_state = GameStateOption::Playing(new_state);
                                    if AutoMovingState::can_auto_move(game_state) {
                                        return UIState::AutoMoving(AutoMovingState::new(self));
                                    }
                                }
                                Err(_) => return UIState::Hovering(self),
                            }
                        }
                        p => match klondike::GameRules::auto_move_card(play.clone(), p, 1) {
                            Ok(new_state) => {
                                *game_state = GameStateOption::from(new_state);
                                if AutoMovingState::can_auto_move(game_state) {
                                    return UIState::AutoMoving(AutoMovingState::new(self));
                                }
                            }
                            Err(_) => return UIState::Hovering(self),
                        },
                    },
                    _ => {}
                }
                UIState::Hovering(self)
            }
            Event::Goto(i) => match i {
                1 => UIState::Hovering(HoveringState::Stock),
                2 => UIState::Hovering(HoveringState::Talon),
                i @ 3..=6 => UIState::Hovering(HoveringState::Foundation(i as usize - 3)),
                _ => UIState::Hovering(self),
            },
            Event::Click(Some(card_location)) => {
                let pile_ref = card_location.pile_ref();
                let pile = game_state.get_stack(pile_ref).unwrap();

                // If the pile is empty and not the stock, just move
                if pile.len() == 0 && pile_ref != klondike::PileRef::Stock {
                    return UIState::Hovering(pile_ref);
                }

                match pile_ref {
                    // Clicking the stock draws a new card
                    klondike::PileRef::Stock => {
                        match game_state {
                            GameStateOption::Playing(play) => {
                                match klondike::GameRules::draw_stock(play.clone(), 1) {
                                    Ok(new_state) => {
                                        *game_state = GameStateOption::Playing(new_state);
                                        if AutoMovingState::can_auto_move(game_state) {
                                            return UIState::AutoMoving(AutoMovingState::new(self));
                                        }
                                    }
                                    Err(_) => return UIState::Hovering(self),
                                }
                            }
                            _ => {}
                        }
                        UIState::Hovering(klondike::PileRef::Stock)
                    }
                    // Selecting talon
                    klondike::PileRef::Talon => UIState::Selecting(SelectingState::Talon),
                    // Hovering foundation
                    klondike::PileRef::Foundation(pile_n) => {
                        UIState::Hovering(HoveringState::Foundation(pile_n))
                    }
                    // Selecting tableau
                    klondike::PileRef::Tableau(pile_n) => {
                        let take_n = card_location.n_from_bottom().unwrap();
                        // Cannot select a face down card
                        if !pile.get(pile.len() - (take_n + 1)).unwrap().face_up {
                            return UIState::Selecting(SelectingState::Tableau {
                                pile_n,
                                take_n: 1,
                            });
                        }
                        UIState::Selecting(SelectingState::Tableau {
                            pile_n,
                            take_n: take_n + 1,
                        })
                    }
                }
            }
            // All other events are a no-op
            _ => UIState::Hovering(self),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SelectingState {
    Tableau { pile_n: usize, take_n: usize },
    Talon,
}

impl State for SelectingState {
    fn on(self, event: Event, game_state: &mut GameStateOption) -> UIState {
        match event {
            Event::Direction { dir, modifier } => {
                match self {
                    SelectingState::Tableau { pile_n, take_n } => match dir {
                        Direction::Up => match modifier {
                            // Increase the cards selected
                            Modifiers { shift: true, .. } => {
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
                    SelectingState::Talon => match dir {
                        // Up and left aren't possible
                        Direction::Up | Direction::Left => UIState::Selecting(self),
                        // Move the cards right/down
                        Direction::Right => UIState::Moving(MovingState {
                            src: klondike::PileRef::Talon,
                            take_n: 1,
                            dst: klondike::PileRef::Foundation(0),
                        }),
                        Direction::Down => UIState::Moving(MovingState {
                            src: klondike::PileRef::Talon,
                            take_n: 1,
                            dst: klondike::PileRef::Tableau(1),
                        }),
                    },
                }
            }
            Event::Interact => match game_state {
                GameStateOption::Playing(play) => match self {
                    SelectingState::Tableau { pile_n, take_n } => {
                        match klondike::GameRules::auto_move_card(
                            play.clone(),
                            klondike::PileRef::Tableau(pile_n),
                            take_n,
                        ) {
                            Ok(new_state) => {
                                *game_state = GameStateOption::from(new_state);
                                if AutoMovingState::can_auto_move(game_state) {
                                    return UIState::AutoMoving(AutoMovingState::new(
                                        klondike::PileRef::Tableau(pile_n),
                                    ));
                                }
                                UIState::Hovering(HoveringState::Tableau(pile_n))
                            }
                            Err(_) => UIState::Selecting(self),
                        }
                    }
                    SelectingState::Talon => {
                        match klondike::GameRules::auto_move_card(
                            play.clone(),
                            klondike::PileRef::Talon,
                            1,
                        ) {
                            Ok(new_state) => {
                                *game_state = GameStateOption::from(new_state);
                                if AutoMovingState::can_auto_move(game_state) {
                                    return UIState::AutoMoving(AutoMovingState::new(
                                        klondike::PileRef::Talon,
                                    ));
                                }
                                UIState::Hovering(HoveringState::Talon)
                            }
                            Err(_) => UIState::Selecting(self),
                        }
                    }
                },
                _ => UIState::Selecting(self),
            },
            Event::Goto(i) => match self {
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
                _ => UIState::Selecting(self),
            },
            Event::Cancel => match self {
                SelectingState::Tableau { pile_n, .. } => {
                    UIState::Hovering(klondike::PileRef::Tableau(pile_n))
                }
                SelectingState::Talon => UIState::Hovering(klondike::PileRef::Talon),
            },
            Event::Click(Some(card_location)) => {
                let pile_ref = card_location.pile_ref();

                // If the user clicked the card that is already selected,
                // treat that as an auto move
                match game_state {
                    GameStateOption::Playing(play) => match self {
                        SelectingState::Talon if pile_ref == klondike::PileRef::Talon => {
                            match klondike::GameRules::auto_move_card(play.clone(), pile_ref, 1) {
                                Ok(new_state) => {
                                    *game_state = GameStateOption::from(new_state);
                                    if AutoMovingState::can_auto_move(game_state) {
                                        return UIState::AutoMoving(AutoMovingState::new(
                                            klondike::PileRef::Talon,
                                        ));
                                    }
                                }
                                Err(_) => {}
                            }
                            return UIState::Hovering(pile_ref);
                        }
                        SelectingState::Tableau { pile_n, take_n }
                            if pile_ref == klondike::PileRef::Tableau(pile_n)
                                && take_n == card_location.n_from_bottom().unwrap() + 1 =>
                        {
                            match klondike::GameRules::auto_move_card(
                                play.clone(),
                                pile_ref,
                                take_n,
                            ) {
                                Ok(new_state) => {
                                    *game_state = GameStateOption::from(new_state);
                                    if AutoMovingState::can_auto_move(game_state) {
                                        return UIState::AutoMoving(AutoMovingState::new(pile_ref));
                                    }
                                }
                                Err(_) => {}
                            }
                            return UIState::Hovering(pile_ref);
                        }
                        _ => {}
                    },
                    _ => {}
                }

                match pile_ref {
                    // Clicking the stock draws a new card
                    klondike::PileRef::Stock => {
                        match game_state {
                            GameStateOption::Playing(play) => {
                                match klondike::GameRules::draw_stock(play.clone(), 1) {
                                    Ok(new_state) => {
                                        *game_state = GameStateOption::Playing(new_state);
                                        if AutoMovingState::can_auto_move(game_state) {
                                            return UIState::AutoMoving(AutoMovingState::new(
                                                klondike::PileRef::Stock,
                                            ));
                                        }
                                    }
                                    Err(_) => return UIState::Selecting(self),
                                }
                            }
                            _ => {}
                        }
                        UIState::Hovering(klondike::PileRef::Stock)
                    }
                    // Selecting talon
                    klondike::PileRef::Talon => UIState::Selecting(SelectingState::Talon),
                    // Moving to a pile
                    klondike::PileRef::Foundation(_) | klondike::PileRef::Tableau(_) => {
                        let (src, take_n) = match self {
                            SelectingState::Tableau { pile_n, take_n } => {
                                (klondike::PileRef::Tableau(pile_n), take_n)
                            }
                            SelectingState::Talon => (klondike::PileRef::Talon, 1),
                        };
                        match game_state {
                            GameStateOption::Playing(play_state) => {
                                match klondike::GameRules::move_cards(
                                    play_state.clone(),
                                    src,
                                    take_n,
                                    pile_ref,
                                ) {
                                    Ok(result) => {
                                        *game_state = GameStateOption::from(result);
                                        if AutoMovingState::can_auto_move(game_state) {
                                            return UIState::AutoMoving(AutoMovingState::new(
                                                pile_ref,
                                            ));
                                        }
                                        return UIState::Hovering(pile_ref);
                                    }
                                    Err(_) => {}
                                }
                            }
                            _ => {}
                        };
                        // Defer to the logic when hovering
                        UIState::Hovering(pile_ref).on(event, game_state)
                    }
                }
            }
            // All other events are a no-op
            _ => UIState::Selecting(self),
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
    fn on(self, event: Event, game_state: &mut GameStateOption) -> UIState {
        match event {
            Event::Direction { dir, .. } => {
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
            Event::Interact => {
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
                                if AutoMovingState::can_auto_move(game_state) {
                                    return UIState::AutoMoving(AutoMovingState::new(self.dst));
                                }
                                UIState::Hovering(self.dst)
                            }
                            Err(_) => UIState::Hovering(self.src),
                        }
                    }
                    _ => {}
                }
                UIState::Hovering(self.src)
            }
            Event::Goto(i) => UIState::Moving(MovingState {
                src: self.src,
                take_n: self.take_n,
                dst: match i {
                    i @ 3..=6 => klondike::PileRef::Foundation(i as usize - 3),
                    _ => return UIState::Moving(self),
                },
            }),
            Event::Click(card_location) => {
                let selecting_state = match card_location {
                    Some(location) => match self.src {
                        klondike::PileRef::Talon => UIState::Selecting(SelectingState::Talon),
                        klondike::PileRef::Tableau(pile_n) => {
                            UIState::Selecting(SelectingState::Tableau {
                                pile_n,
                                take_n: location.n_from_bottom().unwrap() + 1,
                            })
                        }
                        _ => return UIState::Moving(self),
                    },
                    None => return UIState::Moving(self),
                };

                // Defer to the selecting state logic
                selecting_state.on(event, game_state)
            }
            // All other events are a no-op
            _ => UIState::Moving(self),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AutoMovingState {
    since_last_move: Duration,
    interval: Duration,
    prev_pile_ref: klondike::PileRef,
}

impl AutoMovingState {
    const INITIAL_DELAY: Duration = Duration::from_millis(300);
    const INITIAL_MOVE_INTERVAL: Duration = Duration::from_millis(400);
    const MOVE_INTERVAL_DECREASE: Duration = Duration::from_millis(25);
    const MIN_MOVE_INTERVAL: Duration = Duration::from_millis(50);

    pub fn new(prev_pile_ref: klondike::PileRef) -> Self {
        AutoMovingState {
            since_last_move: Self::INITIAL_MOVE_INTERVAL - Self::INITIAL_DELAY,
            interval: Self::INITIAL_MOVE_INTERVAL,
            prev_pile_ref,
        }
    }

    fn can_auto_move(game_state: &GameStateOption) -> bool {
        match game_state {
            GameStateOption::Playing(play) => {
                let new = klondike::GameRules::auto_move_to_foundation(play.clone());
                match new {
                    klondike::MoveResult::Playing(new) => play != &new,
                    _ => true,
                }
            }
            _ => false,
        }
    }
}

impl State for AutoMovingState {
    fn on(self, event: Event, game_state: &mut GameStateOption) -> UIState {
        match event {
            Event::Tick(dt) => {
                let mut since_last_move = self.since_last_move + dt;
                let mut interval = self.interval;
                // Keep auto moving until the game state doesn't change or it's won,
                // so that slow downs don't cause fewer cards to be dealt
                while since_last_move >= interval {
                    since_last_move = since_last_move - interval;
                    interval = max(
                        interval.saturating_sub(Self::MOVE_INTERVAL_DECREASE),
                        Self::MIN_MOVE_INTERVAL,
                    );
                    match game_state {
                        GameStateOption::Playing(play) => {
                            let new = klondike::GameRules::auto_move_to_foundation(play.clone());
                            match new {
                                klondike::MoveResult::Playing(new) => {
                                    // No change, so we're finished auto moving
                                    if play == &new {
                                        return UIState::Hovering(self.prev_pile_ref);
                                    } else {
                                        *game_state = GameStateOption::Playing(new);
                                    }
                                }
                                klondike::MoveResult::Win(win) => {
                                    *game_state = GameStateOption::Win(win);
                                    return UIState::Hovering(self.prev_pile_ref);
                                }
                            }
                        }
                        // Send the user back to hovering
                        _ => return UIState::Hovering(self.prev_pile_ref),
                    }
                }
                // Only continue auto moving if there's a card to auto move on the next run
                if Self::can_auto_move(game_state) {
                    UIState::AutoMoving(AutoMovingState {
                        since_last_move,
                        interval,
                        ..self
                    })
                } else {
                    UIState::Hovering(self.prev_pile_ref)
                }
            }
            // All other events are a no-op
            _ => UIState::AutoMoving(self),
        }
    }
}
