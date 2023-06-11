use std::{mem::MaybeUninit, ops::Range};

use crate::drop_slice;

/// Iterator over the remaining items that couldn't fill a chunk completely.
///
/// See [`ConstChunks::into_remainder`] for more details.
pub struct ConstChunksRemainder<const N: usize, T> {
    /// The array to be initialized.
    pub(crate) remainder_chunk: [MaybeUninit<T>; N],
    /// The range of initialized items.
    // SAFETY: Should always be included in 0..(N-1). A slice of `init_range`
    // into `remainder_chunk` should be fully initialized.
    pub(crate) init_range: Range<usize>,
}

impl<const N: usize, T> Iterator for ConstChunksRemainder<N, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        (!self.init_range.is_empty()).then(|| {
            // Get the next initialized item from the array.
            // This replaces the initialized item with an uninitialized value.
            //
            // SAFETY: `self.init_range.start` is in bounds and points to an initialized item.
            let next_init = unsafe {
                std::mem::replace(
                    self.remainder_chunk
                        .get_unchecked_mut(self.init_range.start),
                    MaybeUninit::uninit(),
                )
                .assume_init()
            };

            // Increment the initialized range start.
            self.init_range.start += 1;

            next_init
        })
    }
}

impl<const N: usize, T> Drop for ConstChunksRemainder<N, T> {
    fn drop(&mut self) {
        // SAFETY: The slice contains only initialized objects.
        unsafe { drop_slice(&mut self.remainder_chunk[self.init_range.clone()]) }
    }
}
