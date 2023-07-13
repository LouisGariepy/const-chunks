//! This crate provides an extension trait that lets you chunk iterators into constant-length arrays using `const` generics.
//!
//! See the [`IteratorConstChunks::const_chunks`] docs for more info.
//!
//! ```rust
//! use const_chunks::IteratorConstChunks;
//!
//! let mut iter = vec![1, 2, 3, 4, 5].into_iter().const_chunks::<2>();
//! assert_eq!(iter.next(), Some([1,2]));
//! assert_eq!(iter.next(), Some([3,4]));
//! assert_eq!(iter.next(), None);
//!
//! let mut remainder = iter.into_remainder().unwrap();
//! assert_eq!(remainder.next(), Some(5));
//! assert_eq!(remainder.next(), None);
//! ```

#![cfg_attr(not(test), no_std)]

mod panic_guard;
mod remainder;

use core::mem::{forget, MaybeUninit};

use panic_guard::ChunkPanicGuard;
use remainder::ConstChunksRemainder;

/// An iterator that iterates over constant-length chunks, where the length is known at compile time.
///
/// This struct is created by the [`IteratorConstChunks::const_chunks`] method. See its documentation for more.
pub struct ConstChunks<const N: usize, I: Iterator> {
    /// The inner iterator from which we take chunks.
    inner: I,
    /// The remainder that couldn't fill a chunk completely.
    ///
    /// This field is None if the underlying iterator hasn't been completely consumed
    /// or if there are no remaining items.
    remainder: Option<ConstChunksRemainder<N, I::Item>>,
}

impl<const N: usize, I: Iterator> ConstChunks<N, I> {
    /// This asserts a const-time that `N` is non-zero.
    /// This is useful to prevent accidental bugs, but
    /// this also acts as a safety invariant.
    const N_GT_ZERO: () = assert!(N > 0, "chunk size must be non-zero");

    /// Consumes self and returns the remainder that could not fill a chunk completely.
    ///
    /// # Usage
    ///
    /// ```rust
    /// use const_chunks::IteratorConstChunks;
    ///
    /// let mut v_iter = vec![1, 2, 3, 4, 5, 6].into_iter().const_chunks::<4>();
    ///
    /// // Collect chunks
    /// let chunks = (&mut v_iter).collect::<Vec<_>>();
    /// assert_eq!(chunks, vec![[1, 2, 3, 4]]);
    ///
    /// // Collect remainder
    /// let remainder = v_iter.into_remainder().unwrap().collect::<Vec<_>>();
    /// assert_eq!(remainder, vec![5, 6]);
    /// ```
    pub fn into_remainder(self) -> Option<ConstChunksRemainder<N, I::Item>> {
        self.remainder
    }
}

impl<const N: usize, I: Iterator> Iterator for ConstChunks<N, I> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        // Early return if the underlying iterator is empty
        let Some(first_item) = self.inner.next() else {
            return None;
        };

        // Create array of unitialized values
        //
        // SAFETY: The `assume_init` is sound because `MaybeUninit`s do not require initialization.
        let mut array: [MaybeUninit<I::Item>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        // Create panic guard
        let mut guard = ChunkPanicGuard {
            slice: &mut array,
            initialized: 0,
        };
        // SAFETY: We enforce N > 0 at compile-time, so it's sound to assume at least one item.
        unsafe { guard.init_next_unchecked(first_item) };

        // Initialize remaining items
        for i in 1..N {
            let Some(item) = self.inner.next() else {
                    // Disarm panic guard. `ConstChunksRemainder` will
                    // handle the partially initialized array.
                    forget(guard);

                    // Set remainder
                    self.remainder = Some(ConstChunksRemainder {
                        remainder_chunk: array,
                        init_range: 0..i
                    });

                    // No more chunks
                    return None;
                };
            // SAFETY: Will be called at most N times (including the initial
            // `init_next_unchecked` call before the loop)
            unsafe { guard.init_next_unchecked(item) };
        }

        // Disarm panic guard. At this point all the items are initialized
        // and we're about to get rid of the `MaybeUninit`s.
        forget(guard);

        // Cast to an array of definitely initialized items
        //
        // SAFETY: If we've reached this point, all the items in the chunk have been initialized.
        //
        // TODO: use `array_assume_init` when stabilized.
        let init_arr = unsafe { (&array as *const _ as *const [I::Item; N]).read() };

        Some(init_arr)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.inner.size_hint();
        (lower / N, upper.map(|upper| upper / N))
    }
}

impl<const N: usize, I: ExactSizeIterator> ExactSizeIterator for ConstChunks<N, I> {
    fn len(&self) -> usize {
        self.inner.len() / N
    }
}

/// An extension trait providing [`Iterator`]s with the capability to iterate
/// over const-sized arrays of items.
pub trait IteratorConstChunks {
    /// The type of iterator from which we take chunks.
    type Inner: Iterator;

