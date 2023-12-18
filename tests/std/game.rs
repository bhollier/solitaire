use super::card::parse;
use solitaire::std;
use solitaire::*;

/// Test the initial deal
#[test]
fn test_game_rules_deal() {
    let deck: std::Deck = std::Card::new_deck();
    let mut game = std::GameState::new(&deck);
    assert_eq!(std::GameRules::deal_mut(&mut game), Ok(()));

    // Verify the cards were added to the tableau in the correct order
    let mut total_taken = 0;
    for (i, stack) in game.tableau.iter().enumerate() {
        assert_eq!(stack.len(), i + 1);
        let mut index = i;
        for (j, card) in stack.iter().enumerate() {
            assert_eq!(*card, &deck[deck.len() - index - 1]);
            index += std::NUM_TABLEAU - j - 1;
            total_taken += 1;
        }
    }

    // Verify the stock has the correct cards
    let expected_stock: std::Stack = deck[0..deck.len() - total_taken].iter().collect();
    assert_eq!(game.stock, expected_stock);

    // Can't deal again
    assert_eq!(
        std::GameRules::deal_mut(&mut game),
        Err(Error::InvalidState)
    );
}

/// Test drawing from the stock pile
#[test]
fn test_game_rules_draw_stock() {
    let stock = parse::cards(&vec!["KC", "AH"]);
    let mut game = std::GameState::new(&stock);

    // idiot check
    assert!(game.talon.is_empty());

    assert_eq!(std::GameRules::draw_stock_mut(&mut game, 1), Ok(()));

    assert_eq!(game.stock, vec![&parse::card("KC")]);
    assert_eq!(game.talon, vec![&parse::card("AH")]);

    assert_eq!(std::GameRules::draw_stock_mut(&mut game, 1), Ok(()));

    assert!(game.stock.is_empty());
    assert_eq!(
        game.talon,
        Stack::from_vec(&parse::cards(&vec!["AH", "KC"]))
    );

    assert_eq!(std::GameRules::draw_stock_mut(&mut game, 1), Ok(()));

    assert_eq!(
        game.stock,
        Stack::from_vec(&parse::cards(&vec!["KC", "AH"]))
    );
    assert!(game.talon.is_empty());
}

/// Test the sequence validation on its own for a tableau pile
#[test]
fn test_game_rules_valid_seq_tableau() {
    let p = &std::PileRef::Tableau(0);

    let valid = parse::cards(&vec!["KC", "QH", "JS", "XD"]);
    assert!(std::GameRules::valid_seq(
        p,
        Stack::from_vec(&valid).as_slice()
    ));

    let invalid_wrong_dir: Vec<&std::Card> = valid.iter().rev().collect();
    assert!(!std::GameRules::valid_seq(p, invalid_wrong_dir.as_slice()));

    let invalid_same_color = parse::cards(&vec!["8H", "7D", "6D"]);
    assert!(!std::GameRules::valid_seq(
        p,
        Stack::from_vec(&invalid_same_color).as_slice()
    ));

    let invalid_overflow = parse::cards(&vec!["2C", "AH", "KS"]);
    assert!(!std::GameRules::valid_seq(
        p,
        Stack::from_vec(&invalid_overflow).as_slice()
    ));
}

/// Test the sequence validation on its own for a foundation pile
#[test]
fn test_game_rules_valid_seq_foundation() {
    let p = &std::PileRef::Foundation(0);

    let valid = parse::cards(&vec!["XC", "JC", "QC", "KC"]);
    assert!(std::GameRules::valid_seq(
        p,
        Stack::from_vec(&valid).as_slice()
    ));

    let invalid_wrong_dir: Vec<&std::Card> = valid.iter().rev().collect();
    assert!(!std::GameRules::valid_seq(p, invalid_wrong_dir.as_slice()));

    let invalid_different_suit = parse::cards(&vec!["6D", "7D", "8H"]);
    assert!(!std::GameRules::valid_seq(
        p,
        Stack::from_vec(&invalid_different_suit).as_slice()
    ));

    let invalid_overflow = parse::cards(&vec!["QC", "KC", "AC"]);
    assert!(!std::GameRules::valid_seq(
        p,
        Stack::from_vec(&invalid_overflow).as_slice()
    ));
}

