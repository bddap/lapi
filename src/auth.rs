//! Levels of authentication are hierarchical.
//! Master > Middle > Lesser
//!
//! Lesser == hash(Middle) == hash(hash(Master))
//!
//! iff hash(Middle(mi)) == Lesser(le) then mi can veiw the balance
//! of funds from le's invoices.

use crate::common::U256 as Number;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

/// Master has total control. Master can send funds.
#[derive(Clone, Copy)]
pub struct Master(Number);

/// Middle can veiw balance, and check invoice status.
#[derive(Clone, Copy)]
pub struct Middle(Number);

/// Lesser is not secret. It can generate an invoice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lesser(Number);

impl Distribution<Master> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Master {
        Master(rng.gen())
    }
}

impl From<Master> for Middle {
    fn from(other: Master) -> Self {
        Self(hash(other.0))
    }
}

impl From<Middle> for Lesser {
    fn from(other: Middle) -> Self {
        Self(hash(other.0))
    }
}

fn hash(_n: Number) -> Number {
    unimplemented!()
}
