use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
#[derive(Serialize, Deserialize, Clone)]
/// This structure stores the parameters for decrypting the disk.
pub struct EncryptionParams {
    pub(crate) keyfile: EncryptedKeyfile,
    pub(crate) password_auth: PasswordAuthParameters,
    pub(crate) yubikey_auth: YubikeyAuthParams,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
/// This is the encrypted disk keyfile.
/// If you decrypt this successfully, you should be able to unlock the disk.
pub struct EncryptedKeyfile {
    /// The encrypted version of the keyfile.
    #[serde_as(as = "Base64")]
    pub(crate) encrypted_keyfile_content: Vec<u8>,

    /// The XChaCha20 nonce.
    #[serde_as(as = "Base64")]
    pub(crate) nonce: [u8; 24],
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
/// The data for decrypting the keyfile with the password
pub struct PasswordAuthParameters {
    /// Argon2id param
    pub(crate) m_cost: u32,
    /// Argon2id param
    pub(crate) t_cost: u32,
    /// Argon2id param
    pub(crate) p_cost: u32,

    #[serde_as(as = "Base64")]
    /// Argon2id parameter.
    /// Public randomized value which is added to the password before hashing,
    /// to avoid using rainbow tables
    pub(crate) salt: Vec<u8>,

    /// This is the version of the KEK specific to password auth.
    /// Use the key derived from the password to get the KEK,
    /// then decrypt the EncryptedKeyFile with it.
    pub(crate) encrypted_kek: EncryptedKek,
}

#[derive(Serialize, Deserialize, Clone)]
/// The data for decrypting the keyfile with the Yubikey
pub struct YubikeyAuthParams {
    pub slots: Vec<YubikeyAuthSlot>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct YubikeyAuthSlot {
    /// Challenge seed. Concatenate this with the PIN
    /// to get the challenge for the Yubikey
    #[serde_as(as = "Base64")]
    pub(crate) challenge_seed: Vec<u8>,

    /// Argon2id param
    pub(crate) m_cost: u32,
    /// Argon2id param
    pub(crate) t_cost: u32,
    /// Argon2id param
    pub(crate) p_cost: u32,

    #[serde_as(as = "Base64")]
    /// Argon2id parameter.
    /// Public randomized value which is added to the password before hashing,
    /// to avoid using rainbow tables
    pub(crate) salt: Vec<u8>,

    /// This slot's encrypted KEK.
    pub(crate) encrypted_kek: EncryptedKek,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedKek {
    #[serde_as(as = "Base64")]
    pub(crate) ciphertext: Vec<u8>,

    #[serde_as(as = "Base64")]
    pub(crate) nonce: [u8; 24],
}
