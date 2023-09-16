To prepare your LUKS crypted disk for use with this:

- Create a keyfile: `openssl genrsa -out ./keyfile.secret 4096`. Keep this value very secret!
- Enroll the keyfile into your disk's keyslots: `cryptsetup luksAddKey /dev/nvme0n1p3 ./keyfile.secret`
- Run the program for generating a boot-menu config: `cargo run`. Follow the prompts.