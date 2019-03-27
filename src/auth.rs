//! Levels of authentication are hierarchical.
//! Master > Middle > Lesser
//!
//! Lesser == hash(Middle) == hash(hash(Master))
//!
//! iff hash(Middle(mi)) == Lesser(le) then mi can veiw the balance
//! of funds from le's invoices.

use crate::common::U256;
use serde::{Deserialize, Serialize};

/// Master can send funds.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct Master(pub U256);

/// Middle can veiw balance
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct Middle(pub U256);

/// Lesser is not secret. It can generate an invoice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub struct Lesser(pub U256);

/// Anyone can check invoice status if they have the 256 bit payment-hash for the invoice.

impl Master {
    pub fn random() -> Master {
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
