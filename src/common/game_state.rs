use crate as solitaire;
use crate::{shuffle, shuffle_with_rng, Card, Deck, GameState, Stack, StackFrom};

/// "Standard" solitaire piles
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
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
pub struct InitialGameState<C: Card<N>, const N: usize> {
    /// The stock, see [Stock](PileRef::Stock)
    pub stock: Stack<C>,
}

impl<C: Card<N>, const N: usize> GameState<C, N, PileRef> for InitialGameState<C, N> {
    fn get_stack(&self, p: PileRef) -> Option<&Stack<C>> {
        match p {
            PileRef::Stock => Some(&self.stock),
            _ => None,
        }
    }

    fn get_stack_mut(&mut self, p: PileRef) -> Option<&mut Stack<C>> {
        match p {
            PileRef::Stock => Some(&mut self.stock),
            _ => None,
        }
    }
}

impl<C: Card<N>, const N: usize> InitialGameState<C, N> {
    pub fn new() -> InitialGameState<C, N> {
        let mut d = C::new_deck();
        shuffle(&mut d);
        InitialGameState::from(d)
    }

    pub fn new_with_rng<RNG: rand::Rng>(rng: &mut RNG) -> InitialGameState<C, N> {
        let mut d = C::new_deck();
        shuffle_with_rng(&mut d, rng);
        InitialGameState::from(d)
    }
}

impl<C: Card<N>, const N: usize> From<Deck<C, N>> for InitialGameState<C, N> {
    fn from(d: Deck<C, N>) -> Self {
        InitialGameState {
            stock: Stack::from_slice(&d),
        }
    }
}

/// Struct for a mid-game "playing" [GameState] with four [piles](PileRef) of generic [Card]s
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayingGameState<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> {
    /// The tableau, see [Tableau](PileRef::Tableau)
    pub tableau: [Stack<C>; NT],

    /// The foundations, see [Foundation](PileRef::Foundation)
    pub foundations: [Stack<C>; NF],

    /// The stock, see [Stock](PileRef::Stock)
    pub stock: Stack<C>,

    /// The talon, see [Talon](PileRef::Talon)
    pub talon: Stack<C>,
}

impl<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> GameState<C, NC, PileRef>
    for PlayingGameState<C, NC, NT, NF>
{
    fn get_stack(&self, p: PileRef) -> Option<&Stack<C>> {
        match p {
            PileRef::Tableau(n) => self.tableau.get(n),
            PileRef::Foundation(n) => self.foundations.get(n),
            PileRef::Stock => Some(&self.stock),
            PileRef::Talon => Some(&self.talon),
        }
    }

    fn get_stack_mut(&mut self, p: PileRef) -> Option<&mut Stack<C>> {
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
pub struct WinGameState<C: Card<NC>, const NC: usize, const NF: usize> {
    /// The foundations, see [Foundation](PileRef::Foundation)
    pub foundations: [Stack<C>; NF],
}

impl<'d, C: Card<NC>, const NC: usize, const NF: usize> GameState<C, NC, PileRef>
    for WinGameState<C, NC, NF>
{
    fn get_stack(&self, p: PileRef) -> Option<&Stack<C>> {
        match p {
            PileRef::Foundation(n) => self.foundations.get(n),
            _ => None,
        }
    }

    fn get_stack_mut(&mut self, p: PileRef) -> Option<&mut Stack<C>> {
        match p {
            PileRef::Foundation(n) => self.foundations.get_mut(n),
            _ => None,
        }
    }
}

/// Enum for all possible [GameState]s
#[derive(Clone, Eq, PartialEq)]
pub enum GameStateOption<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> {
    Initial(InitialGameState<C, NC>),
    Playing(PlayingGameState<C, NC, NT, NF>),
    Win(WinGameState<C, NC, NF>),
}

impl<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> From<MoveResult<C, NC, NT, NF>>
    for GameStateOption<C, NC, NT, NF>
{
    fn from(value: MoveResult<C, NC, NT, NF>) -> Self {
        match value {
            MoveResult::Playing(s) => GameStateOption::Playing(s),
            MoveResult::Win(s) => GameStateOption::Win(s),
        }
    }
}

impl<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> GameState<C, NC, PileRef>
    for GameStateOption<C, NC, NT, NF>
{
    fn get_stack(&self, p: PileRef) -> Option<&Stack<C>> {
        match self {
            GameStateOption::Initial(s) => s.get_stack(p),
            GameStateOption::Playing(s) => s.get_stack(p),
            GameStateOption::Win(s) => s.get_stack(p),
        }
    }

    fn get_stack_mut(&mut self, p: PileRef) -> Option<&mut Stack<C>> {
        match self {
            GameStateOption::Initial(s) => s.get_stack_mut(p),
            GameStateOption::Playing(s) => s.get_stack_mut(p),
            GameStateOption::Win(s) => s.get_stack_mut(p),
        }
    }
}

impl<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> From<InitialGameState<C, NC>>
    for GameStateOption<C, NC, NT, NF>
{
    fn from(value: InitialGameState<C, NC>) -> Self {
        GameStateOption::Initial(value)
    }
}

impl<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize>
    From<PlayingGameState<C, NC, NT, NF>> for GameStateOption<C, NC, NT, NF>
{
    fn from(value: PlayingGameState<C, NC, NT, NF>) -> Self {
        GameStateOption::Playing(value)
    }
}

impl<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> From<WinGameState<C, NC, NF>>
    for GameStateOption<C, NC, NT, NF>
{
    fn from(value: WinGameState<C, NC, NF>) -> Self {
        GameStateOption::Win(value)
    }
}

/// Enum for the resulting [GameState] after making a move,
/// either [Playing](PlayingGameState) (game not finished) or [Win](WinGameState)
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MoveResult<C: Card<NC>, const NC: usize, const NT: usize, const NF: usize> {
    Playing(PlayingGameState<C, NC, NT, NF>),
    Win(WinGameState<C, NC, NF>),
}
