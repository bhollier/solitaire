use solitaire::variant::klondike::*;
use test_util::parse;

fn validate_deal_all_tableau(deck: &Deck, game: &PlayingGameState) {
    // Verify the cards were added to the tableau in the correct order
    let mut total_taken = 0;
    for (i, stack) in game.tableau.iter().enumerate() {
        assert_eq!(stack.len(), i + 1);
        let mut index = i;
        for (j, card) in stack.iter().enumerate() {
            let deck_card = &deck[deck.len() - index - 1];
            assert_eq!(card.rank, deck_card.rank);
            assert_eq!(card.suit, deck_card.suit);
            if j == stack.len() - 1 {
                assert_eq!(card.face_up, true)
            }
            index += NUM_TABLEAU - j - 1;
            total_taken += 1;
        }
    }

    // Verify the stock has the correct cards
    let expected_stock: Stack = deck[0..deck.len() - total_taken].iter().cloned().collect();
    assert_eq!(game.stock, expected_stock);
}

/// Test a single card deal
#[test]
fn test_game_rules_deal_one() {
    let deck: Deck = Card::new_deck();
    let mut game = InitialGameState::from(deck);

    game = match GameRules::deal_one(game) {
        DealResult::Dealing(s) => s,
        DealResult::Complete(_) => panic!(),
    };

    {
        assert_eq!(game.tableau[0].len(), 1);
        let deck_card = &deck[deck.len() - 1];
        assert_eq!(game.tableau[0][0].rank, deck_card.rank);
        assert_eq!(game.tableau[0][0].suit, deck_card.suit);
        assert!(game.tableau[0][0].face_up);

        for tableau in &game.tableau[1..NUM_TABLEAU] {
            assert!(tableau.is_empty());
        }
    }

    game = match GameRules::deal_one(game) {
        DealResult::Dealing(s) => s,
        DealResult::Complete(_) => panic!(),
    };

    {
        assert_eq!(game.tableau[0].len(), 1);
        let deck_card = &deck[deck.len() - 1];
        assert_eq!(game.tableau[0][0].rank, deck_card.rank);
        assert_eq!(game.tableau[0][0].suit, deck_card.suit);
        assert!(game.tableau[0][0].face_up);

        assert_eq!(game.tableau[1].len(), 1);
        let deck_card = &deck[deck.len() - 2];
        assert_eq!(game.tableau[1][0].rank, deck_card.rank);
        assert_eq!(game.tableau[1][0].suit, deck_card.suit);
        assert!(!game.tableau[1][0].face_up);

        for tableau in &game.tableau[2..NUM_TABLEAU] {
            assert!(tableau.is_empty());
        }
    }

    fn deal_all(mut g: InitialGameState) -> PlayingGameState {
        loop {
            match GameRules::deal_one(g) {
                DealResult::Dealing(new_state) => g = new_state,
                DealResult::Complete(new_state) => return new_state,
            }
        }
    }

    validate_deal_all_tableau(&deck, &deal_all(game))
}

/// Test the full initial deal
#[test]
fn test_game_rules_deal_all() {
    let deck: Deck = Card::new_deck();
    let game = GameRules::deal_all(InitialGameState::from(deck));

    validate_deal_all_tableau(&deck, &game)
}

/// Test drawing from the stock pile
#[test]
fn test_game_rules_draw_stock() -> Result<()> {
    let stock = parse::cards(&vec!["#KC", "#AH"]);
    let mut game = PlayingGameState {
        tableau: [(); NUM_TABLEAU].map(|_| Stack::new()),
        foundations: [(); NUM_FOUNDATIONS].map(|_| Stack::new()),
        stock: Stack::from_slice(&stock),
        talon: Stack::new(),
    };

    // idiot check
    assert!(game.talon.is_empty());

    game = GameRules::draw_stock(game, 1)?;

    assert_eq!(game.stock, vec![parse::card("#KC")]);
    assert_eq!(game.talon, vec![parse::card("AH")]);

    game = GameRules::draw_stock(game, 1)?;

    assert!(game.stock.is_empty());
    assert_eq!(game.talon, parse::cards(&vec!["AH", "KC"]));

    game = GameRules::draw_stock(game, 1)?;

    assert_eq!(game.stock, parse::cards(&vec!["#KC", "#AH"]));
    assert!(game.talon.is_empty());

    Ok(())
}

