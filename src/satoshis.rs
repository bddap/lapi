#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Satoshis(u64);

impl Satoshis {
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

pub struct Withdrawal(pub Satoshis);
