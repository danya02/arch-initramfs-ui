use std::{io::Read, process::Stdio};

use dialoguer::theme::ColorfulTheme;
use secrecy::Secret;

use crate::params::{
    EncryptedKeyfile, EncryptionParams, PasswordAuthParameters, YubikeyAuthParams,
};

pub mod keyfile;
mod params;
mod unlock_password;
mod unlock_yubikey;
fn main() -> anyhow::Result<()> {
    use dialoguer::*;
    println!("This tool will generate a new config file for the boot menu.");
    let theme = ColorfulTheme::default();

    // Check whether there is already a config file.
    {
        let existing_file = std::fs::OpenOptions::new()
            .read(true)
            .create(false)
            .open("encrypt-config.json");
        if let Ok(_file) = existing_file {
            if Confirm::with_theme(&theme).with_prompt("Found file encrypt-config.json already; do you want to delete it and regenerate all keys?").interact()? {
                std::fs::remove_file("encrypt-config.json")?;
            } else {
                return Ok(());
            }
        }
    }

    println!("Reading keyfile...");
    let keyfile_bytes = match std::fs::OpenOptions::new()
        .read(true)
        .create(false)
        .open("keyfile.secret")
    {
        Err(why) => {
            println!("Failed to open `keyfile.secret`: {why}");
            println!("The file `keyfile.secret` must contain a keyfile that has been enrolled into cryptsetup.");
            println!("Check README.md for details.");
            return Ok(());
        }
        Ok(mut file) => {
            let mut out: Vec<u8> = vec![];
            file.read_to_end(&mut out)?;
            println!("Read {} bytes!", out.len());
            out
        }
    };
    let keyfile_bytes = Secret::new(keyfile_bytes);

    println!("Encrypting keyfile...");
    let (encrypted_keyfile, kek) = EncryptedKeyfile::new(keyfile_bytes);

    println!("Keyfile encrypted! Now building decryption methods:");

    println!("Password:");
    let password = Password::with_theme(&theme)
        .with_prompt("Please enter password to use at boot")
        .with_confirmation("Repeat password", "Error: the passwords don't match.")
        .interact()?;
    let password = Secret::new(password);

    println!("Building password-based keyfile unlock...");
    let pw_params = PasswordAuthParameters::new(password, &kek);
    println!("Done!");

    println!("Yubikey challenge-response:");
    println!("For this step, please make sure that this computer has exactly one Yubikey plugged into it,");
    println!("that its slot 1 is configured for challenge-response auth,");
    println!("and that the `ykchalresp` program is available.");
    println!("Alternatively, you can skip this step and not register a Yubikey:");
    println!("if a Yubikey is inserted at boot, you will still be asked for a PIN,");
    println!("but it will not unlock the disk.");

    let yk_params;
    if !Confirm::with_theme(&theme)
        .with_prompt("Choose Yes once the Yubikey is ready, or No to skip")
        .interact()?
    {
        println!("Yubikey will not be used");
        yk_params = YubikeyAuthParams { slots: vec![] };
    } else {
        let pin = Password::with_theme(&theme)
            .with_prompt("Please enter the PIN (short password) to use at boot with Yubikey")
            .with_confirmation("Repeat PIN", "Error: the PINs don't match.")
            .interact()?;
        let slots = 16;
        println!("We will enroll {slots} slots. You may need to hold down the Yubikey button.");
        let chalresp = |data: [u8; 32]| -> Option<[u8; 20]> {
            let data = hex_string::HexString::from_bytes(&data.to_vec());
            println!("Trying to perform challenge-response with ykchalresp and value={data:?}...");
            let child = std::process::Command::new("ykchalresp")
                .arg("-x")
                .arg(&data.as_string())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to launch ykchalresp");
            let output = child
                .wait_with_output()
                .expect("Failed to wait for child to return");

            if !output.status.success() {
                println!("ykchalresp returned status: {:?}", output.status);
                return None;
            }
            let outdata = String::from_utf8_lossy(&output.stdout);
            let outdata = hex_string::HexString::from_string(
                &outdata
                    .strip_suffix("\n")
                    .expect("ykchalresp returned empty string?"),
            )
            .expect("ykchalresp returned non-hex string?");
            let outdata: [u8; 20] = outdata
                .as_bytes()
                .try_into()
                .expect("ykchalresp returned not 20 bytes?");

            Some(outdata)
        };
        yk_params = YubikeyAuthParams::new_with_slots(slots, Secret::new(pin), chalresp, &kek);
        println!("Done!");
    }

    println!("Writing config file...");

    let config = EncryptionParams {
        keyfile: encrypted_keyfile,
        password_auth: pw_params,
        yubikey_auth: yk_params,
    };
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("encrypt-config.json")?;
    serde_json::to_writer(file, &config)?;

    Ok(())
}
