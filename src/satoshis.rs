#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Satoshis(pub u64);

impl Satoshis {
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
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
}

pub struct Withdrawal(pub Satoshis);

/// Returned when converting from pico-btc to satoshi when pico-btc is not a multiple of 10000.
#[derive(Debug, Clone)]
pub struct NotDivisible;

// TODO review for overflow errs
