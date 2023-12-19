use solitaire::std;
use solitaire::*;

#[test]
fn test_take_n() {
    let d: std::Deck = std::Card::new_deck();
    {
        let (rest, cs) = take_n_slice(&d, 10);
        assert_eq!(rest, &d[..42]);
        assert_eq!(cs, &d[42..52]);
    }

    {
        let (rest, cs) = take_n_vec(&Stack::from_slice(&d), 10);
        assert_eq!(rest, d[..42].iter().collect::<std::Stack>());
        assert_eq!(cs, d[42..52].iter().collect::<std::Stack>());
    }

    {
        let mut d_vec = Stack::from_slice(&d);
        let cs = take_n_vec_mut(&mut d_vec, 10);
        assert_eq!(d_vec, d[..42].iter().collect::<std::Stack>());
        assert_eq!(cs, d[42..52].iter().collect::<std::Stack>());
    }
}

#[test]
fn test_take_one() {
    let d: std::Deck = std::Card::new_deck();
    {
        let (rest, c) = take_one_slice(&d);
        assert_eq!(rest, &d[..51]);
        assert_eq!(c, &d[51]);
    }

    {
        let (rest, c) = take_one_vec(&Stack::from_slice(&d));
        assert_eq!(rest, d[..51].iter().collect::<std::Stack>());
        assert_eq!(c, &d[51]);
    }

    {
        let mut d_vec: std::Stack = Stack::from_slice(&d);
        let c = take_one_vec_mut(&mut d_vec);
        assert_eq!(d_vec, d[..51].iter().collect::<std::Stack>());
        assert_eq!(c, &d[51]);
    }
}
