use solitaire::variant::klondike::*;
use solitaire::*;
use test_util::parse;

/// Test the initial deal
#[test]
fn test_game_rules_deal() {
    let deck: std::Deck = std::Card::new_deck();
    let game = GameRules::deal(InitialGameState::from(deck));

    // Verify the cards were added to the tableau in the correct order
    let mut total_taken = 0;
    for (i, stack) in game.tableau.iter().enumerate() {
        assert_eq!(stack.len(), i + 1);
        let mut index = i;
        for (j, card) in stack.iter().enumerate() {
            assert_eq!(card, &deck[deck.len() - index - 1]);
            index += NUM_TABLEAU - j - 1;
            total_taken += 1;
        }
    }

    // Verify the stock has the correct cards
    let expected_stock: std::Stack = deck[0..deck.len() - total_taken].iter().cloned().collect();
    assert_eq!(game.stock, expected_stock);
}

/// Test drawing from the stock pile
#[test]
fn test_game_rules_draw_stock() -> Result<()> {
    let stock = parse::cards(&vec!["KC", "AH"]);
    let mut game = PlayingGameState {
        tableau: [(); NUM_TABLEAU].map(|_| Stack::new()),
        foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
        stock: Stack::from_slice(&stock),
        talon: Stack::new(),
    };

    // idiot check
    assert!(game.talon.is_empty());

    game = GameRules::draw_stock(game, 1)?;

    assert_eq!(game.stock, vec![parse::card("KC")]);
    assert_eq!(game.talon, vec![parse::card("AH")]);

    game = GameRules::draw_stock(game, 1)?;

    assert!(game.stock.is_empty());
    assert_eq!(game.talon, parse::cards(&vec!["AH", "KC"]));

    game = GameRules::draw_stock(game, 1)?;

    assert_eq!(game.stock, parse::cards(&vec!["KC", "AH"]));
    assert!(game.talon.is_empty());

    Ok(())
}

/// Test the sequence validation on its own for a tableau pile
#[test]
fn test_game_rules_valid_seq_tableau() {
    let p = std::PileRef::Tableau(0);

    let valid = parse::cards(&vec!["KC", "QH", "JS", "XD"]);
    assert!(GameRules::valid_seq(p, valid.as_slice()));

    let invalid_wrong_dir: Vec<std::Card> = valid.iter().rev().cloned().collect();
    assert!(!GameRules::valid_seq(p, invalid_wrong_dir.as_slice()));

    let invalid_same_color = parse::cards(&vec!["8H", "7D", "6D"]);
    assert!(!GameRules::valid_seq(p, invalid_same_color.as_slice()));

    let invalid_overflow = parse::cards(&vec!["2C", "AH", "KS"]);
    assert!(!GameRules::valid_seq(p, invalid_overflow.as_slice()));
}

/// Test the sequence validation on its own for a foundation pile
#[test]
fn test_game_rules_valid_seq_foundation() {
    let p = std::PileRef::Foundation(0);

    let valid = parse::cards(&vec!["XC", "JC", "QC", "KC"]);
    assert!(GameRules::valid_seq(p, valid.as_slice()));

    let invalid_wrong_dir: Vec<std::Card> = valid.iter().rev().cloned().collect();
    assert!(!GameRules::valid_seq(p, invalid_wrong_dir.as_slice()));

    let invalid_different_suit = parse::cards(&vec!["6D", "7D", "8H"]);
    assert!(!GameRules::valid_seq(p, invalid_different_suit.as_slice()));

    let invalid_overflow = parse::cards(&vec!["QC", "KC", "AC"]);
    assert!(!GameRules::valid_seq(p, invalid_overflow.as_slice()));
}

