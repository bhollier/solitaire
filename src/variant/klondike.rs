use std::cmp;

pub use common::{Card, Color, Deck, FrenchSuit, PileRef, Rank, Stack};

use crate::{common, take_n_slice, take_n_vec_mut, take_one_vec_mut, GameState};
pub use crate::{Card as CardTrait, Error, Result, StackFrom};

/// The number of [Tableau](PileRef::Tableau) piles in Klondike Solitaire
pub const NUM_TABLEAU: usize = 7;

/// The number of [Foundation](PileRef::Foundation) piles in Klondike Solitaire
pub const NUM_FOUNDATIONS: usize = FrenchSuit::N;

/// The initial [GameState] for Klondike Solitaire with [common::Card]
pub type InitialGameState = common::InitialGameState<Card, { Card::N }, NUM_TABLEAU>;

/// The mid-game "playing" [GameState] for Klondike Solitaire with [common::Card]
pub type PlayingGameState =
    common::PlayingGameState<Card, { Card::N }, NUM_TABLEAU, NUM_FOUNDATIONS>;

/// The win [GameState] for Klondike Solitaire with [common::Card]
pub type WinGameState = common::WinGameState<Card, { Card::N }, NUM_FOUNDATIONS>;

/// Enum for all possible [GameState]s, for Klondike Solitaire with [Card]
pub type GameStateOption = common::GameStateOption<Card, { Card::N }, NUM_TABLEAU, NUM_FOUNDATIONS>;

/// Enum for the resulting [GameState] after a single deal,
/// for Klondike Solitaire with [common::Card]
pub type DealResult = common::DealResult<Card, { Card::N }, NUM_TABLEAU, NUM_FOUNDATIONS>;

/// Enum for the resulting [GameState] after making a move,
/// for Klondike Solitaire with [common::Card]
pub type MoveResult = common::MoveResult<Card, { Card::N }, NUM_TABLEAU, NUM_FOUNDATIONS>;

/// The Game rules for Klondike Solitaire
pub struct GameRules;

impl GameRules {
    const DEAL_N: usize = NUM_TABLEAU * (NUM_TABLEAU + 1) / 2;

    /// Deals out a single initial card of an [InitialGameState],
    /// returning either an [InitialGameState] or a [PlayingGameState]
    /// if the tableau has been built
    pub fn deal_one(state: InitialGameState) -> DealResult {
        let mut tableau = state.tableau;
        let mut stock = state.stock;

        let drawn = Card::N - stock.len();

        // Figure out the tableau index using triangle numbers
        // Tableau is a "top heavy" triangle so have to invert it
        let card_triangle_num = Self::DEAL_N - drawn;
        // Root is (-1 + √(1 + 8x + 1)) / 2
        let card_triangle_root = (-1f64 + ((1 + (8 * card_triangle_num)) as f64).sqrt()) / 2f64;

        // If the root is integral then this is a new row
        let card_root_trunc = card_triangle_root.trunc();
        let is_new_row = card_triangle_root == card_root_trunc;

        // Calculate the triangle number of the row
        let row_triangle_num = if is_new_row {
            card_triangle_num
        } else {
            (card_root_trunc as usize + 1) * (card_root_trunc as usize + 2) / 2
        };

        // Calculate the tableau index
        let tableau_index = (NUM_TABLEAU - card_triangle_root.ceil() as usize) + row_triangle_num
            - card_triangle_num;

        let mut card = take_one_vec_mut(&mut stock);
        if is_new_row {
            card.face_up = true
        }
        tableau[tableau_index].push(card);

        if Card::N - stock.len() >= Self::DEAL_N {
            DealResult::Complete(PlayingGameState {
                tableau,
                foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
                stock,
                talon: Stack::new(),
            })
        } else {
            DealResult::Dealing(InitialGameState { tableau, stock })
        }
    }

