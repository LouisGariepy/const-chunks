use core::mem::MaybeUninit;

use crate::drop_slice;

/// This type acts as a guard that drops the currently initialized
/// items when itself is dropped. This prevents leaking memory when
/// a panic occurs during chunk initialization.
pub struct PanicGuard<'a, T> {
    /// The array being initialized.
    pub slice: &'a mut [MaybeUninit<T>],
    /// The number of items that have been initialized so far.
    pub initialized: usize,
}

impl<'a, T> PanicGuard<'a, T> {
    /// Initializes the next uninitialized item and updates the initialized item counter.
    ///
    /// # Safety
    ///
    /// This function causes undefined behaviour if called more times than the  slice's size.
    #[inline]
    pub unsafe fn init_next_unchecked(&mut self, item: T) {
        self.slice.get_unchecked_mut(self.initialized).write(item);
        self.initialized += 1;
    }
}

impl<'a, T> Drop for PanicGuard<'a, T> {
    /// Drops all the initialized items in the slice.
    fn drop(&mut self) {
        // SAFETY: The slice contains only initialized objects.
        unsafe { drop_slice(&mut self.slice[..self.initialized]) }
    }
}
