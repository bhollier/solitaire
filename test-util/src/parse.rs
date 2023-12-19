use solitaire::std;

pub fn rank(str: &str) -> std::Rank {
    match str {
        "A" | "1" => std::Rank::Ace,
        "2" => std::Rank::Two,
        "3" => std::Rank::Three,
        "4" => std::Rank::Four,
        "5" => std::Rank::Five,
        "6" => std::Rank::Six,
        "7" => std::Rank::Seven,
        "8" => std::Rank::Eight,
        "9" => std::Rank::Nine,
        "X" => std::Rank::Ten,
        "J" => std::Rank::Jack,
        "Q" => std::Rank::Queen,
        "K" => std::Rank::King,
        &_ => panic!("unknown rank {}", str),
    }
}

pub fn suit(str: &str) -> std::FrenchSuit {
    match str {
        "♣" | "C" => std::FrenchSuit::Clubs,
        "♠" | "S" => std::FrenchSuit::Spades,
        "♥" | "H" => std::FrenchSuit::Hearts,
        "♦" | "D" => std::FrenchSuit::Diamonds,
        &_ => panic!("unknown suit {}", str),
    }
}

pub fn card(str: &str) -> std::Card {
    let (rank_str, suit_str) = str.split_at(1);
    std::Card {
        suit: suit(suit_str),
        rank: rank(rank_str),
    }
}

pub fn cards(strs: &[&str]) -> Vec<std::Card> {
    strs.iter().map(|str| card(str)).collect()
}