    /// Deals out the initial cards of a [InitialGameState],
    /// returning a [PlayingGameState] with the result of the deal.
    pub fn deal_all(mut state: InitialGameState) -> PlayingGameState {
        match Card::N - state.stock.len() {
            // If the stock is empty, use nested for loops which are simpler and marginally more performant
            0 => {
                let mut new_state = PlayingGameState {
                    tableau: [(); NUM_TABLEAU].map(|_| Stack::new()),
                    foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
                    stock: state.stock,
                    talon: Stack::new(),
                };

                let mut card: Card;
                for i in 0..NUM_TABLEAU {
                    for j in i..NUM_TABLEAU {
                        card = take_one_vec_mut(&mut new_state.stock);
                        new_state.tableau[j].push(card);
                    }
                    new_state.tableau[i].last_mut().unwrap().face_up = true;
                }

                new_state
            }
            // Otherwise use deal_one in a loop
            _ => loop {
                match Self::deal_one(state) {
                    DealResult::Dealing(new_state) => state = new_state,
                    DealResult::Complete(new_state) => return new_state,
                }
            },
        }
    }

    /// Convenience function to create a new [InitialGameState]
    /// and then deal the cards with [deal](Self::deal)
    pub fn new_and_deal() -> PlayingGameState {
        Self::deal_all(InitialGameState::new())
    }

