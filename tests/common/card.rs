use std::collections::HashSet;

use solitaire::{common, Card};

#[test]
fn test_new_deck() {
    let d: common::Deck = common::Card::new_deck();
    assert_eq!(d.len(), 52);

    let distinct_cards = d.iter().collect::<HashSet<_>>();
    assert_eq!(distinct_cards.len(), 52);
}

#[test]
fn test_ordering() {
    let c1 = common::Card {
        suit: common::FrenchSuit::Clubs,
        rank: common::Rank::Ace,
    };
    let c2 = common::Card {
        suit: common::FrenchSuit::Clubs,
        rank: common::Rank::King,
    };
    assert!(c1 > c2);
    assert!(c2 < c1);
}