/// Test basic input validation when moving cards,
/// e.g. making sure you can't take 0 cards,
/// or trying to move cards to the stock, etc.
#[test]
fn test_game_rules_move_cards_invalid_input() {
    let deck = std::Card::new_deck();
    let mut game = std::GameState::new(&deck);

    assert_eq!(
        std::GameRules::move_cards_mut(
            &mut game,
            std::PileRef::Tableau(0),
            0,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "cannot take 0 cards"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Stock, 1, std::PileRef::Tableau(0)),
        Err(Error::InvalidInput {
            field: "src",
            reason: "cannot move cards from stock"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Talon, 2, std::PileRef::Tableau(0)),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "cannot move more than 1 card from talon"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(
            &mut game,
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
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Tableau(0), 1, std::PileRef::Stock),
        Err(Error::InvalidInput {
            field: "dst",
            reason: "cannot move cards to stock"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Tableau(0), 1, std::PileRef::Talon),
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
fn test_game_rules_move_cards_invalid_move() {
    let stock = parse::cards(&vec!["KC", "AH"]);
    let tableau0 = parse::cards(&vec!["2S"]);
    let tableau1 = parse::cards(&vec!["6H", "3D"]);

    let mut game = std::GameState::new(&stock);
    game.tableau[0] = Stack::from_vec(&tableau0);
    game.tableau[1] = Stack::from_vec(&tableau1);

    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Talon, 1, std::PileRef::Tableau(0)),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "not enough cards in src pile"
        })
    );

    // Draw so the card in the stock is available in the talon
    assert_eq!(std::GameRules::draw_stock_mut(&mut game, 1), Ok(()));

    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Talon, 1, std::PileRef::Tableau(2)),
        Err(Error::InvalidMove {
            reason: "can only move a King to a space"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(
            &mut game,
            std::PileRef::Tableau(0),
            1,
            std::PileRef::Foundation(0)
        ),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Talon, 1, std::PileRef::Tableau(1)),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(
            &mut game,
            std::PileRef::Tableau(1),
            2,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidMove {
            reason: "src sequence is invalid"
        })
    );

    assert_eq!(
        std::GameRules::move_cards_mut(
            &mut game,
            std::PileRef::Tableau(1),
            1,
            std::PileRef::Tableau(0)
        ),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );
}

/// Test moving cards around
#[test]
fn test_game_rules_move_cards() {
    let stock = parse::cards(&vec!["KC", "AH"]);
    let tableau0 = parse::cards(&vec!["2S"]);
    let tableau1 = parse::cards(&vec!["3D"]);

    let mut game = std::GameState::new(&stock);
    game.tableau[0] = Stack::from_vec(&tableau0);
    game.tableau[1] = Stack::from_vec(&tableau1);

    // Draw so the card in the stock is available in the talon
    assert_eq!(std::GameRules::draw_stock_mut(&mut game, 1), Ok(()));

    // Move the Ace of Hearts to the first tableau with a 2 of Spades
    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Talon, 1, std::PileRef::Tableau(0)),
        Ok(())
    );

    // Talon is now empty
    assert!(game.talon.is_empty());
    // Tableau is 2 of Spades and Ace of Hearts
    assert_eq!(
        game.tableau[0],
        Stack::from_vec(&parse::cards(&vec!["2S", "AH"]))
    );

    // Move the stack to the second tableau with a 3 of Diamonds
    assert_eq!(
        std::GameRules::move_cards_mut(
            &mut game,
            std::PileRef::Tableau(0),
            2,
            std::PileRef::Tableau(1)
        ),
        Ok(())
    );

    // First tableau is now empty
    assert!(game.tableau[0].is_empty());
    // Second tableau is the 3 of Diamonds, 2 of Spades and Ace of Hearts
    assert_eq!(
        game.tableau[1],
        Stack::from_vec(&parse::cards(&vec!["3D", "2S", "AH"]))
    );

    // Move the Ace of Hearts to the foundation
    assert_eq!(
        std::GameRules::move_cards_mut(
            &mut game,
            std::PileRef::Tableau(1),
            1,
            std::PileRef::Foundation(0)
        ),
        Ok(())
    );

    // Tableau is the 3 of Diamonds and 2 of Spades
    assert_eq!(
        game.tableau[1],
        Stack::from_vec(&parse::cards(&vec!["3D", "2S"]))
    );
    // Foundation is the Ace of Hearts
    assert_eq!(game.foundations[0], vec![&parse::card("AH")]);

    // Draw so the King of Clubs is available
    assert_eq!(std::GameRules::draw_stock_mut(&mut game, 1), Ok(()));

    // Move the King of Clubs to the third tableau which is empty
    assert_eq!(
        std::GameRules::move_cards_mut(&mut game, std::PileRef::Talon, 1, std::PileRef::Tableau(2)),
        Ok(())
    );

    // Talon is now empty
    assert!(game.talon.is_empty());
    // Third tableau is King of Clubs
    assert_eq!(game.tableau[2], vec![&parse::card("KC")]);
}
