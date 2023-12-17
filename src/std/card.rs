use crate as solitaire;
use arr_macro::arr;
use std::cmp::Ordering;
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
    pub const VALUES: [FrenchSuit; FrenchSuit::COUNT] = [
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

/// The standard Ranks of a cards, which is King, Queen, Jack, Ten to Two and Ace.
/// [Ord] is defined according to this ordering, as this is how cards are ordered in a [Stack](solitaire::Stack)
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
    pub const VALUES: [Rank; Rank::COUNT] = [
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

/// A standard [Card](solitaire::Card) with a suit and a rank.
/// Ord is implemented but only acts on the card's [Rank]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Card {
    pub suit: FrenchSuit,
    pub rank: Rank,
}

impl Card {
    pub const N: usize = FrenchSuit::COUNT * Rank::COUNT;
    fn from_index(i: usize) -> Card {
        Card {
            suit: FrenchSuit::VALUES[i / Rank::COUNT],
            rank: Rank::VALUES[i % Rank::COUNT],
        }
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rank.partial_cmp(&other.rank)
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

/// Convenience type alias for a [Stack](solitaire::Stack) for [Card]
pub type Stack<'a> = solitaire::Stack<'a, Card>;
