use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{Error, Result};

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            m_cost_kib: 64 * 1024,
            t_cost: 3,
            p_cost: 1,
        }
    }
}

impl KdfParams {
    pub fn fast_for_tests() -> Self {
        Self {
            m_cost_kib: 8 * 1024,
            t_cost: 1,
            p_cost: 1,
        }
    }
}

pub fn random_salt() -> Result<[u8; SALT_LEN]> {
    let mut salt = [0u8; SALT_LEN];
    getrandom::fill(&mut salt).map_err(|e| Error::Kdf(e.to_string()))?;
    Ok(salt)
}

pub fn derive(
    password: &[u8],
    salt: &[u8; SALT_LEN],
    params: &KdfParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>> {
    let params = Params::new(
        params.m_cost_kib,
        params.t_cost,
        params.p_cost,
        Some(KEY_LEN),
    )
    .map_err(|e| Error::Kdf(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon
        .hash_password_into(password, salt, key.as_mut())
        .map_err(|e| Error::Kdf(e.to_string()))?;
    Ok(key)
}

pub fn to_hex_key(key: &[u8; KEY_LEN]) -> Zeroizing<String> {
    Zeroizing::new(hex::encode(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_key() {
        let salt = [7u8; SALT_LEN];
        let p = KdfParams::fast_for_tests();
        let a = derive(b"hunter2", &salt, &p).unwrap();
        let b = derive(b"hunter2", &salt, &p).unwrap();
        assert_eq!(*a, *b);
    }

    #[test]
    fn different_salt_different_key() {
        let p = KdfParams::fast_for_tests();
        let a = derive(b"hunter2", &[1u8; SALT_LEN], &p).unwrap();
        let b = derive(b"hunter2", &[2u8; SALT_LEN], &p).unwrap();
        assert_ne!(*a, *b);
    }

    #[test]
    fn different_password_different_key() {
        let salt = [7u8; SALT_LEN];
        let p = KdfParams::fast_for_tests();
        let a = derive(b"hunter2", &salt, &p).unwrap();
        let b = derive(b"hunter3", &salt, &p).unwrap();
        assert_ne!(*a, *b);
    }

    #[test]
    fn hex_key_is_64_lowercase_hex_chars() {
        let key = [0xABu8; KEY_LEN];
        let hex = to_hex_key(&key);
        assert_eq!(hex.len(), 64);
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn random_salts_differ() {
        assert_ne!(random_salt().unwrap(), random_salt().unwrap());
    }
}