/// Test the sequence validation on its own for a tableau pile
#[test]
fn test_game_rules_valid_seq_tableau() {
    let p = PileRef::Tableau(0);

    let valid = parse::cards(&vec!["KC", "QH", "JS", "XD"]);
    assert!(GameRules::valid_seq(p, valid.as_slice()));

    let invalid_wrong_dir: Vec<_> = valid.iter().rev().cloned().collect();
    assert!(!GameRules::valid_seq(p, invalid_wrong_dir.as_slice()));

    let invalid_same_color = parse::cards(&vec!["8H", "7D", "6D"]);
    assert!(!GameRules::valid_seq(p, invalid_same_color.as_slice()));

    let invalid_overflow = parse::cards(&vec!["2C", "AH", "KS"]);
    assert!(!GameRules::valid_seq(p, invalid_overflow.as_slice()));
}

/// Test the sequence validation on its own for a foundation pile
#[test]
fn test_game_rules_valid_seq_foundation() {
    let p = PileRef::Foundation(0);

    let valid = parse::cards(&vec!["XC", "JC", "QC", "KC"]);
    assert!(GameRules::valid_seq(p, valid.as_slice()));

    let invalid_wrong_dir: Vec<_> = valid.iter().rev().cloned().collect();
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

    let pile = PileRef::Tableau(0);

    assert_eq!(
        GameRules::move_cards(game.clone(), pile, 0, pile).err(),
        Some(Error::InvalidInput {
            field: "take_n",
            reason: "cannot take 0 cards"
        })
    );

    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Stock, 1, PileRef::Tableau(0)),
        Err(Error::InvalidInput {
            field: "src",
            reason: "cannot move cards from stock"
        })
    );

    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Talon, 2, PileRef::Tableau(0)),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "cannot move more than 1 card from talon"
        })
    );

    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Tableau(0), 2, PileRef::Foundation(0)),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "cannot move more than 1 card to foundation"
        })
    );

    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Tableau(0), 1, PileRef::Stock),
        Err(Error::InvalidInput {
            field: "dst",
            reason: "cannot move cards to stock"
        })
    );

    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Tableau(0), 1, PileRef::Talon),
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
    let stock = parse::cards(&vec!["#KC", "#AH"]);
    let tableau0 = parse::cards(&vec!["2S"]);
    let tableau1 = parse::cards(&vec!["6H", "3S"]);
    let tableau2 = parse::cards(&vec!["#2H", "AC"]);

    let mut game = PlayingGameState {
        tableau: [
            tableau0,
            tableau1,
            tableau2,
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
        GameRules::move_cards(game.clone(), PileRef::Talon, 1, PileRef::Tableau(0)),
        Err(Error::InvalidInput {
            field: "take_n",
            reason: "not enough cards in src pile"
        })
    );

    // Draw so the card in the stock is available in the talon
    game = GameRules::draw_stock(game, 1)?;

    // Ace of Hearts in talon to empty space
    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Talon, 1, PileRef::Tableau(3)),
        Err(Error::InvalidMove {
            reason: "can only move a King to a space"
        })
    );

    // 2 of Spades to foundation
    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Tableau(0), 1, PileRef::Foundation(0)),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    // Ace of Hearts in talon to 2 of Spades in tableau
    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Talon, 1, PileRef::Tableau(1)),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    // 6 of Hearts and 3 of Spades to 2 of Spades
    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Tableau(1), 2, PileRef::Tableau(0)),
        Err(Error::InvalidMove {
            reason: "src sequence is invalid"
        })
    );

    // Hidden 2 of Hearts and Ace of Clubs to 3 of Spades
    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Tableau(2), 2, PileRef::Tableau(1)),
        Err(Error::InvalidMove {
            reason: "src sequence is invalid"
        })
    );

    // 3 of Spades to 2 of Spades
    assert_eq!(
        GameRules::move_cards(game.clone(), PileRef::Tableau(1), 1, PileRef::Tableau(0)),
        Err(Error::InvalidMove {
            reason: "dst sequence is invalid"
        })
    );

    Ok(())
}

