use crate as solitaire;
use crate::std;
use crate::{take_n_slice, GameState as GameStateTrait};
use crate::{Card, Error, Result, Stack};
use ::std::cmp::Ordering;

pub const NUM_TABLEAU: usize = 7;
pub const NUM_FOUNDATIONS: usize = 4;

/// "Standard" (Klondike) solitaire piles
#[derive(Eq, PartialEq)]
pub enum PileRef {
    /// The "tableau" of 7 [Stack]s where cards moved around
    Tableau(usize),
    /// The "foundation" where cards of each suit are accumulated
    Foundation(usize),
    /// The "stock" (or "hand") where the [Tableau](PileRef::Tableau)
    /// is initially created from and additional cards are taken from
    Stock,
    /// The "talon" (or "waste") where cards from the [Stock](PileRef::Stock)
    /// with no place in the [Tableau](PileRef::Tableau) or [Foundation](PileRef::Foundation)
    /// are added to
    Talon,
}

impl solitaire::PileRef for PileRef {}

/// Struct for a [GameState](solitaire::GameState) containing the four piles in Klondike
#[derive(Clone)]
pub struct GenericGameState<'a, C: Card<N>, const N: usize> {
    /// The tableau, see [Tableau](PileRef::Tableau)
    pub tableau: [Stack<'a, C>; NUM_TABLEAU],
    /// The foundations, see [Foundation](PileRef::Foundation)
    pub foundations: [Stack<'a, C>; NUM_FOUNDATIONS],
    /// The stock, see [Stock](PileRef::Stock)
    pub stock: Stack<'a, C>,
    /// The talon, see [Talon](PileRef::Talon)
    pub talon: Stack<'a, C>,
}

impl<'a, C: Card<N>, const N: usize> solitaire::GameState<'a, C, N, PileRef>
    for GenericGameState<'a, C, N>
{
    fn new(deck: &'a solitaire::Deck<C, N>) -> GenericGameState<'a, C, N> {
        GenericGameState {
            tableau: [(); NUM_TABLEAU].map(|_| Stack::new()),
            foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
            stock: deck.iter().collect(),
            talon: Stack::new(),
        }
    }

    fn get_stack(&self, p: &PileRef) -> Option<&Stack<'a, C>> {
        match p {
            PileRef::Tableau(n) => self.tableau.get(*n),
            PileRef::Foundation(n) => self.foundations.get(*n),
            PileRef::Stock => Some(&self.stock),
            PileRef::Talon => Some(&self.talon),
        }
    }

    fn get_stack_mut(&mut self, p: &PileRef) -> Option<&mut Stack<'a, C>> {
        match p {
            PileRef::Tableau(n) => self.tableau.get_mut(*n),
            PileRef::Foundation(n) => self.foundations.get_mut(*n),
            PileRef::Stock => Some(&mut self.stock),
            PileRef::Talon => Some(&mut self.talon),
        }
    }
}

/// A [GameState](solitaire::GameState) for a "standard" (Klondike) game of Solitaire with [std::Card]
pub type GameState<'a> = GenericGameState<'a, std::Card, { std::Card::N }>;

/// The Game rules for a "standard" (Klondike) game of Solitaire.
/// Only works with a [std::GameState](GameState)
pub struct GameRules;

impl GameRules {
    /// If the given sequence of cards is valid to be moved by a player for the given [pile](PileRef).
    /// For example, for a [Foundation](PileRef::Foundation) cards must be of the same [Suit] and in Ace to King order,
    /// but for a [Tableau](PileRef::Tableau) cards must be of alternating [Color](std::Color) and in King to Ace order.
    pub fn valid_seq(p: &PileRef, cs: &[&std::Card]) -> bool {
        match p {
            PileRef::Tableau(_) => {
                let mut prev_card = cs[0];
                for card in cs {
                    if card.suit.color() == prev_card.suit.color() {
                        return false;
                    }
                    if card.rank.cmp(&prev_card.rank) != Ordering::Greater {
                        return false;
                    }
                    prev_card = card;
                }
                return true;
            }
            PileRef::Foundation(_) => {
                let mut prev_card = cs[0];
                for card in cs {
                    if card.suit.color() != prev_card.suit.color() {
                        return false;
                    }
                    if card.rank.cmp(&prev_card.rank) != Ordering::Less {
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

    /// Immutable version of [GameRules::deal_mut] which operates on a clone of the given [crate::GameState]
    pub fn deal(state: GameState) -> Result<GameState> {
        let mut new_state = state;
        Self::deal_mut(&mut new_state).map(|_| new_state)
    }

    /// Deals out the initial cards at the start of the game.
    /// If [Err] is returned then `state` should not be modified
    pub fn deal_mut(state: &mut GameState) -> Result<()> {
        if state.tableau.iter().any(|s| !s.is_empty()) {
            return Err(Error::InvalidState);
        }

        Ok({
            let mut card: &std::Card;
            for i in 0..NUM_TABLEAU {
                for j in i..NUM_TABLEAU {
                    card = solitaire::take_one_vec_mut(&mut state.stock);
                    state.tableau[j].push(card);
                }
            }
        })
    }

    /// Immutable version of [GameRules::move_card] which operates on a clone of the given [crate::GameState]
    pub fn move_card(
        state: GameState,
        src: PileRef,
        take: usize,
        dst: PileRef,
    ) -> Result<GameState> {
        let mut new_state = state;
        Self::move_card_mut(&mut new_state, src, take, dst).map(|_| new_state)
    }

    /// Attempts to move `take` [Card]s from the stack at `src` and place them onto `dst`.
    /// If [Err] is returned then `state` is not modified
    pub fn move_card_mut(
        state: &mut GameState,
        src: PileRef,
        take: usize,
        dst: PileRef,
    ) -> Result<()> {
        if take == 0 {
            return Err(Error::InvalidInput { field: "take" });
        }

        let src_pile = state
            .get_stack(&src)
            .ok_or(Error::InvalidInput { field: "src" })?;

        if take > src_pile.len() {
            return Err(Error::InvalidInput { field: "take" });
        }

        let dst_pile = state
            .get_stack(&dst)
            .ok_or(Error::InvalidInput { field: "dst" })?;

        match src {
            PileRef::Tableau(_) | PileRef::Foundation(_) => {}
            PileRef::Stock => {
                if take != 1 {
                    return Err(Error::InvalidInput { field: "take" });
                }
                if dst != PileRef::Talon {
                    return Err(Error::InvalidMove);
                }
            }
            PileRef::Talon => {
                if take != 1 {
                    return Err(Error::InvalidInput { field: "take" });
                }
            }
        }

        if !Self::valid_seq(&src, take_n_slice(src_pile.as_slice(), take).0) {
            return Err(Error::InvalidMove);
        }

        // Source == destination is a no-op
        if src == dst {
            return Ok(());
        }

        todo!()
    }
}
