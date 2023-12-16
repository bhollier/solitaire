use solitaire::std;

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
