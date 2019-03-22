use std::ops::{Div, Sub};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Fee<T>(pub T);

impl<T: Div<Output = T>> Div for Fee<T> {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        Fee::<T>(self.0.div(other.0))
    }
}

impl<T: Sub<Output = T>> Sub for Fee<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Fee::<T>(self.0.sub(other.0))
    }
}
