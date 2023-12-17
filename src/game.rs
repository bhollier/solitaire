use crate::*;
use ::std;
use thiserror;

/// A reference to a "Pile" of [Card]s, e.g. the stock, a foundation
pub trait PileRef: Eq {}

/// Trait for the state of a Solitaire game
pub trait GameState<'a, C: Card<N>, const N: usize, P: PileRef>: Sized + Clone {
    /// Creates a new game, using the given [Deck]
    fn new(deck: &'a Deck<C, N>) -> Self;

    /// Retrieve a reference to the [Stack] at the given [PileRef]
    fn get_stack(&self, p: &P) -> Option<&Stack<'a, C>>;
    /// Retrieve a mutable reference to the [Stack] at the given [PileRef]
    fn get_stack_mut(&mut self, p: &P) -> Option<&mut Stack<'a, C>>;
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
