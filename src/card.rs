use rand;
use rand::seq::SliceRandom;
use std::cmp::Ordering;
use std::slice::Iter;

/// The Suit of a [Card], e.g. Clubs, Hearts, etc.
pub trait Suit: Sized + Copy + Eq {
    /// An iterator of all the valid [Suit]s a [Card] can have
    fn iter() -> Iter<'static, Self>;
}

/// The Rank of a [Card]
pub trait Rank: Sized + Copy + Eq + Ord {
    /// An iterator of all the valid [Rank]s a [Card] can have
    fn iter() -> Iter<'static, Self>;
}

/// A solitaire Card. Ord is implemented but only acts only on the card's [Rank]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Card<S: Suit, R: Rank> {
    pub suit: S,
    pub rank: R,
}

impl<S: Suit, R: Rank> Ord for Card<S, R> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl<S: Suit, R: Rank> PartialOrd<Self> for Card<S, R> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rank.partial_cmp(&other.rank)
    }
}

/// A Deck of [Card]s
pub type Deck<S, R> = Vec<Card<S, R>>;

/// A "Stack" of [Card] references, usually the [Card]s referencing the elements of a [Deck]
pub type Stack<'a, S, R> = Vec<&'a Card<S, R>>;

/// Creates a new deck of all the possible cards, using [Suit::iter] and [Rank::iter]
pub fn new_deck<S: Suit + 'static, R: Rank + 'static>() -> Deck<S, R> {
    S::iter()
        .flat_map(|s| R::iter().map(|r| Card { suit: *s, rank: *r }))
        .collect()
}

/// Shuffles the given deck mutably, using [rand::thread_rng()]
pub fn shuffle<S: Suit, R: Rank>(d: &mut Deck<S, R>) {
    shuffle_with_rng(d, &mut rand::thread_rng())
}

/// Shuffles the given deck mutably, using the given [rand::Rng]
pub fn shuffle_with_rng<S: Suit, R: Rank, RNG: rand::Rng>(d: &mut Deck<S, R>, r: &mut RNG) {
    d.shuffle(r)
}

/// Returns two slices from the given slice, as a tuple of the `(taken, remaining)`
pub fn take_n_slice<T>(slice: &[T], n: usize) -> (&[T], &[T]) {
    (&slice[slice.len() - n..], &slice[0..slice.len() - n])
}

/// Creates two [Vec] from the given [Vec], as a tuple of the `(taken, remaining)`
pub fn take_n_vec<'a, T>(cs: &Vec<&'a T>, n: usize) -> (Vec<&'a T>, Vec<&'a T>) {
    let (cs, rest) = take_n_slice(cs.as_slice(), n);
    (
        cs.iter().map(|c| *c).collect(),
        rest.iter().map(|c| *c).collect(),
    )
}

/// Creates `n` elements from the given [Vec] and returns it,
/// modifying the given [Vec] in the process
pub fn take_n_vec_mut<'a, T>(cs: &mut Vec<&'a T>, n: usize) -> Vec<&'a T> {
    cs.split_off(cs.len() - n)
}

/// Returns the "top" card and a slice of the remaining elements as a tuple
pub fn take_one_slice<T>(cs: &[T]) -> (&T, &[T]) {
    let (cs, rest) = take_n_slice(cs, 1);
    (&cs[0], rest)
}

/// Returns the "top" card and a [Vec] of the remaining elements
pub fn take_one_vec<'a, T>(cs: &Vec<&'a T>) -> (&'a T, Vec<&'a T>) {
    let (cs, rest) = take_n_slice(cs.as_slice(), 1);
    (cs[0], rest.iter().map(|c| *c).collect())
}

/// Returns the "top" card and removes the element from the given [Vec]
pub fn take_one_vec_mut<'a, T>(cs: &mut Vec<&'a T>) -> &'a T {
    cs.pop().unwrap()
}
