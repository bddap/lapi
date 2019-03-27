use core::str::FromStr;
use hex::{decode, encode, FromHexError};
use rand::Rng;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{digest::FixedOutput, Digest, Sha256};
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub struct U256(pub [u8; 32]);

impl U256 {
    pub fn try_from_slice(inp: &[u8]) -> Option<U256> {
        if inp.len() != 32 {
            None
        } else {
            let mut ar: [u8; 32] = [0u8; 32];
            ar.copy_from_slice(inp);
            Some(U256(ar))
        }
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn random() -> U256 {
        U256(rand::thread_rng().gen()) // TODO, make sure this is secure
    }

    pub fn hash(&self) -> U256 {
        let mut hasher = Sha256::new();
        hasher.input(self.0);
        let result = hasher.fixed_result();
        let result_slice = result.as_slice();
        U256::try_from_slice(&result).unwrap()
    }

    pub fn zero() -> U256 {
        U256([0; 32])
    }

    pub fn from_array(other: [u8; 32]) -> Self {
        U256(other)
    }
}

impl Display for U256 {
    fn fmt(&self, fmtr: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(fmtr, "{}", encode(&self.0))
    }
}

impl Serialize for U256 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for U256 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = <Cow<str>>::deserialize(deserializer)?;
        <U256 as FromStr>::from_str(&hex).map_err(|err| match err {
            U256ParseError::LenNot64 { got } => de::Error::invalid_length(got, &"64"),
            U256ParseError::Hex(err) => de::Error::custom(format!("{}", err)),
        })
    }
}

impl FromStr for U256 {
    type Err = U256ParseError;
    fn from_str(other: &str) -> Result<Self, <Self as FromStr>::Err> {
        if other.len() != 64 {
            return Err(U256ParseError::LenNot64 { got: other.len() });
        }
        let vect: Vec<u8> = decode(other).map_err(U256ParseError::Hex)?;
        debug_assert_eq!(std::mem::size_of::<U256>(), 32);
        Ok(U256::try_from_slice(&vect).expect("checked above"))
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum U256ParseError {
    Hex(FromHexError),
    LenNot64 { got: usize },
}
