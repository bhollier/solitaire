use ::std::collections::HashSet;
use solitaire::{std, Card};

#[test]
fn test_new_deck() {
    let d: std::Deck = std::Card::new_deck();
    assert_eq!(d.len(), 52);

    let distinct_cards = d.iter().collect::<HashSet<_>>();
    assert_eq!(distinct_cards.len(), 52);
}

#[test]
fn test_ordering() {
    let c1 = std::Card {
        suit: std::FrenchSuit::Clubs,
        rank: std::Rank::Ace,
    };
    let c2 = std::Card {
        suit: std::FrenchSuit::Clubs,
        rank: std::Rank::King,
    };
    assert!(c1 > c2);
    assert!(c2 < c1);
}
