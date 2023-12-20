use ::std::cmp;

use crate::{
    std, std::PileRef, take_n_slice, take_n_vec_mut, take_one_vec_mut, Error, GameState, Result,
    Stack,
};

/// The number of [Tableau](PileRef::Tableau) piles in Klondike Solitaire
pub const NUM_TABLEAU: usize = 7;

/// The number of [Foundation](PileRef::Foundation) piles in Klondike Solitaire
pub const NUM_FOUNDATIONS: usize = std::FrenchSuit::N;

/// The initial [GameState] for Klondike Solitaire with [std::Card]
pub type InitialGameState = std::InitialGameState<std::Card, { std::Card::N }>;

/// The mid-game "playing" [GameState] for Klondike Solitaire with [std::Card]
pub type PlayingGameState =
    std::PlayingGameState<std::Card, { std::Card::N }, NUM_TABLEAU, NUM_FOUNDATIONS>;

/// The win [GameState] for Klondike Solitaire with [std::Card]
pub type WinGameState = std::WinGameState<std::Card, { std::Card::N }, NUM_FOUNDATIONS>;

/// Enum for all possible [GameState]s, for Klondike Solitaire with [std::Card]
pub type GameStateOption =
    std::GameStateOption<std::Card, { std::Card::N }, NUM_TABLEAU, NUM_FOUNDATIONS>;

/// Enum for the resulting [GameState] after making a move,
/// for Klondike Solitaire with [std::Card]
pub type MoveResult = std::MoveResult<std::Card, { std::Card::N }, NUM_TABLEAU, NUM_FOUNDATIONS>;

/// The Game rules for Klondike Solitaire
pub struct GameRules;

impl GameRules {
    /// Deals out the initial cards of a [InitialGameState],
    /// returning a [PlayingGameState] with the result of the deal.
    pub fn deal(state: InitialGameState) -> PlayingGameState {
        let mut new_state = PlayingGameState {
            tableau: [(); NUM_TABLEAU].map(|_| Stack::new()),
            foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
            stock: state.stock,
            talon: Stack::new(),
        };

        let mut card: std::Card;
        for i in 0..NUM_TABLEAU {
            for j in i..NUM_TABLEAU {
                card = take_one_vec_mut(&mut new_state.stock);
                new_state.tableau[j].push(card);
            }
        }

        new_state
    }

    /// Convenience function to create a new [InitialGameState]
    /// and then deal the cards with [deal](Self::deal)
    pub fn new_and_deal() -> PlayingGameState {
        Self::deal(InitialGameState::new())
    }

    /// Draws `n` cards from the [Stock](PileRef::Stock) onto the [Talon](PileRef::Talon).
    /// If the stock is empty, the talon is turned over and used as the stock.
    pub fn draw_stock(state: PlayingGameState, n: usize) -> Result<PlayingGameState> {
        let mut new_state = state;
        match new_state.stock.len() {
            // Empty stock
            0 => {
                // Transfer all cards from the talon to the stock
                (new_state.stock, new_state.talon) = (new_state.talon, new_state.stock);
                new_state.stock.reverse();
            }
            len => {
                // Edge case where stock has some cards but not enough.
                // In theory for standard Klondike n should always be either 1 or 3
                // and the stock after draw should be 24,
                // so this should never happen (24 is divisible by both),
                // but handle it anyway by taking as much from the stock as is available
                let n = cmp::min(n, len);
                // Take the cards from the stock
                let take = take_n_vec_mut(&mut new_state.stock, n);
                // Transfer to the talon
                take.iter().cloned().for_each(|c| new_state.talon.push(c));
            }
        }
        Ok(new_state)
    }

    /// If the given sequence of cards is valid to be moved by a player for the given [pile](PileRef),
    /// using the following rules:
    /// - [Foundation](PileRef::Foundation): cards must be of the same [Suit] and in Ace to King order
    /// - [Tableau](PileRef::Tableau): cards must be of alternating [Color](std::Color) and in King to Ace order
    /// - [Stock](PileRef::Stock): always true
    /// - [Talon](PileRef::Talon): always true
    pub fn valid_seq(p: PileRef, cs: &[std::Card]) -> bool {
        match p {
            PileRef::Tableau(_) => {
                let mut prev_card = &cs[0];
                for card in &cs[1..cs.len()] {
                    if card.suit.color() == prev_card.suit.color() {
                        return false;
                    }
                    if card.rank.cmp(&prev_card.rank) != cmp::Ordering::Greater {
                        return false;
                    }
                    prev_card = card;
                }
                return true;
            }
            PileRef::Foundation(_) => {
                let mut prev_card = &cs[0];
                for card in &cs[1..cs.len()] {
                    if card.suit != prev_card.suit {
                        return false;
                    }
                    if card.rank.cmp(&prev_card.rank) != cmp::Ordering::Less {
                        return false;
                    }
                    prev_card = card;
                }
                return true;
            }
            PileRef::Stock => true,
            PileRef::Talon => true,
        }
    }

