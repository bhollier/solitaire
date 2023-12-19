use rand;
use rand::seq::SliceRandom;
use std::hash::Hash;

/// A solitaire Card. `N` is the total number of variations of Cards
pub trait Card<const N: usize>: Copy + Clone + Eq + Ord + Hash {
    /// Create a new (unshuffled) deck of Cards
    fn new_deck() -> Deck<Self, N>;
}

/// A Deck of [Card]s
pub type Deck<C, const N: usize> = [C; N];

/// A "Stack" of [Card] references, usually the [Card]s referencing the elements of a [Deck]
// todo change this to be backed by an array and benchmark
pub type Stack<'d, C> = Vec<&'d C>;

/// Trait extension for adding `from_*` functions to [Stack]
pub trait StackFrom<C> {
    /// Convenience function to create a [Stack] from a slice of `C` (usually a [Deck] or slice of [Card]).
    /// Intended to be used to construct from a deck, as a stack usually contains reference to cards within a deck
    fn from_slice(cs: &[C]) -> Stack<C> {
        cs.iter().collect()
    }

    /// Convenience function to create a [Stack] from a [Vec<C>]
    fn from_vec(cs: &Vec<C>) -> Stack<C> {
        Self::from_slice(cs.as_slice())
    }
}

impl<'d, C> StackFrom<C> for Stack<'d, C> {}

/// Shuffles the given deck mutably, using [rand::thread_rng()]
pub fn shuffle<C: Card<N>, const N: usize>(d: &mut Deck<C, N>) {
    shuffle_with_rng(d, &mut rand::thread_rng())
}

/// Shuffles the given deck mutably, using the given [rand::Rng]
pub fn shuffle_with_rng<C: Card<N>, const N: usize, RNG: rand::Rng>(
    d: &mut Deck<C, N>,
    r: &mut RNG,
) {
    d.shuffle(r)
}

/// Returns two slices from the given slice, as a tuple of the `(remaining, taken)`
pub fn take_n_slice<T>(slice: &[T], n: usize) -> (&[T], &[T]) {
    (&slice[0..slice.len() - n], &slice[slice.len() - n..])
}

/// Creates two [Vec] from the given [Vec], as a tuple of the `(remaining, taken)`
pub fn take_n_vec<'a, T>(cs: &Vec<&'a T>, n: usize) -> (Vec<&'a T>, Vec<&'a T>) {
    let (rest, cs) = take_n_slice(cs.as_slice(), n);
    (rest.iter().cloned().collect(), cs.iter().cloned().collect())
}

/// Creates `n` elements from the given [Vec] and returns it,
/// modifying the given [Vec] in the process
pub fn take_n_vec_mut<'a, T>(cs: &mut Vec<&'a T>, n: usize) -> Vec<&'a T> {
    cs.split_off(cs.len() - n)
}

/// Returns the "top" card and a slice of the remaining elements as a tuple
pub fn take_one_slice<T>(cs: &[T]) -> (&[T], &T) {
    let (rest, cs) = take_n_slice(cs, 1);
    (rest, &cs[0])
}

/// Returns the "top" card and a [Vec] of the remaining elements
pub fn take_one_vec<'a, T>(cs: &Vec<&'a T>) -> (Vec<&'a T>, &'a T) {
    let (rest, cs) = take_n_slice(cs.as_slice(), 1);
    (rest.iter().cloned().collect(), cs[0])
}

/// Returns the "top" card and removes the element from the given [Vec]
pub fn take_one_vec_mut<'a, T>(cs: &mut Vec<&'a T>) -> &'a T {
    cs.pop().unwrap()
}
