use argon2::Params;
use rand::{seq::SliceRandom, Rng};
use secrecy::{Secret, SecretString};
use sha2::Sha256;

use crate::{
    keyfile::KeyEncryptionKey,
    params::{YubikeyAuthParams, YubikeyAuthSlot},
};

impl YubikeyAuthParams {
    pub fn new_with_slots<F>(
        how_many: usize,
        pin: SecretString,
        mut chalresp: F,
        kek: &KeyEncryptionKey,
    ) -> Self
    where
        F: FnMut([u8; 32]) -> Option<[u8; 20]>,
    {
        let mut slots = Vec::with_capacity(how_many);
        for _ in 0..how_many {
            let slot = YubikeyAuthSlot::new(&pin, &mut chalresp, kek);
            slots.push(slot);
        }
        Self { slots }
    }

    pub fn decrypt<F>(&self, pin: SecretString, mut chalresp: F) -> Result<KeyEncryptionKey, ()>
    where
        F: FnMut([u8; 32]) -> Option<[u8; 20]>,
    {
        // Pick one of the slots at random.
        let mut rng = rand::rngs::OsRng::default();
        let chosen_slot = self.slots.choose(&mut rng).ok_or(())?;
        chosen_slot.decrypt(&pin, &mut chalresp)
    }
}

impl YubikeyAuthSlot {
    pub fn new<F>(pin: &SecretString, chalresp: &mut F, kek: &KeyEncryptionKey) -> Self
    where
        F: FnMut([u8; 32]) -> Option<[u8; 20]>,
    {
        use secrecy::ExposeSecret;
        use sha2::Digest;
        let mut rng = rand::rngs::OsRng::default();
        let seed_length = rng.gen_range(64..128);

        let seed: Vec<u8> = (0..seed_length).map(|_| rng.gen()).collect();

        let mut raw_challenge = seed.clone();
        raw_challenge.extend(pin.expose_secret().as_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&raw_challenge);
        let challenge: [u8; 32] = hasher.finalize().into();

        let response =
            chalresp(challenge).expect("Failed to perform challenge-response on Yubikey");

        // These params are tweaked to be faster than normal
        let m_cost = argon2::Params::MIN_M_COST * 64;
        let p_cost = 8;
        let t_cost = 16;

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
        kdf.hash_password_into(&response, &salt, &mut output_hash)
            .expect("Failed to hash Yubikey response");

        // Use the hash to encrypt the KEK
        let key: [u8; 32] = output_hash
            .try_into()
            .expect("Argon2 produced not 32 bytes?");
        let ekek = kek.encrypt(Secret::new(key));

        Self {
            challenge_seed: seed,
            salt,
            m_cost,
            t_cost,
            p_cost,
            encrypted_kek: ekek,
        }
    }

    pub fn decrypt<F>(&self, pin: &SecretString, chalresp: &mut F) -> Result<KeyEncryptionKey, ()>
    where
        F: FnMut([u8; 32]) -> Option<[u8; 20]>,
    {
        use secrecy::ExposeSecret;
        use sha2::Digest;

        let mut raw_challenge = self.challenge_seed.clone();
        raw_challenge.extend(pin.expose_secret().as_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&raw_challenge);
        let challenge: [u8; 32] = hasher.finalize().into();

        let response = chalresp(challenge).ok_or(())?;

        let kdf = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(self.m_cost, self.t_cost, self.p_cost, Some(32))
                .expect("Failed to build Argon2 params"),
        );
        let mut output_hash = vec![0; 32];
        kdf.hash_password_into(&response, &self.salt, &mut output_hash)
            .expect("Failed to hash Yubikey response");
        let key: [u8; 32] = output_hash
            .try_into()
            .expect("Argon2 produced not 32 bytes?");

        self.encrypted_kek.decrypt(Secret::new(key))
    }
}

#[cfg(test)]
mod test {
    use secrecy::{ExposeSecret, Secret};

    use crate::{keyfile::KeyEncryptionKey, params::YubikeyAuthParams};

    #[test]
    fn test_yubikey_round_trip() {
        let src_kek_data: Vec<u8> = (0..32).collect();
        let src_kek_data: [u8; 32] = src_kek_data.try_into().unwrap();

        let kek = KeyEncryptionKey {
            key: Secret::new(src_kek_data),
        };

        // For testing, the Yubikey will be substituted by a simple in-memory transformation.
        let mock_chalresp = |data: [u8; 32]| -> Option<[u8; 20]> {
            // Pick the first 20 bytes, then negate them
            let mut slice = data[0..20].to_vec();
            for i in slice.iter_mut() {
                *i = 255 - *i;
            }
            Some(slice.try_into().unwrap())
        };

        let pin = String::from("1234");

        let params =
            YubikeyAuthParams::new_with_slots(10, Secret::new(pin.clone()), mock_chalresp, &kek);

        // ...

        let dkek = params.decrypt(Secret::new(pin), mock_chalresp).unwrap();
        let dest_kek_data = dkek.key.expose_secret();
        assert_eq!(dest_kek_data, &src_kek_data);
    }
}
