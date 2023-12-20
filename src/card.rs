use std::hash::Hash;

use rand::seq::SliceRandom;

/// A solitaire Card. `N` is the total number of variations of Cards
pub trait Card<const N: usize>: Copy + Clone + Eq + Ord + Hash {
    /// Create a new (unshuffled) deck of Cards
    fn new_deck() -> Deck<Self, N>;
}

/// A Deck of [Card]s
pub type Deck<C, const N: usize> = [C; N];

/// A "Stack" of [Card]s
// todo change this to be backed by an array and benchmark
pub type Stack<C> = Vec<C>;

/// Trait extension for adding `from_*` functions to [Stack]
pub trait StackFrom<C: Card<N>, const N: usize> {
    /// Convenience function to create a [Stack] from a slice of `C` (usually a [Deck] or slice of [Card])
    fn from_slice(cs: &[C]) -> Stack<C> {
        cs.iter().cloned().collect()
    }
}

impl<C: Card<N>, const N: usize> StackFrom<C, N> for Stack<C> {}

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

/// Creates `n` elements from the given [Vec] and returns it,
/// modifying the given [Vec] in the process
pub fn take_n_vec_mut<T>(cs: &mut Vec<T>, n: usize) -> Vec<T> {
    cs.split_off(cs.len() - n)
}

/// Returns the "top" card and a slice of the remaining elements as a tuple
pub fn take_one_slice<T>(cs: &[T]) -> (&[T], &T) {
    let (rest, cs) = take_n_slice(cs, 1);
    (rest, &cs[0])
}

/// Returns the "top" card and removes the element from the given [Vec]
pub fn take_one_vec_mut<T>(cs: &mut Vec<T>) -> T {
    cs.pop().unwrap()
}
