//! Levels of authentication are hierarchical.
//! Master > Middle > Lesser
//!
//! Lesser == hash(Middle) == hash(hash(Master))
//!
//! iff hash(Middle(mi)) == Lesser(le) then mi can veiw the balance
//! of funds from le's invoices.

use crate::common::U256;

/// Master has total control. Master can send funds.
#[derive(Clone, Copy)]
pub struct Master(U256);

/// Middle can veiw balance, and check invoice status.
#[derive(Clone, Copy)]
pub struct Middle(U256);

/// Lesser is not secret. It can generate an invoice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lesser(U256);

impl Master {
    fn random() -> Master {
        Master(U256::random())
    }
}

impl From<Master> for Middle {
    fn from(other: Master) -> Self {
        Self(other.0.hash())
    }
}

impl From<Middle> for Lesser {
    fn from(other: Middle) -> Self {
        Self(other.0.hash())
    }
}
