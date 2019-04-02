use crate::common::*;
use serde::{Deserialize, Serialize};

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Preimage(pub U256);

impl Preimage {
    pub fn hash(self) -> PaymentHash {
        self.0.hash()
    }
}
