use lazy_static::lazy_static;
use regex::Regex;
use solitaire::common;

pub fn rank(str: &str) -> common::Rank {
    match str {
        "A" | "1" => common::Rank::Ace,
        "2" => common::Rank::Two,
        "3" => common::Rank::Three,
        "4" => common::Rank::Four,
        "5" => common::Rank::Five,
        "6" => common::Rank::Six,
        "7" => common::Rank::Seven,
        "8" => common::Rank::Eight,
        "9" => common::Rank::Nine,
        "X" => common::Rank::Ten,
        "J" => common::Rank::Jack,
        "Q" => common::Rank::Queen,
        "K" => common::Rank::King,
        &_ => panic!("unknown rank {}", str),
    }
}

pub fn suit(str: &str) -> common::FrenchSuit {
    match str {
        "♣" | "C" => common::FrenchSuit::Clubs,
        "♠" | "S" => common::FrenchSuit::Spades,
        "♥" | "H" => common::FrenchSuit::Hearts,
        "♦" | "D" => common::FrenchSuit::Diamonds,
        &_ => panic!("unknown suit {}", str),
    }
}

lazy_static! {
    static ref CARD_PATTERN: Regex = Regex::new(r"^(?<h>#)?(?<r>.)(?<s>.)$").unwrap();
}

pub fn card(str: &str) -> common::Card {
    let captures = CARD_PATTERN.captures(str).unwrap();
    let (rank_str, suit_str) = (
        captures.name("r").unwrap().as_str(),
        captures.name("s").unwrap().as_str(),
    );
    common::Card {
        suit: suit(suit_str),
        rank: rank(rank_str),
        face_up: captures.name("h") == None,
    }
}

pub fn cards(strs: &[&str]) -> Vec<common::Card> {
    strs.iter().map(|str| card(str)).collect()
}
