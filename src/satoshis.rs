#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Satoshis(u64);

impl Satoshis {
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
    
    pub fn from_pico_btc(pico: u64) -> Self {
        Self(pico / 10000)
    }
}

pub struct Withdrawal(pub Satoshis);
