use chacha20poly1305::{aead::Aead, AeadCore, KeyInit, XChaCha20Poly1305};
use rand::Rng;
use secrecy::{Secret, SecretVec};

use crate::params::{EncryptedKek, EncryptedKeyfile};

impl EncryptedKeyfile {
    pub fn new(plain_keyfile_content: SecretVec<u8>) -> (Self, KeyEncryptionKey) {
        use secrecy::ExposeSecret;
        let mut rng = rand::rngs::OsRng::default();
        // Generate the encryption key for self.
        let auth_key = XChaCha20Poly1305::generate_key(&mut rng);

        let cipher = XChaCha20Poly1305::new(&auth_key);
        let nonce = XChaCha20Poly1305::generate_nonce(&mut rng);

        let plaintext: &[u8] = plain_keyfile_content.expose_secret();
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .expect("Failed to encrypt keyfile");
        (
            Self {
                encrypted_keyfile_content: ciphertext,
                nonce: nonce.try_into().unwrap(),
            },
            KeyEncryptionKey {
                key: Secret::new(auth_key.try_into().unwrap()),
            },
        )
    }

    pub fn decrypt(&self, kek: KeyEncryptionKey) -> Result<SecretVec<u8>, ()> {
        use secrecy::ExposeSecret;
        let key = kek.key.expose_secret();
        let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| ())?;
        let ciphertext: &[u8] = &self.encrypted_keyfile_content;
        let plaintext = cipher
            .decrypt(&self.nonce.into(), ciphertext)
            .map_err(|_| ())?;
        Ok(Secret::new(plaintext))
    }
}

/// This can be used to decrypt the EncryptedKeyfile.
pub struct KeyEncryptionKey {
    pub(crate) key: Secret<[u8; 32]>,
}

impl KeyEncryptionKey {
    /// Encrypt the KEK for on-disk storage
    pub fn encrypt(&self, key: Secret<[u8; 32]>) -> EncryptedKek {
        use secrecy::ExposeSecret;

        let mut rng = rand::rngs::OsRng::default();

        let key = key.expose_secret();
        let cipher =
            XChaCha20Poly1305::new_from_slice(key).expect("XChaCha20 key should be 32 bytes");
        let plaintext: &[u8] = self.key.expose_secret();
        let nonce: [u8; 24] = rng.gen();
        let ciphertext = cipher
            .encrypt(&nonce.try_into().unwrap(), plaintext)
            .expect("Failed to encrypt KEK");
        EncryptedKek { ciphertext, nonce }
    }
}

impl EncryptedKek {
    /// Decrypt the KEK for use in the app
    pub fn decrypt(&self, key: Secret<[u8; 32]>) -> Result<KeyEncryptionKey, ()> {
        use secrecy::ExposeSecret;

        let key = key.expose_secret();
        let cipher =
            XChaCha20Poly1305::new_from_slice(key).expect("XChaCha20 key should be 32 bytes");

        let plaintext = cipher.decrypt(&self.nonce.into(), &self.ciphertext[..]);
        let plaintext = plaintext.map_err(|_| ())?;
        let plaintext: [u8; 32] = plaintext
            .try_into()
            .expect("KEK should be 32 bytes in length");

        Ok(KeyEncryptionKey {
            key: Secret::new(plaintext),
        })
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;
    use secrecy::{ExposeSecret, Secret, SecretVec};

    use crate::params::EncryptedKeyfile;

    use super::KeyEncryptionKey;

    #[test]
    fn test_keyfile_round_trip() {
        let src_keyfile = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let keyfile: SecretVec<u8> = src_keyfile.clone().into();
        let (enc_keyfile, kek) = EncryptedKeyfile::new(keyfile);

        // ...

        let decrypted_keyfile = enc_keyfile.decrypt(kek).unwrap();
        let decrypted_keyfile = decrypted_keyfile.expose_secret();
        assert_eq!(&src_keyfile, decrypted_keyfile);
    }

    #[test]
    fn test_kek_round_trip() {
        let src_kek_data: Vec<u8> = (0..32).collect();
        let src_kek_data: [u8; 32] = src_kek_data.try_into().unwrap();

        let kek = KeyEncryptionKey {
            key: Secret::new(src_kek_data),
        };

        let mut rng = rand::rngs::OsRng::default();

        let kek_key: [u8; 32] = rng.gen();
        let kek_key = Secret::new(kek_key);
        let ekek = kek.encrypt(kek_key.clone());

        // ...

        let dkek = ekek.decrypt(kek_key).unwrap();
        let dest_kek_data = dkek.key.expose_secret();
        assert_eq!(dest_kek_data, &src_kek_data);
    }
}
