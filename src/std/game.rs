use crate as solitaire;
use crate::{
    std, take_n_slice, take_n_vec_mut, Card, Error, GameState as GameStateTrait, Result, Stack,
    StackFrom,
};
use ::std::cmp;

pub const NUM_TABLEAU: usize = 7;
pub const NUM_FOUNDATIONS: usize = 4;

/// "Standard" (Klondike) solitaire piles
#[derive(Eq, PartialEq)]
pub enum PileRef {
    /// The "tableau" of 7 [Stack]s where cards are moved around
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
    fn new(deck: &'a [C]) -> GenericGameState<'a, C, N> {
        GenericGameState {
            tableau: [(); NUM_TABLEAU].map(|_| Stack::new()),
            foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
            stock: Stack::from_slice(deck),
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

macro_rules! impl_mut {
    ($func_name:ident, $mut_func_name:ident $(,$($arg:ident:$arg_ty:ty),+)?) => {
        #[doc="Mutable version of "]
        #[doc=concat!("[GameRules::", stringify!($func_name), "]")]
        #[doc=" which applies the result to the `state`."]
        #[doc="If [Err] is returned then `state` is not be modified"]
        pub fn $mut_func_name(state: &mut GameState $(,$($arg: $arg_ty),*)?) -> Result<()> {
            let result = Self::$func_name(state.clone() $(,$($arg),*)?)?;
            Ok(*state = result)
        }
    };
}

impl GameRules {
    /// Deals out the initial cards at the start of the game,
    /// returning a clone of `state` with the result of the deal.
    pub fn deal(state: GameState) -> Result<GameState> {
        if state.tableau.iter().any(|s| !s.is_empty()) {
            return Err(Error::InvalidState);
        }

        let mut new_state = state;
        let mut card: &std::Card;
        for i in 0..NUM_TABLEAU {
            for j in i..NUM_TABLEAU {
                card = solitaire::take_one_vec_mut(&mut new_state.stock);
                new_state.tableau[j].push(card);
            }
        }

        Ok(new_state)
    }

    impl_mut!(deal, deal_mut);

    /// Draws `n` cards from the [Stock](PileRef::Stock) onto the [Talon](PileRef::Talon).
    /// If the stock is empty, the talon is turned over and used as the stock.
    pub fn draw_stock(state: GameState, n: usize) -> Result<GameState> {
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
                take.iter().for_each(|c| new_state.talon.push(*c));
            }
        }
        Ok(new_state)
    }

    impl_mut!(draw_stock, draw_stock_mut, n: usize);

    /// If the given sequence of cards is valid to be moved by a player for the given [pile](PileRef),
    /// using the following rules:
    /// - [Foundation](PileRef::Foundation): cards must be of the same [Suit] and in Ace to King order
    /// - [Tableau](PileRef::Tableau): cards must be of alternating [Color](std::Color) and in King to Ace order
    /// - [Stock](PileRef::Stock): always true
    /// - [Talon](PileRef::Talon): always true
    pub fn valid_seq(p: &PileRef, cs: &[&std::Card]) -> bool {
        match p {
            PileRef::Tableau(_) => {
                let mut prev_card = cs[0];
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
                let mut prev_card = cs[0];
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
    /// returning a clone of `state` with the result of the move.
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
        state: GameState,
        src: PileRef,
        take_n: usize,
        dst: PileRef,
    ) -> Result<GameState> {
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
            return Ok(state);
        }

        // Create stacks for the new state of src and dst
        let new_src_stack: std::Stack;
        let new_dst_stack: std::Stack;
        {
            let src_pile = state.get_stack(&src).ok_or(Error::InvalidInput {
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
            if !Self::valid_seq(&src, take) {
                return Err(Error::InvalidMove {
                    reason: "src sequence is invalid",
                });
            }

            new_src_stack = rest.iter().map(|c| *c).collect();

            let dst_pile = state.get_stack(&dst).ok_or(Error::InvalidInput {
                field: "dst",
                reason: "pile does not exist",
            })?;

            new_dst_stack = dst_pile.iter().chain(take.iter()).map(|c| *c).collect();

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
                    &dst,
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
        *new_state.get_stack_mut(&src).unwrap() = new_src_stack;
        *new_state.get_stack_mut(&dst).unwrap() = new_dst_stack;
        Ok(new_state)
    }

    impl_mut!(
        move_cards,
        move_cards_mut,
        src: PileRef,
        take_n: usize,
        dst: PileRef
    );
}
