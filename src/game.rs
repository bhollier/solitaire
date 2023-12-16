use crate::*;
use ::std;
use thiserror;

/// A reference to a "Pile" of [Card]s, e.g. the stock, a foundation
pub trait PileRef: Eq {}

/// Trait for the state of a Solitaire game
pub trait GameState<'a, S: Suit, R: Rank, P: PileRef>: Sized + Clone {
    /// Creates a new game, using the given [Deck]
    fn new(deck: &'a Deck<S, R>) -> Self;

    /// Retrieve a reference to the [Stack] at the given [PileRef]
    fn get_stack(&self, p: &P) -> Option<&Stack<'a, S, R>>;
    /// Retrieve a mutable reference to the [Stack] at the given [PileRef]
    fn get_stack_mut(&mut self, p: &P) -> Option<&mut Stack<'a, S, R>>;
}

/// Enum of all the possible errors that [GameRules] returns in a [Result]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The given GameState was invalid")]
    InvalidState,

    #[error("The given input {field:?} was invalid")]
    InvalidInput { field: &'static str },

    #[error("Requested move was invalid")]
    InvalidMove,

    #[error("An unknown error occurred")]
    Unknown,
}

/// [`std::result::Result`] type for [Error]
pub type Result<T> = std::result::Result<T, Error>;

/// Trait for a Solitaire variant's rules
pub trait GameRules<'a, GS: GameState<'a, S, R, P>, S: Suit, R: Rank, P: PileRef>: Sized {
    /// Immutable version of [GameRules::deal_mut] which operates on a clone of the given [GameState]
    fn deal(state: GS) -> Result<GS>;
    /// Deals out the initial cards at the start of the game. 
    /// If [Err] is returned then `state` should not be modified
    fn deal_mut(state: &mut GS) -> Result<()>;

    /// Immutable version of [GameRules::move_card] which operates on a clone of the given [GameState]
    fn move_card(state: GS, src: P, take: usize, dst: P) -> Result<GS>;
    /// Attempts to move `take` [Card]s from the stack at `src` and place them onto `dst`. 
    /// If [Err] is returned then `state` is not modified
    fn move_card_mut(state: &mut GS, src: P, take: usize, dst: P) -> Result<()>;
}
