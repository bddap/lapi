use rand::Rng;
use sha2::{digest::FixedOutput, Digest, Sha256};

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub struct U256([u8; 32]);

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
}
