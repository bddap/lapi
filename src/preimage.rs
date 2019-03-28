use crate::common::*;
use serde::{Deserialize, Serialize};

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Preimage(pub U256);
