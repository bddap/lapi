use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Satoshis(pub u64);

impl Satoshis {
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn from_pico_btc(pico: u64) -> Result<Self, NotDivisible> {
        if pico % 10_000 == 0 {
            Ok(Self(pico / 10_000))
        } else {
            Err(NotDivisible {})
        }
    }

    pub fn checked_to_pico_btc(self) -> Option<u64> {
        self.0.checked_mul(10_000)
    }

    pub fn checked_to_i64(self) -> Option<i64> {
        debug_assert_eq!(i64::max_value(), 9223372036854775807i64);
        if self.0 > 9223372036854775807u64 {
            None
        } else {
            Some(self.0 as i64)
        }
    }
}

/// When converting from pico-btc to satoshi, pico-btc was not a multiple of 10000.
#[derive(Debug, Clone)]
pub struct NotDivisible;

impl Div for Satoshis {
    type Output = Self;
    fn div(self, other: Self) -> Satoshis {
        Satoshis(self.0.div(other.0))
    }
}

impl Sub for Satoshis {
    type Output = Self;
    fn sub(self, other: Self) -> Satoshis {
        Satoshis(self.0.sub(other.0))
    }
}

impl Mul for Satoshis {
    type Output = Self;
    fn mul(self, other: Satoshis) -> Satoshis {
        Satoshis(self.0.mul(other.0))
    }
}

impl Add for Satoshis {
    type Output = Self;
    fn add(self, other: Self) -> Satoshis {
        Satoshis(self.0.add(other.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn serde_as_expected() {
        assert_eq!(json!(9023), to_value(Satoshis(9023)).unwrap());
        assert_eq!(Satoshis(23235), from_value(json!(23235)).unwrap());
        from_value::<Satoshis>(json!("534")).unwrap_err();
        from_value::<Satoshis>(json!([1, 2, 2, 2])).unwrap_err();
    }
}
