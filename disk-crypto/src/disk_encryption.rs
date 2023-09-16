use std::process::Stdio;

use secrecy::{ExposeSecret, Secret};

use crate::params::EncryptionParams;

impl EncryptionParams {
    pub fn try_keyfile_from_password(&self, pw: String) -> Result<Vec<u8>, ()> {
        let kek = self.password_auth.decrypt(Secret::new(pw))?;
        let keyfile = self.keyfile.decrypt(kek)?;
        Ok(keyfile.expose_secret().clone())
    }

    pub fn try_keyfile_from_pin(&self, pin: String) -> Result<Vec<u8>, ()> {
        let chalresp = |data: [u8; 32]| -> Option<[u8; 20]> {
            let data = hex_string::HexString::from_bytes(&data.to_vec());
            let child = std::process::Command::new("ykchalresp")
                .arg("-x")
                .arg(&data.as_string())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to launch ykchalresp");
            let output = child.wait_with_output().ok()?;

            if !output.status.success() {
                return None;
            }
            let outdata = String::from_utf8_lossy(&output.stdout);
            let outdata = hex_string::HexString::from_string(&outdata.strip_suffix("\n")?).ok()?;
            let outdata: [u8; 20] = outdata.as_bytes().try_into().ok()?;

            Some(outdata)
        };

        let kek = self.yubikey_auth.decrypt(Secret::new(pin), chalresp)?;
        let keyfile = self.keyfile.decrypt(kek)?;
        Ok(keyfile.expose_secret().clone())
    }
}
