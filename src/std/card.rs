use crate as solitaire;
use arr_macro::arr;
use std::*;
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

/// The color of a [FrenchSuit]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Color {
    Black,
    Red,
}

/// A classic "French" [Suit](solitaire::Suit), with "Clubs", "Spades", "Hearts" and "Diamonds"
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumCountMacro)]
pub enum FrenchSuit {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

impl FrenchSuit {
    pub const N: usize = <FrenchSuit as EnumCount>::COUNT;
    pub const VALUES: [FrenchSuit; FrenchSuit::N] = [
        FrenchSuit::Clubs,
        FrenchSuit::Spades,
        FrenchSuit::Hearts,
        FrenchSuit::Diamonds,
    ];

    pub fn color(&self) -> Color {
        match self {
            FrenchSuit::Clubs => Color::Black,
            FrenchSuit::Spades => Color::Black,
            FrenchSuit::Hearts => Color::Red,
            FrenchSuit::Diamonds => Color::Red,
        }
    }
}

impl fmt::Display for FrenchSuit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrenchSuit::Clubs => write!(f, "♣"),
            FrenchSuit::Spades => write!(f, "♠"),
            FrenchSuit::Hearts => write!(f, "♥"),
            FrenchSuit::Diamonds => write!(f, "♦"),
        }
    }
}

/// The standard Ranks of cards, which is King, Queen, Jack, Ten to Two and Ace.
/// [Ord] is defined according to this ordering,
/// as this is how cards are ordered in a [Stack](solitaire::Stack)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, EnumCountMacro)]
pub enum Rank {
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
    Ace,
}

impl Rank {
    pub const N: usize = <Rank as EnumCount>::COUNT;
    pub const VALUES: [Rank; Rank::N] = [
        Rank::King,
        Rank::Queen,
        Rank::Jack,
        Rank::Ten,
        Rank::Nine,
        Rank::Eight,
        Rank::Seven,
        Rank::Six,
        Rank::Five,
        Rank::Four,
        Rank::Three,
        Rank::Two,
        Rank::Ace,
    ];
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rank::King => write!(f, "K"),
            Rank::Queen => write!(f, "Q"),
            Rank::Jack => write!(f, "J"),
            Rank::Ten => write!(f, "X"),
            Rank::Nine => write!(f, "9"),
            Rank::Eight => write!(f, "8"),
            Rank::Seven => write!(f, "7"),
            Rank::Six => write!(f, "6"),
            Rank::Five => write!(f, "5"),
            Rank::Four => write!(f, "4"),
            Rank::Three => write!(f, "3"),
            Rank::Two => write!(f, "2"),
            Rank::Ace => write!(f, "A"),
        }
    }
}

/// A standard [Card](solitaire::Card) with a suit and a rank.
/// Ord is implemented but only acts on the card's [Rank]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Card {
    pub suit: FrenchSuit,
    pub rank: Rank,
}

impl Card {
    pub const N: usize = FrenchSuit::N * Rank::N;
    fn from_index(i: usize) -> Card {
        Card {
            suit: FrenchSuit::VALUES[i / Rank::N],
            rank: Rank::VALUES[i % Rank::N],
        }
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.rank.partial_cmp(&other.rank)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl solitaire::Card<{ Card::N }> for Card {
    fn new_deck() -> Deck {
        let mut i = 0;
        arr![Card::from_index({i += 1; i - 1}); 52]
    }
}

/// Convenience type alias for a [Deck](solitaire::Deck) of [Card]
pub type Deck = solitaire::Deck<Card, { Card::N }>;

/// Convenience type alias for a [Stack](solitaire::Stack) of [Card]
pub type Stack<'d> = solitaire::Stack<'d, Card>;
