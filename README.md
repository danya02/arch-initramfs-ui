# arch-initramfs-ui
My custom boot menu for Arch Linux built on top of initramfs.


# !!! WARNING !!!
This is currently very based on specific assumptions about my computer (such as GPT partition numbers and file system UUIDs).
*DO NOT RUN ANY SCRIPTS* like the `Makefile` before reading and editing them to fix these assumptions.

# Encryption info
For decrypting the system drive, two options are provided:
- Password
- Yubikey challenge-response

You must provide a keyfile, called `DK`, which can be used with `cryptsetup` to unlock the drive.
`DK` is encrypted with ChaCha20, producing `EDK`, and this is stored in the binary.
The key used for this encryption is the key encryption key `KEK`, which is then encrypted in other ways.
`DK` is prefixed with a known string in order to detect whether the decryption is successful.

```
DK -- target value
EDK := ChaCha20_encrypt(DK, KEK)
store EDK in binary
```

## Password
For the password authentication, the password `P` is first converted into a key `K` using the Argon2id key derivation function. 
We then use this key to decrypt the copy of the `KEK` for password auth, called `PKEK`.

```
get P from user
read PKEK from the executable
K := Argon2id(P)
KEK := ChaCha20_decrypt(PKEK, K)
DK := ChaCha20_decrypt(DK, KEK)
unlock disk with DK
```

## Yubikey
For Yubikey authentication, the challenge-response mode is used.
During setup, a number of different challenge seeds `CS` are created.
At runtime, a random `CS` is selected.
It is then concatenated with the input `PIN`, then hashed using SHA256.
This hash is sent to the Yubikey as the challenge `C`.
It responds with a byte string, which is expanded into a 32-byte key
with a simpler configuration of Argon2id, 
which is used to unlock the copy of `KEK` for Yubikey auth, called `YKEK`.

```
choose $N randomly
read CS_$N, RS_$N and YKEK_$N from the executable
get PIN
C := SHA256(CS_$N + PIN)
send C to Yubikey, get R
K := Argon2id(R)
KEK := ChaCha20_decrypt(YKEK_$N, K)
DK := ChaCha20_decrypt(DK, KEK)
unlock disk with DK
```