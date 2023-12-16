use crate as solitaire;
use crate::Error::InvalidInput;
use crate::{take_n_slice, GameState as GameStateTrait};
use crate::{Card, Error, Rank, Result, Stack, Suit};

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

/// A [GameState](solitaire::GameState) for a "standard" (Klondike) game of Solitaire
#[derive(Clone)]
pub struct GameState<'a, S: Suit, R: Rank> {
    /// The tableau, see [Tableau](PileRef::Tableau)
    pub tableau: [Stack<'a, S, R>; NUM_TABLEAU],
    /// The foundations, see [Foundation](PileRef::Foundation)
    pub foundations: [Stack<'a, S, R>; NUM_FOUNDATIONS],
    /// The stock, see [Stock](PileRef::Stock)
    pub stock: Stack<'a, S, R>,
    /// The talon, see [Talon](PileRef::Talon)
    pub talon: Stack<'a, S, R>,
}

impl<'a, S: Suit, R: Rank> solitaire::GameState<'a, S, R, PileRef> for GameState<'a, S, R> {
    fn new(deck: &'a solitaire::Deck<S, R>) -> GameState<'a, S, R> {
        GameState {
            tableau: [(); NUM_TABLEAU].map(|_| Stack::new()),
            foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
            stock: deck.iter().collect(),
            talon: Stack::new(),
        }
    }

    fn get_stack(&self, p: &PileRef) -> Option<&Stack<'a, S, R>> {
        match p {
            PileRef::Tableau(n) => self.tableau.get(n),
            PileRef::Foundation(n) => self.foundations.get(n),
            PileRef::Stock => Some(&self.stock),
            PileRef::Talon => Some(&self.talon),
        }
    }

    fn get_stack_mut(&mut self, p: &PileRef) -> Option<&mut Stack<'a, S, R>> {
        match p {
            PileRef::Tableau(n) => self.tableau.get_mut(n),
            PileRef::Foundation(n) => self.foundations.get_mut(n),
            PileRef::Stock => Some(&mut self.stock),
            PileRef::Talon => Some(&mut self.talon),
        }
    }
}

/// The [GameRules](solitaire::GameRules) for a "standard" (Klondike) game of Solitaire.
/// Only works with a [std::GameState](GameState)
pub struct GameRules;

impl GameRules {
    /// If the given sequence of cards is valid to be moved by a player for the given [pile](PileRef).
    /// For example, for a [Foundation](PileRef::Foundation) cards must be of the same [Suit] and in Ace to King order,
    /// but for a [Tableau](PileRef::Tableau) cards must be of alternating [Color] and in King to Ace order.
    fn valid_seq<S: Suit, R: Rank>(p: &PileRef, cs: &[&Card<S, R>]) -> bool {}
}

impl<'a, S: Suit, R: Rank> solitaire::GameRules<'a, GameState<'a, S, R>, S, R, PileRef>
    for GameRules
{
    fn deal(state: GameState<'a, S, R>) -> Result<GameState<'a, S, R>> {
        let mut new_state = state;
        Self::deal_mut(&mut new_state).map(|_| new_state)
    }

    fn deal_mut<'b>(state: &'b mut GameState<'a, S, R>) -> Result<()> {
        if state.tableau.iter().any(|s| !s.is_empty()) {
            return Err(Error::InvalidState);
        }

        Ok({
            let mut card: &Card<S, R>;
            for i in 0..NUM_TABLEAU {
                for j in i..NUM_TABLEAU {
                    card = solitaire::take_one_vec_mut(&mut state.stock);
                    state.tableau[j].push(card);
                }
            }
        })
    }

    fn move_card(
        state: GameState<'a, S, R>,
        src: PileRef,
        take: usize,
        dst: PileRef,
    ) -> Result<GameState<'a, S, R>> {
        let mut new_state = state;
        Self::move_card_mut(&mut new_state, src, take, dst).map(|_| new_state)
    }

    fn move_card_mut(
        state: &mut GameState<'a, S, R>,
        src: PileRef,
        take: usize,
        dst: PileRef,
    ) -> Result<()> {
        // Source == destination is a no-op
        if src == dst {
            return Ok(());
        }

        let src_pile = state
            .get_stack_mut(&src)
            .ok_or(InvalidInput { field: "src" })?;

        if take > src_pile.len() {
            return Err(InvalidInput { field: "take" });
        }

        let dst_pile = state
            .get_stack_mut(&dst)
            .ok_or(InvalidInput { field: "src" })?;

        if !Self::valid_seq(&src, take_n_slice(src_pile.as_slice(), take).0) {}

        todo!()
    }
}