/// Test moving cards around
#[test]
#[allow(unused_variables, unused_braces)]
fn test_game_rules_move_cards() -> Result<()> {
    let stock = parse::cards(&vec!["#KC", "#AH"]);
    let tableau0 = parse::cards(&vec!["#4D", "2S"]);
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

    macro_rules! test_move_and_auto {
        (
            GameRules::move_cards($src:tt, $take_n:literal, $dst:expr);
            $asserts:block
        ) => {
            let game_clone = game.clone();

            game = match GameRules::move_cards(game_clone.clone(), $src, $take_n, $dst)? {
                MoveResult::Playing(new) => new,
                MoveResult::Win(_) => panic!(),
            };
            $asserts

            game = match GameRules::auto_move_card(game_clone.clone(), $src, $take_n)? {
                MoveResult::Playing(new) => new,
                MoveResult::Win(_) => panic!(),
            };
            $asserts
        };
    }

    // Move the Ace of Hearts to the first tableau with a hidden and a 2 of Spades
    // (can't test auto here as it would put it straight into the foundation)
    game = match GameRules::move_cards(game, PileRef::Talon, 1, PileRef::Tableau(0))? {
        MoveResult::Playing(new) => new,
        MoveResult::Win(_) => panic!(),
    };

    // Talon is now empty
    assert!(game.talon.is_empty());
    // Tableau is a hidden card, 2 of Spades and Ace of Hearts
    assert_eq!(game.tableau[0], parse::cards(&vec!["#4D", "2S", "AH"]));

    test_move_and_auto! {
        // Move the stack to the second tableau with a 3 of Diamonds
        GameRules::move_cards({PileRef::Tableau(0)}, 2, {PileRef::Tableau(1)});
        {
            // First tableau is now the (face up) 4 of diamonds
            assert_eq!(game.tableau[0], vec![parse::card("4D")]);
            // Second tableau is the 3 of Diamonds, 2 of Spades and Ace of Hearts
            assert_eq!(game.tableau[1], parse::cards(&vec!["3D", "2S", "AH"]));
        }
    }

    test_move_and_auto! {
        // Move the Ace of Hearts to the foundation
        GameRules::move_cards({PileRef::Tableau(1)}, 1, {PileRef::Foundation(0)});
        {
            // Tableau is the 3 of Diamonds and 2 of Spades
            assert_eq!(game.tableau[1], parse::cards(&vec!["3D", "2S"]));
            // Foundation is the Ace of Hearts
            assert_eq!(game.foundations[0], vec![parse::card("AH")]);
        }
    }

    // Draw so the King of Clubs is available
    game = GameRules::draw_stock(game, 1)?;

    test_move_and_auto! {
        // Move the King of Clubs to the third tableau which is empty
        GameRules::move_cards({PileRef::Talon}, 1, {PileRef::Tableau(2)});
        {
            // Talon is now empty
            assert!(game.talon.is_empty());
            // Third tableau is King of Clubs
            assert_eq!(game.tableau[2], vec![parse::card("KC")]);
        }
    }

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
    let win = match GameRules::move_cards(game, PileRef::Tableau(0), 1, PileRef::Foundation(1))? {
        MoveResult::Playing(_) => panic!(),
        MoveResult::Win(new) => new,
    };

    for foundation in win.foundations {
        assert_eq!(foundation.len(), Rank::N);
    }

    Ok(())
}