    /// Attempts to move `take_n` [Card]s from the stack at `src` and place them onto `dst`,
    /// returning a copy of `state` with the result of the move.
    /// See [GameRules::valid_seq] for rules on what card sequences are valid to move.
    ///
    /// # Arguments
    ///
    /// - `src`: The [PileRef] to move the cards from. Must be one of:
    ///   - [Tableau](PileRef::Tableau)
    ///   - [Foundation](PileRef::Foundation)
    ///   - [Talon](PileRef::Talon)
    /// - `take_n`: The total number of cards to take from `src`.
    ///   Cannot be `0`, and if `src` is [Talon](PileRef::Talon) then must be `1`.
    /// - `dst`: The [PileRef] to move the cards to. Must be one of:
    ///   - [Tableau](PileRef::Tableau)
    ///   - [Foundation](PileRef::Foundation)
    pub fn move_cards(
        state: PlayingGameState,
        src: PileRef,
        take_n: usize,
        dst: PileRef,
    ) -> Result<MoveResult> {
        if take_n == 0 {
            return Err(Error::InvalidInput {
                field: "take_n",
                reason: "cannot take 0 cards",
            });
        }

        // Validate src
        match src {
            PileRef::Tableau(_) | PileRef::Foundation(_) => {}
            PileRef::Stock => {
                return Err(Error::InvalidInput {
                    field: "src",
                    reason: "cannot move cards from stock",
                })
            }
            PileRef::Talon => {
                if take_n != 1 {
                    return Err(Error::InvalidInput {
                        field: "take_n",
                        reason: "cannot move more than 1 card from talon",
                    });
                }
            }
        }

        // Validate dst
        match dst {
            PileRef::Tableau(_) => {}
            PileRef::Foundation(_) => {
                if take_n != 1 {
                    return Err(Error::InvalidInput {
                        field: "take_n",
                        reason: "cannot move more than 1 card to foundation",
                    });
                }
            }
            PileRef::Stock => {
                return Err(Error::InvalidInput {
                    field: "dst",
                    reason: "cannot move cards to stock",
                })
            }
            PileRef::Talon => {
                return Err(Error::InvalidInput {
                    field: "dst",
                    reason: "cannot move cards to talon",
                })
            }
        }

        // Source == destination is a no-op
        if src == dst {
            return Ok(MoveResult::Playing(state));
        }

        // Create stacks for the new state of src and dst
        let new_src_stack: std::Stack;
        let new_dst_stack: std::Stack;
        {
            let src_pile = state.get_stack(src).ok_or(Error::InvalidInput {
                field: "src",
                reason: "pile does not exist",
            })?;

            if take_n > src_pile.len() {
                return Err(Error::InvalidInput {
                    field: "take_n",
                    reason: "not enough cards in src pile",
                });
            }

            let (rest, take) = take_n_slice(src_pile.as_slice(), take_n);
            if !Self::valid_seq(src, take) {
                return Err(Error::InvalidMove {
                    reason: "src sequence is invalid",
                });
            }

            new_src_stack = rest.iter().cloned().collect();

            let dst_pile = state.get_stack(dst).ok_or(Error::InvalidInput {
                field: "dst",
                reason: "pile does not exist",
            })?;

            new_dst_stack = dst_pile.iter().chain(take.iter()).cloned().collect();

            if dst_pile.is_empty() {
                match dst {
                    PileRef::Tableau(_) => {
                        if take[0].rank != std::Rank::King {
                            return Err(Error::InvalidMove {
                                reason: "can only move a King to a space",
                            });
                        }
                    }
                    PileRef::Foundation(_) => {
                        if take[0].rank != std::Rank::Ace {
                            return Err(Error::InvalidMove {
                                reason: "dst sequence is invalid",
                            });
                        }
                    }
                    PileRef::Stock => {}
                    PileRef::Talon => {}
                }
            } else {
                if !Self::valid_seq(
                    dst,
                    &new_dst_stack
                        [new_dst_stack.len() - take_n - 1..new_dst_stack.len() - take_n + 1],
                ) {
                    return Err(Error::InvalidMove {
                        reason: "dst sequence is invalid",
                    });
                }
            }
        }

        let mut new_state = state;
        *new_state.get_stack_mut(src).unwrap() = new_src_stack;
        *new_state.get_stack_mut(dst).unwrap() = new_dst_stack;

        match dst {
            // If dst is a foundation, check for a win condition
            PileRef::Foundation(_) => {
                for foundation in &new_state.foundations {
                    // Foundation doesn't have enough cards
                    if foundation.len() < std::Rank::N {
                        // So still playing
                        return Ok(MoveResult::Playing(new_state));
                    }
                }
                // All the foundations have the full suit, so return win state
                Ok(MoveResult::Win(WinGameState {
                    foundations: new_state.foundations,
                }))
            }
            _ => Ok(MoveResult::Playing(new_state)),
        }
    }
}
