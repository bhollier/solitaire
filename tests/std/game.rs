use solitaire::std;
use solitaire::*;

#[test]
fn test_game_rules_deal() -> Result<()> {
    Ok({
        let deck: std::Deck = new_deck();
        let mut game = std::GameState::new(&deck);
        std::GameRules::deal_mut(&mut game)?;

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
        assert_eq!(game.stock, expected_stock)
    })
}