    /// Convenience function to create a new [InitialGameState]
    /// with the given [rand::Rng] and then deal the cards with [deal](Self::deal)
    pub fn new_and_deal_with_rng<RNG: rand::Rng>(rng: &mut RNG) -> PlayingGameState {
        Self::deal_all(InitialGameState::new_with_rng(rng))
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
                // Mark as face down
                for c in &mut new_state.stock {
                    c.face_up = false;
                }
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
                let mut take = take_n_vec_mut(&mut new_state.stock, n);
                // Mark as face up
                for c in &mut take {
                    c.face_up = true;
                }
                // Transfer to the talon
                new_state.talon.append(&mut take);
            }
        }
        Ok(new_state)
    }

    /// If the given sequence of cards is valid to be moved by a player for the given [pile](PileRef),
    /// using the following rules:
    /// - [Foundation](PileRef::Foundation): cards must be of the same [Suit] and in Ace to King order
    /// - [Tableau](PileRef::Tableau): cards must be of alternating [Color](Color) and in King to Ace order
    /// - [Stock](PileRef::Stock): always false
    /// - [Talon](PileRef::Talon): always true
    pub fn valid_seq(p: PileRef, cs: &[Card]) -> bool {
        // Can't take non-face up cards
        for c in cs {
            if !c.face_up {
                return false;
            }
        }
        match p {
            PileRef::Tableau(_) => {
                let mut prev_card = &cs[0];
                for card in &cs[1..cs.len()] {
                    if card.suit.color() == prev_card.suit.color() {
                        return false;
                    }
                    if prev_card.rank.next() != Some(&card.rank) {
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
                    if prev_card.rank.prev() != Some(&card.rank) {
                        return false;
                    }
                    prev_card = card;
                }
                return true;
            }
            PileRef::Stock => false,
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
        let mut new_src_stack: Stack;
        let new_dst_stack: Stack;
        {
            let src_stack = state.get_stack(src).ok_or(Error::InvalidInput {
                field: "src",
                reason: "pile does not exist",
            })?;

            if take_n > src_stack.len() {
                return Err(Error::InvalidInput {
                    field: "take_n",
                    reason: "not enough cards in src pile",
                });
            }

            let (rest, take) = take_n_slice(src_stack.as_slice(), take_n);
            if !Self::valid_seq(src, take) {
                return Err(Error::InvalidMove {
                    reason: "src sequence is invalid",
                });
            }

            new_src_stack = rest.iter().cloned().collect();
            match new_src_stack.last_mut() {
                Some(c) => c.face_up = true,
                _ => {}
            }

            let dst_stack = state.get_stack(dst).ok_or(Error::InvalidInput {
                field: "dst",
                reason: "pile does not exist",
            })?;

            new_dst_stack = dst_stack.iter().chain(take.iter()).cloned().collect();

            if dst_stack.is_empty() {
                match dst {
                    PileRef::Tableau(_) => {
                        if take[0].rank != Rank::King {
                            return Err(Error::InvalidMove {
                                reason: "can only move a King to a space",
                            });
                        }
                    }
                    PileRef::Foundation(_) => {
                        if take[0].rank != Rank::Ace {
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
                    if foundation.len() < Rank::N {
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

    /// Attempts to move `take_n` [Card]s from the stack at `src` and place them anywhere that
    /// they can be moved.
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
    pub fn auto_move_card(
        state: PlayingGameState,
        src: PileRef,
        take_n: usize,
    ) -> Result<MoveResult> {
        match src {
            // No op
            PileRef::Foundation(_) => return Ok(MoveResult::Playing(state)),
            _ => {}
        }

        let try_move_cards = |dst| -> Result<Option<MoveResult>> {
            match Self::move_cards(state.clone(), src, take_n, dst) {
                Ok(result) => Ok(Some(result)),
                // Return if there's a legitimate error (invalid input)
                Err(err @ Error::InvalidInput { .. }) => Err(err),
                _ => Ok(None),
            }
        };

        // First, try placing in the foundations
        // (but only if take_n is 1)
        if take_n == 1 {
            for dst in (0..NUM_FOUNDATIONS).map(|i| PileRef::Foundation(i)) {
                match try_move_cards(dst)? {
                    Some(result) => return Ok(result),
                    _ => {}
                }
            }
        }

        // Try placing in the tableau
        for dst in (0..NUM_TABLEAU).map(|i| PileRef::Tableau(i)) {
            // Skip if this is the source
            if src == dst {
                continue;
            }
            match try_move_cards(dst)? {
                Some(result) => return Ok(result),
                _ => {}
            }
        }

        // No where to move the card, so no-op
        Ok(MoveResult::Playing(state))
    }

    /// If the provided card is safe to be moved to one of the provided foundations, defined as
    /// "both foundations for the opposite coloured suit have reached the card's rank - 1".
    /// It is assumed that it is always safe to move an Ace or a Two.
    /// Returns false if it's not possible to move the card to a foundation at all.
    pub fn is_safe_to_move_to_foundation(
        card_to_move: &Card,
        foundations: &[Stack; NUM_FOUNDATIONS],
    ) -> bool {
        // Get the card's previous rank
        // (using next() since it's ordered by how cards appear in the tableau)
        let prev_rank = match card_to_move.rank.next() {
            Some(rank) => rank,
            // If there's no previous rank it must be an Ace, which is always safe to move
            None => return true,
        };
        // Find the correct foundation to move the card to
        let card_foundation = foundations
            .iter()
            .find(|foundation| match foundation.last() {
                Some(c) => c.suit == card_to_move.suit && c.rank == *prev_rank,
                None => false,
            });
        // Return false if the card can't even be moved to a foundation
        if card_foundation.is_none() {
            return false;
        }
        // A two is always safe (if there's somewhere to put it)
        if card_to_move.rank == Rank::Two {
            return true;
        };
        // Make sure the foundations for the opposite colour are safe
        let mut foundation_matches: usize = 0;
        for foundation in foundations {
            let top = match foundation.last() {
                Some(card) => card,
                None => continue,
            };
            if top.suit.color() == card_to_move.suit.color().opposite() {
                if top.rank <= *prev_rank {
                    foundation_matches += 1;
                } else {
                    // The card can't be safe, exit
                    return false;
                }
            }
        }
        foundation_matches == 2
    }

    /// Attempts to move any card from the tableau or talon and place them in the foundations.
    /// Unlike [GameRules::auto_move_card], a card will only be moved to the foundation if it's
    /// considered "safe" to do so; see [GameRules::is_safe_to_move_to_foundation] for rules on
    /// what cards can be moved
    pub fn auto_move_to_foundation(state: PlayingGameState) -> MoveResult {
        // Consider the talon first, then the tableaus left to right
        let piles = [PileRef::Talon]
            .iter()
            .cloned()
            .chain((0..NUM_TABLEAU).map(|i| PileRef::Tableau(i)));

        for pile_ref in piles {
            // Get the top card, skipping empty stacks
            let card = match state.get_stack(pile_ref).unwrap().last() {
                Some(c) => c,
                None => continue,
            };
            // is_safe_to_move_to_foundation validates both that the card is safe to move
            // AND that it can be moved, so if it returns true we know to move it
            if Self::is_safe_to_move_to_foundation(card, &state.foundations) {
                return Self::auto_move_card(state, pile_ref, 1).unwrap();
            }
        }

        // No matches, so just return the state back unchanged
        MoveResult::Playing(state)
    }
}
