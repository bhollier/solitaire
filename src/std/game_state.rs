use crate as solitaire;
use crate::{Card, GameState, Stack, StackFrom};

/// "Standard" solitaire piles
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PileRef {
    /// The "tableau" of [Stack]s where cards are moved around
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

/// Struct for the initial [GameState] with just the [Stock](PileRef::Stock) pile
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialGameState<'d, C: Card<N>, const N: usize> {
    /// The stock, see [Stock](PileRef::Stock)
    pub stock: Stack<'d, C>,
}

impl<'d, C: Card<N>, const N: usize> GameState<'d, C, N, PileRef> for InitialGameState<'d, C, N> {
    fn get_stack(&self, p: PileRef) -> Option<&Stack<'d, C>> {
        match p {
            PileRef::Stock => Some(&self.stock),
            _ => None,
        }
    }

    fn get_stack_mut(&mut self, p: PileRef) -> Option<&mut Stack<'d, C>> {
        match p {
            PileRef::Stock => Some(&mut self.stock),
            _ => None,
        }
    }
}

impl<'d, C: Card<N>, const N: usize> InitialGameState<'d, C, N> {
    pub fn new(deck: &'d [C]) -> InitialGameState<'d, C, N> {
        InitialGameState {
            stock: Stack::from_slice(deck),
        }
    }
}

/// Struct for a mid-game "playing" [GameState] with four [piles](PileRef) of generic [Card]s
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayingGameState<'d, C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> {
    /// The tableau, see [Tableau](PileRef::Tableau)
    pub tableau: [Stack<'d, C>; NT],

    /// The foundations, see [Foundation](PileRef::Foundation)
    pub foundations: [Stack<'d, C>; NF],

    /// The stock, see [Stock](PileRef::Stock)
    pub stock: Stack<'d, C>,

    /// The talon, see [Talon](PileRef::Talon)
    pub talon: Stack<'d, C>,
}

impl<'d, C: Card<NC>, const NC: usize, const NT: usize, const NF: usize>
    GameState<'d, C, NC, PileRef> for PlayingGameState<'d, C, NC, NT, NF>
{
    fn get_stack(&self, p: PileRef) -> Option<&Stack<'d, C>> {
        match p {
            PileRef::Tableau(n) => self.tableau.get(n),
            PileRef::Foundation(n) => self.foundations.get(n),
            PileRef::Stock => Some(&self.stock),
            PileRef::Talon => Some(&self.talon),
        }
    }

    fn get_stack_mut(&mut self, p: PileRef) -> Option<&mut Stack<'d, C>> {
        match p {
            PileRef::Tableau(n) => self.tableau.get_mut(n),
            PileRef::Foundation(n) => self.foundations.get_mut(n),
            PileRef::Stock => Some(&mut self.stock),
            PileRef::Talon => Some(&mut self.talon),
        }
    }
}

/// Struct for a win [GameState] with just the [Foundation](PileRef::Foundation) piles
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WinGameState<'d, C: Card<NC>, const NC: usize, const NF: usize> {
    /// The foundations, see [Foundation](PileRef::Foundation)
    pub foundations: [Stack<'d, C>; NF],
}

impl<'d, C: Card<NC>, const NC: usize, const NF: usize> GameState<'d, C, NC, PileRef>
    for WinGameState<'d, C, NC, NF>
{
    fn get_stack(&self, p: PileRef) -> Option<&Stack<'d, C>> {
        match p {
            PileRef::Foundation(n) => self.foundations.get(n),
            _ => None,
        }
    }

    fn get_stack_mut(&mut self, p: PileRef) -> Option<&mut Stack<'d, C>> {
        match p {
            PileRef::Foundation(n) => self.foundations.get_mut(n),
            _ => None,
        }
    }
}

/// Enum for the resulting [GameState] after making a move,
/// either [Playing] (game not finished) or [Win]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MoveResult<'d, C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> {
    Playing(PlayingGameState<'d, C, NC, NT, NF>),
    Win(WinGameState<'d, C, NC, NF>),
}
