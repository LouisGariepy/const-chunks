//! This crate provides an extension trait that lets you chunk iterators into constant-length arrays using `const` generics.
//!
//! See the [`IteratorConstChunks::const_chunks`] docs for more info.
//!
//! ```rust
//! use const_chunks::IteratorConstChunks;
//!
//! let v = vec![1, 2, 3, 4, 5, 6];
//! let mut v_iter = v.into_iter().const_chunks::<2>();
//! assert_eq!(v_iter.next(), Some([1,2]));
//! assert_eq!(v_iter.next(), Some([3,4]));
//! assert_eq!(v_iter.next(), Some([5,6]));
//! ```

use std::mem::MaybeUninit;

/// An iterator that iterates over constant-length chunks, where the length is known at compile time.
///
/// This struct is created by the [`IteratorConstChunks::const_chunks`]. See its documentation for more.
pub struct ConstChunks<const N: usize, I: Iterator> {
    /// The inner iterator from which we take chunks.
    inner: I,
}

impl<const N: usize, I: Iterator> Iterator for ConstChunks<N, I> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            // Create array of unitialized values
            let mut arr: [MaybeUninit<I::Item>; N] = MaybeUninit::uninit().assume_init();

            // Initialize items
            for i in 0..N {
                // Get the next iterator item
                let Some(val) = self.inner.next() else {
                    // If there wasn't enough items to fill the array then
                    // manually drop the initialized values
                    for v in &mut arr[..i] {
                        v.assume_init_drop();
                    }
                    // early-return None.
                    return None;
                };

                // Initialize item
                arr[i].write(val);
            }

            // Cast to an array of definitely initialized `I::Item`s
            // TODO: use `array_assume_init` when stabilized.
            let init_arr = (&arr as *const _ as *const [I::Item; N]).read();

            Some(init_arr)
        }
    }
}

/// An extension trait providing [`Iterator`]s with the capability to iterate
/// over chunks of items.
pub trait IteratorConstChunks {
    /// The type of iterator from which we take chunks.
    type Inner: Iterator;

    /// This function returns an iterator over constant-length chunks of items, where
    /// the length is provided as a const-generic. This function assumes that the number
    /// of items can be divided into an integer number of chunks. If there are not
    /// enough items to fully fill a chunk, then the items are consumed, but no chunk
    /// will be yielded.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
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
    /// ```
    /// use const_chunks::IteratorConstChunks;
    ///
    /// let v = (1..=5).map(|n| n.to_string()).collect::<Vec<String>>(); // Five items cannot fit into chunks of length 2!
    /// let mut v_iter = v.into_iter().const_chunks::<2>();
    /// assert_eq!(v_iter.next(), Some([String::from("1"),String::from("2")]));
    /// assert_eq!(v_iter.next(), Some([String::from("3"),String::from("4")]));
    /// assert_eq!(v_iter.next(), None); // `None`, even though there was still one item
    /// ```
    fn const_chunks<const N: usize>(self) -> ConstChunks<N, Self::Inner>;
}

/// Blanket implementation over all iterators.
impl<I: Iterator> IteratorConstChunks for I {
    type Inner = Self;

    fn const_chunks<const N: usize>(self) -> ConstChunks<N, Self::Inner> {
        ConstChunks { inner: self }
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
}