/// Test basic input validation when moving cards,
/// e.g. making sure you can't take 0 cards,
/// or trying to move cards to the stock, etc.
#[test]
fn test_game_rules_move_cards_invalid_input() {
    let game = GameRules::new_and_deal();

    let pile = std::PileRef::Tableau(0);

    assert_eq!(
        GameRules::move_cards(game.clone(), pile, 0, pile).err(),
        Some(Error::InvalidInput {
            field: "take_n",
            reason: "cannot take 0 cards"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Stock,
            1,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidInput {
            field: "src",
            reason: "cannot move cards from stock"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Talon,
            2,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "cannot move more than 1 card from talon"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Tableau(0),
            2,
            std::PileRef::Foundation(0)
        ),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "cannot move more than 1 card to foundation"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Tableau(0),
            1,
            std::PileRef::Stock
        ),
        Err(Error::InvalidInput {
            field: "dst",
            reason: "cannot move cards to stock"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Tableau(0),
            1,
            std::PileRef::Talon
        ),
        Err(Error::InvalidInput {
            field: "dst",
            reason: "cannot move cards to talon"
        })
    );
}

/// Test state-dependent validation when moving cards,
/// e.g. making sure you can't take an invalid sequence,
/// or move cards onto an invalid card, etc.
#[test]
fn test_game_rules_move_cards_invalid_move() -> Result<()> {
    let stock = parse::cards(&vec!["KC", "AH"]);
    let tableau0 = parse::cards(&vec!["2S"]);
    let tableau1 = parse::cards(&vec!["6H", "3D"]);

    let mut game = PlayingGameState {
        tableau: [
            tableau0,
            tableau1,
            Stack::new(),
            Stack::new(),
            Stack::new(),
            Stack::new(),
            Stack::new(),
        ],
        foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
        stock: Stack::from_slice(&stock),
        talon: Stack::new(),
    };

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Talon,
            1,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "not enough cards in src pile"
        })
    );

    // Draw so the card in the stock is available in the talon
    game = GameRules::draw_stock(game, 1)?;

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Talon,
            1,
            std::PileRef::Tableau(2)
        ),
        Err(Error::InvalidMove {
            reason: "can only move a King to a space"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Tableau(0),
            1,
            std::PileRef::Foundation(0)
        ),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Talon,
            1,
            std::PileRef::Tableau(1)
        ),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Tableau(1),
            2,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidMove {
            reason: "src sequence is invalid"
        })
    );

    assert_eq!(
        GameRules::move_cards(
            game.clone(),
            std::PileRef::Tableau(1),
            1,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    Ok(())
}

/// Test moving cards around
#[test]
fn test_game_rules_move_cards() -> Result<()> {
    let stock = parse::cards(&vec!["KC", "AH"]);
    let tableau0 = parse::cards(&vec!["2S"]);
    let tableau1 = parse::cards(&vec!["3D"]);

    let mut game = PlayingGameState {
        tableau: [
            tableau0,
            tableau1,
            Stack::new(),
            Stack::new(),
            Stack::new(),
            Stack::new(),
            Stack::new(),
        ],
        foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
        stock: Stack::from_slice(&stock),
        talon: Stack::new(),
    };

    // Draw so the card in the stock is available in the talon
    game = GameRules::draw_stock(game, 1)?;

    // Move the Ace of Hearts to the first tableau with a 2 of Spades
    game = match GameRules::move_cards(game, std::PileRef::Talon, 1, std::PileRef::Tableau(0))? {
        MoveResult::Playing(new) => new,
        MoveResult::Win(_) => panic!(),
    };

    // Talon is now empty
    assert!(game.talon.is_empty());
    // Tableau is 2 of Spades and Ace of Hearts
    assert_eq!(game.tableau[0], parse::cards(&vec!["2S", "AH"]));

    // Move the stack to the second tableau with a 3 of Diamonds
    game = match GameRules::move_cards(game, std::PileRef::Tableau(0), 2, std::PileRef::Tableau(1))?
    {
        MoveResult::Playing(new) => new,
        MoveResult::Win(_) => panic!(),
    };

    // First tableau is now empty
    assert!(game.tableau[0].is_empty());
    // Second tableau is the 3 of Diamonds, 2 of Spades and Ace of Hearts
    assert_eq!(game.tableau[1], parse::cards(&vec!["3D", "2S", "AH"]));

    // Move the Ace of Hearts to the foundation
    game = match GameRules::move_cards(
        game,
        std::PileRef::Tableau(1),
        1,
        std::PileRef::Foundation(0),
    )? {
        MoveResult::Playing(new) => new,
        MoveResult::Win(_) => panic!(),
    };

    // Tableau is the 3 of Diamonds and 2 of Spades
    assert_eq!(game.tableau[1], parse::cards(&vec!["3D", "2S"]));
    // Foundation is the Ace of Hearts
    assert_eq!(game.foundations[0], vec![parse::card("AH")]);

    // Draw so the King of Clubs is available
    game = GameRules::draw_stock(game, 1)?;

    // Move the King of Clubs to the third tableau which is empty
    game = match GameRules::move_cards(game, std::PileRef::Talon, 1, std::PileRef::Tableau(2))? {
        MoveResult::Playing(new) => new,
        MoveResult::Win(_) => panic!(),
    };

    // Talon is now empty
    assert!(game.talon.is_empty());
    // Third tableau is King of Clubs
    assert_eq!(game.tableau[2], vec![parse::card("KC")]);

    Ok(())
}

/// Test the win condition
#[test]
fn test_game_rules_move_cards_win() -> Result<()> {
    let tableau0 = parse::cards(&vec!["KS"]);
    let foundation0 = parse::cards(&vec![
        "AC", "2C", "3C", "4C", "5C", "6C", "7C", "8C", "9C", "XC", "JC", "QC", "KC",
    ]);
    let foundation1 = parse::cards(&vec![
        "AS", "2S", "3S", "4S", "5S", "6S", "7S", "8S", "9S", "XS", "JS", "QS",
    ]);
    let foundation2 = parse::cards(&vec![
        "AH", "2H", "3H", "4H", "5H", "6H", "7H", "8H", "9H", "XH", "JH", "QH", "KH",
    ]);
    let foundation3 = parse::cards(&vec![
        "AD", "2D", "3D", "4D", "5D", "6D", "7D", "8D", "9D", "XD", "JD", "QD", "KD",
    ]);

    let game = PlayingGameState {
        tableau: [
            tableau0,
            Stack::new(),
            Stack::new(),
            Stack::new(),
            Stack::new(),
            Stack::new(),
            Stack::new(),
        ],
        foundations: [foundation0, foundation1, foundation2, foundation3],
        stock: Stack::new(),
        talon: Stack::new(),
    };

    // Move the King to the foundation
    let win = match GameRules::move_cards(
        game,
        std::PileRef::Tableau(0),
        1,
        std::PileRef::Foundation(1),
    )? {
        MoveResult::Playing(_) => panic!(),
        MoveResult::Win(new) => new,
    };

    for foundation in win.foundations {
        assert_eq!(foundation.len(), std::Rank::N);
    }

    Ok(())
}
