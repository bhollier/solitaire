use ::std::collections::HashSet;
use solitaire::std;
use solitaire::*;

#[test]
fn test_new_deck() {
    let d: std::Deck = std::Card::new_deck();
    assert_eq!(d.len(), 52);

    let distinct_cards = d.iter().collect::<HashSet<_>>();
    assert_eq!(distinct_cards.len(), 52);
}

#[test]
fn test_take_n() {
    let d: std::Deck = std::Card::new_deck();
    {
        let (cs, rest) = take_n_slice(d.as_slice(), 10);
        assert_eq!(rest, &d[..42]);
        assert_eq!(cs, &d[42..52]);
    }

    {
        let (cs, rest) = take_n_vec(&d.iter().collect(), 10);
        assert_eq!(rest, d[..42].iter().collect::<std::Stack>());
        assert_eq!(cs, d[42..52].iter().collect::<std::Stack>());
    }

    {
        let mut d_vec: std::Stack = d.iter().collect();
        let cs = take_n_vec_mut(&mut d_vec, 10);
        assert_eq!(d_vec, d[..42].iter().collect::<std::Stack>());
        assert_eq!(cs, d[42..52].iter().collect::<std::Stack>());
    }
}

#[test]
fn test_take_one() {
    let d: std::Deck = std::Card::new_deck();
    {
        let (c, rest) = take_one_slice(d.as_slice());
        assert_eq!(rest, &d[..51]);
        assert_eq!(c, &d[51]);
    }

    {
        let (c, rest) = take_one_vec(&d.iter().collect());
        assert_eq!(rest, d[..51].iter().collect::<std::Stack>());
        assert_eq!(c, &d[51]);
    }

    {
        let mut d_vec: std::Stack = d.iter().collect();
        let c = take_one_vec_mut(&mut d_vec);
        assert_eq!(d_vec, d[..51].iter().collect::<std::Stack>());
        assert_eq!(c, &d[51]);
    }
}
