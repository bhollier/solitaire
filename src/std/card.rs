use crate as solitaire;
use std::slice::Iter;

/// A classic "French" [Suit](solitaire::Suit), with "Clubs", "Spades", "Hearts" and "Diamonds"
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FrenchSuit {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

impl solitaire::Suit for FrenchSuit {
    fn iter() -> Iter<'static, Self> {
        static VALUES: [FrenchSuit; 4] = [
            FrenchSuit::Clubs,
            FrenchSuit::Spades,
            FrenchSuit::Hearts,
            FrenchSuit::Diamonds,
        ];
        VALUES.iter()
    }
}

/// The standard Ranks of a cards, which is King, Queen, Jack, Ten to Two and Ace.
/// [Ord] is defined according to this ordering, as this is how cards are ordered in a [Stack](solitaire::Stack)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

impl solitaire::Rank for Rank {
    fn iter() -> Iter<'static, Self> {
        static VALUES: [Rank; 13] = [
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
        VALUES.iter()
    }
}

/// Convenience type alias for a "standard" Card
pub type Card = solitaire::Card<FrenchSuit, Rank>;
/// Convenience type alias for a [Deck](solitaire::Deck) of [Card]
pub type Deck = solitaire::Deck<FrenchSuit, Rank>;
/// Convenience type alias for a [Stack](solitaire::Stack) for [Card]
pub type Stack<'a> = solitaire::Stack<'a, FrenchSuit, Rank>;
