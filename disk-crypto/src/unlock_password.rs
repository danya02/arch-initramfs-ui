use argon2::Params;
use rand::Rng;
use secrecy::{Secret, SecretString};

use crate::{keyfile::KeyEncryptionKey, params::PasswordAuthParameters};

impl PasswordAuthParameters {
    /// Create a new password keyslot from a password.
    pub fn new(password: SecretString, kek: &KeyEncryptionKey) -> Self {
        use secrecy::ExposeSecret;
        // WARNING: if your string has Unicode combining characters,
        // they may be encoded differently despite having the same meaning.
        // Either normalize your string before this,
        // or (better) disallow using those characters.
        let pw_buf = password.expose_secret().as_bytes();
        Self::new_internal(pw_buf, kek)
    }

    /// Create a new password keyslot that cannot be solved.
    /// This is guaranteed, as the password used contains invalid Unicode,
    /// and cannot be represented by a String.
    pub fn new_unsolveable(kek: &KeyEncryptionKey) -> Self {
        let mut rng = rand::rngs::OsRng::default();

        let mut fake_password = Vec::with_capacity(32);
        for _ in 0..31 {
            fake_password.push(rng.gen());
        }
        fake_password.push(0); // This line ensures that the password is not a String,
                               // even if the previous ones didn't.

        Self::new_internal(&fake_password, kek)
    }

    fn new_internal(password: &[u8], kek: &KeyEncryptionKey) -> Self {
        let m_cost = argon2::Params::DEFAULT_M_COST;
        let p_cost = argon2::Params::DEFAULT_P_COST;
        let t_cost = argon2::Params::DEFAULT_T_COST;

        let mut rng = rand::rngs::OsRng::default();

        let salt: Vec<u8> = (0..argon2::RECOMMENDED_SALT_LEN)
            .map(|_| rng.gen())
            .collect();

        let kdf = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(m_cost, t_cost, p_cost, Some(32)).expect("Failed to build Argon2 params"),
        );
        let mut output_hash = vec![0; 32];
        kdf.hash_password_into(password, &salt, &mut output_hash)
            .expect("Failed to hash password");

        // Use the hash to encrypt the KEK
        let key: [u8; 32] = output_hash
            .try_into()
            .expect("Argon2 produced not 32 bytes?");
        let ekek = kek.encrypt(Secret::new(key));

        Self {
            m_cost,
            t_cost,
            p_cost,
            salt,
            encrypted_kek: ekek,
        }
    }

    pub fn decrypt(&self, password: SecretString) -> Result<KeyEncryptionKey, ()> {
        use secrecy::ExposeSecret;

        let kdf = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(self.m_cost, self.t_cost, self.p_cost, Some(32))
                .expect("Failed to build Argon2 params"),
        );

        let password = password.expose_secret().as_bytes();
        let mut output_hash = vec![0; 32];
        kdf.hash_password_into(password, &self.salt, &mut output_hash)
            .expect("Failed to hash password");

        let output_hash: [u8; 32] = output_hash
            .try_into()
            .expect("Argon2 generated not 32 bytes?");
        self.encrypted_kek.decrypt(Secret::new(output_hash))
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;
    use secrecy::{ExposeSecret, Secret};

    use crate::{keyfile::KeyEncryptionKey, params::PasswordAuthParameters};

    #[test]
    fn test_password_kek_round_trip() {
        let src_kek_data: Vec<u8> = (0..32).collect();
        let src_kek_data: [u8; 32] = src_kek_data.try_into().unwrap();

        let kek = KeyEncryptionKey {
            key: Secret::new(src_kek_data),
        };

        let mut rng = rand::rngs::OsRng::default();
        let password = format!("Hello Cryptography! {}", rng.gen_range(0f64..1f64));

        let pw_auth = PasswordAuthParameters::new(Secret::new(password.clone()), &kek);

        // ...

        let dkek = pw_auth.decrypt(Secret::new(password)).unwrap();
        let dest_kek_data = dkek.key.expose_secret();
        assert_eq!(dest_kek_data, &src_kek_data);
    }
}
