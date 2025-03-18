use std::ops::Deref;
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use thiserror::Error;

/// Length grabbing functions
pub trait Length {
    /// Gets the length
    fn len(&self) -> usize;
    /// Tells whether the length is 0
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Length for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Length for &'_ [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for &'_ mut [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T, const N: usize> Length for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Length for &'_ [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Length for &'_ mut [T; N] {
    fn len(&self) -> usize {
        N
    }
}

#[derive(Error, Debug)]
pub enum AdvanceError {
    #[error("Not enough data, needed: `{needed}`, remaining: `{remaining}`")]
    NotEnoughData { needed: usize, remaining: usize },
}

// TODO: impl this const when const traits stabilized.
/// Advances a given slice while maintaining lifetimes
pub trait Advance: Length {
    /// The element of the array
    type Element;
    /// The output of advancing
    type AdvanceOut<'a>: Deref<Target = [Self::Element]>
    where
        Self: 'a;

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Panics if not enough data.
    fn advance(&mut self, amount: usize) -> Self::AdvanceOut<'_> {
        assert!(amount <= self.len());
        // Safety: amount is not greater than the length of self
        unsafe { self.advance_unchecked(amount) }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Errors if not enough data.
    fn try_advance(&mut self, amount: usize) -> Result<Self::AdvanceOut<'_>, AdvanceError> {
        if self.len() < amount {
            Err(AdvanceError::NotEnoughData {
                needed: amount,
                remaining: self.len(),
            })
        } else {
            // Safety: amount is not greater than the length of self
            Ok(unsafe { self.advance_unchecked(amount) })
        }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `amount` is not greater than the length of self.
    unsafe fn advance_unchecked(&mut self, amount: usize) -> Self::AdvanceOut<'_>;
}

// TODO: impl this const when const traits stabilized.
/// Advances a given slice giving back an array
pub trait AdvanceArray: Length {
    /// The element of the array
    type Element;
    /// The output of advancing
    type AdvanceOut<'a, const N: usize>: Deref<Target = [Self::Element; N]>
    where
        Self: 'a;

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Panics if not enough data.
    fn advance_array<const N: usize>(&mut self) -> Self::AdvanceOut<'_, N> {
        assert!(N <= self.len());
        // Safety: N is not greater than the length of self
        unsafe { self.advance_array_unchecked() }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Errors if not enough data.
    fn try_advance_array<const N: usize>(
        &mut self,
    ) -> Result<Self::AdvanceOut<'_, N>, AdvanceError> {
        if self.len() < N {
            Err(AdvanceError::NotEnoughData {
                needed: N,
                remaining: self.len(),
            })
        } else {
            // Safety: N is not greater than the length of self
            Ok(unsafe { self.advance_array_unchecked() })
        }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `N` is not greater than the length of self.
    unsafe fn advance_array_unchecked<const N: usize>(&mut self) -> Self::AdvanceOut<'_, N>;
}

impl<T> Advance for &'_ mut [T] {
    type Element = T;
    type AdvanceOut<'a>
        = &'a mut [T]
    where
        Self: 'a;

    unsafe fn advance_unchecked(&mut self, amount: usize) -> Self::AdvanceOut<'_> {
        // Safety neither slice overlaps and points to valid r/w data
        let len = self.len();
        let ptr = self.as_mut_ptr();
        *self = &mut *slice_from_raw_parts_mut(ptr.add(amount), len - amount);
        &mut *slice_from_raw_parts_mut(ptr, amount)
    }
}

impl<T> AdvanceArray for &'_ mut [T] {
    type Element = T;
    type AdvanceOut<'a, const N: usize>
        = &'a mut [T; N]
    where
        Self: 'a;

    unsafe fn advance_array_unchecked<const N: usize>(&mut self) -> Self::AdvanceOut<'_, N> {
        // Safe conversion because returned array will always be same size as value passed in (`N`)
        &mut *(
            // Safety: Same requirements as this function
            self.advance_unchecked(N).as_mut_ptr().cast::<[T; N]>()
        )
    }
}

impl<T> Advance for &'_ [T] {
    type Element = T;
    type AdvanceOut<'a>
        = &'a [T]
    where
        Self: 'a;

    unsafe fn advance_unchecked(&mut self, amount: usize) -> Self::AdvanceOut<'_> {
        // Safety neither slice overlaps and points to valid r/w data
        let len = self.len();
        let ptr = self.as_ptr();
        *self = &*slice_from_raw_parts(ptr.add(amount), len - amount);
        &*slice_from_raw_parts(ptr, amount)
    }
}

impl<T> AdvanceArray for &'_ [T] {
    type Element = T;
    type AdvanceOut<'a, const N: usize>
        = &'a [T; N]
    where
        Self: 'a;

    unsafe fn advance_array_unchecked<const N: usize>(&mut self) -> Self::AdvanceOut<'_, N> {
        // Safe conversion because returned array will always be same size as value passed in (`N`)
        &*(
            // Safety: Same requirements as this function
            self.advance_unchecked(N).as_ptr().cast::<[T; N]>()
        )
    }
}
