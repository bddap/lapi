//! Levels of authentication are hierarchical.
//! Master > Middle > Lesser
//!
//! Lesser == hash(Middle) == hash(hash(Master))
//!
//! iff hash(Middle(mi)) == Lesser(le) then mi can veiw the balance
//! of funds from le's invoices.

use crate::common::U256;
use core::{fmt, str::FromStr};
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

impl From<Master> for Lesser {
    fn from(other: Master) -> Self {
        let middle: Middle = other.into();
        middle.into()
    }
}

macro_rules! derive_from_str {
    ($typ:ty, $constructor:expr) => {
        impl FromStr for $typ {
            type Err = <U256 as FromStr>::Err;
            fn from_str(other: &str) -> Result<Self, <Self as FromStr>::Err> {
                U256::from_str(other).map($constructor)
            }
        }
    };
}

derive_from_str!(Master, Master);
derive_from_str!(Middle, Middle);
derive_from_str!(Lesser, Lesser);

macro_rules! derive_display {
    ($typ:ty) => {
        impl fmt::Display for $typ {
            fn fmt(&self, fm: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                fmt::Display::fmt(&self.0, fm)
            }
        }
    };
}

derive_display!(Master);
derive_display!(Middle);
derive_display!(Lesser);