    /// This function returns an iterator over constant-length chunks of items, where
    /// the length is provided as a const-generic.
    ///
    /// # Usage
    ///
    /// ```rust
    /// use const_chunks::IteratorConstChunks;
    ///
    /// let v = vec![1, 2, 3, 4, 5, 6];
    /// let mut v_iter = v.into_iter().const_chunks::<2>();
    /// assert_eq!(v_iter.next(), Some([1,2]));
    /// assert_eq!(v_iter.next(), Some([3,4]));
    /// assert_eq!(v_iter.next(), Some([5,6]));
    /// ```
    ///
    /// When the number of items in the iterator cannot be divided exactly
    /// into chunks, then the iterator will be fully consumed, but the last
    /// chunk will not be yielded.
    ///
    /// ```rust
    /// use const_chunks::IteratorConstChunks;
    ///
    /// // Five items cannot fit into chunks of length 2!
    /// let v = [1, 2, 3, 4, 5];
    ///
    /// let mut v_iter = v.into_iter().const_chunks::<2>();
    /// assert_eq!(v_iter.next(), Some([1, 2]));
    /// assert_eq!(v_iter.next(), Some([3, 4]));
    ///
    /// // `None`, even though there was still one item
    /// assert_eq!(v_iter.next(), None);
    /// ```
    ///
    /// To get the remaining items, you can use the [`ConstChunks::into_remainder`] method (see for more).
    ///
    /// Note that trying to build chunks of size 0 will fail to compile:
    ///
    // TODO: Workaround until MIRI can catch const-eval panics as compilation errors (https://github.com/rust-lang/miri/issues/2423).
    #[cfg_attr(miri, doc = "```should_panic")]
    #[cfg_attr(not(miri), doc = "```compile_fail,E0080")]
    /// use const_chunks::IteratorConstChunks;
    ///
    /// let _ = vec![1, 2].into_iter().const_chunks::<0>();
    /// ```
    ///
    /// You should get an error similar to this one:
    /// ```text
    ///     |     const N_GT_ZERO: () = assert!(N > 0, "chunk size must be non-zero");
    ///     |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'chunk size must be non-zero'
    /// ```
    fn const_chunks<const N: usize>(self) -> ConstChunks<N, Self::Inner>;
}

/// Blanket implementation over all [`Iterator`]s.
impl<I: Iterator> IteratorConstChunks for I {
    type Inner = Self;

    fn const_chunks<const N: usize>(self) -> ConstChunks<N, Self::Inner> {
        // Assert N > 0 (see `ConstChunks::N_GT_ZERO`)
        #[allow(clippy::let_unit_value)]
        let _ = ConstChunks::<N, Self::Inner>::N_GT_ZERO;

        ConstChunks {
            inner: self,
            remainder: None,
        }
    }
}

/// Drops all the initialized items in the underlying array.
///
/// # Safety
///
/// The slice must contain only initialized objects.
unsafe fn drop_slice<T>(slice: &mut [MaybeUninit<T>]) {
    for init in slice {
        init.assume_init_drop();
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use crate::IteratorConstChunks;

    #[test]
    fn test_panic_leak() {
        // Setup an iterator that can panic on `next`.
        struct PanicIter<I: Iterator> {
            inner: I,
        }
        impl<I: Iterator> Iterator for PanicIter<I> {
            type Item = I::Item;

            fn next(&mut self) -> Option<Self::Item> {
                // Causes a panic when the inner iterator is exhausted
                Some(self.inner.next().unwrap())
            }
        }
        let panic_iter = PanicIter {
            inner: [String::from("1")].into_iter(),
        };

        // Catch the panic to try to cause a leak
        let _ = catch_unwind(|| panic_iter.const_chunks::<4>().collect::<Vec<_>>());
    }

    #[test]
    fn test_exhausted() {
        // Five items cannot fit into chunks of length 2!
        let mut v_iter = (1..=5).map(|n| n.to_string()).const_chunks::<2>();
        assert_eq!(v_iter.next(), Some([1, 2].map(|n| n.to_string())));
        assert_eq!(v_iter.next(), Some([3, 4].map(|n| n.to_string())));

        // Assert iterator is exhausted.
        assert_eq!(v_iter.next(), None);
    }

    #[test]
    fn test_remainder() {
        let v = vec![1, 2, 3, 4, 5, 6];
        let mut v_iter = v.into_iter().const_chunks::<4>();
        let chunks = (&mut v_iter).collect::<Vec<_>>();
        let remainder = v_iter.into_remainder().unwrap().collect::<Vec<_>>();
        assert_eq!(chunks, vec![[1, 2, 3, 4]]);
        assert_eq!(remainder, vec![5, 6]);
    }

    #[test]
    fn test_remainder_leak() {
        let mut v_iter = (1..=6).map(|n| n.to_string()).const_chunks::<4>();
        // Exhaust iterator.
        let _ = (&mut v_iter).collect::<Vec<_>>();
        // Assert iterator is exhausted.
        assert_eq!(v_iter.next(), None);

        // Get remainder
        let mut remainder = v_iter.into_remainder().unwrap();
        // Fetch the next value out of the remainder
        assert_eq!(remainder.next(), Some(5.to_string()));
        drop(remainder);
    }
}
