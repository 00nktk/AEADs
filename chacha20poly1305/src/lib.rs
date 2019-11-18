//! [`ChaCha20Poly1305`] ([RFC 8439][1]) is an
//! [Authenticated Encryption with Associated Data (AEAD)][2]
//! cipher amenable to fast, constant-time implementations in software, based on
//! the [ChaCha20][3] stream cipher and [Poly1305][4] universal hash function.
//!
//! This crate also contains an implementation of [`XChaCha20Poly1305`] -
//! a variant of ChaCha20Poly1305 with an extended 192-bit (24-byte) nonce.
//!
//! ## Security Warning
//!
//! No security audits of this crate have ever been performed, and it has not been
//! thoroughly assessed to ensure its operation is constant-time on common CPU
//! architectures.
//!
//! Where possible the implementation uses constant-time hardware intrinsics,
//! or otherwise falls back to an implementation which contains no secret-dependent
//! branches or table lookups, however it's possible LLVM may insert such
//! operations in certain scenarios.
//!
//! # Usage
//!
//! ```
//! use chacha20poly1305::ChaCha20Poly1305;
//! use aead::{Aead, NewAead, generic_array::GenericArray};
//!
//! let key = GenericArray::clone_from_slice(b"an example very very secret key."); // 32-bytes
//! let aead = ChaCha20Poly1305::new(key);
//!
//! let nonce = GenericArray::from_slice(b"unique nonce"); // 12-bytes; unique per message
//! let ciphertext = aead.encrypt(nonce, b"plaintext message".as_ref()).expect("encryption failure!");
//! let plaintext = aead.decrypt(nonce, ciphertext.as_ref()).expect("decryption failure!");
//! assert_eq!(&plaintext, b"plaintext message");
//! ```
//!
//! [1]: https://tools.ietf.org/html/rfc8439
//! [2]: https://en.wikipedia.org/wiki/Authenticated_encryption
//! [3]: https://github.com/RustCrypto/stream-ciphers/tree/master/chacha20
//! [4]: https://github.com/RustCrypto/universal-hashes/tree/master/poly1305

#![no_std]
#![doc(html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png")]
#![warn(missing_docs, rust_2018_idioms, intra_doc_link_resolution_failure)]

mod cipher;
#[cfg(feature = "xchacha20poly1305")]
mod xchacha20poly1305;

pub use aead;
#[cfg(feature = "xchacha20poly1305")]
pub use xchacha20poly1305::XChaCha20Poly1305;

use self::cipher::Cipher;
use aead::generic_array::{
    typenum::{U0, U12, U16, U32},
    GenericArray,
};
use aead::{Aead, Error, NewAead};
use chacha20::{stream_cipher::NewStreamCipher, ChaCha20};
use zeroize::Zeroize;

/// Poly1305 tags
pub type Tag = GenericArray<u8, U16>;

/// ChaCha20Poly1305 Authenticated Encryption with Additional Data (AEAD).
///
/// The [`Aead`] and [`NewAead`] traits provide the primary API for using this
/// construction.
///
/// See the [toplevel documentation](https://docs.rs/chacha20poly1305) for
/// a usage example.
#[derive(Clone)]
pub struct ChaCha20Poly1305 {
    /// Secret key
    key: GenericArray<u8, U32>,
}

impl NewAead for ChaCha20Poly1305 {
    type KeySize = U32;

    fn new(key: GenericArray<u8, U32>) -> Self {
        ChaCha20Poly1305 { key }
    }
}

impl Aead for ChaCha20Poly1305 {
    type NonceSize = U12;
    type TagSize = U16;
    type CiphertextOverhead = U0;

    fn encrypt_in_place_detached(
        &self,
        nonce: &GenericArray<u8, Self::NonceSize>,
        associated_data: &[u8],
        buffer: &mut [u8],
    ) -> Result<Tag, Error> {
        Cipher::new(ChaCha20::new(&self.key, nonce))
            .encrypt_in_place_detached(associated_data, buffer)
    }

    fn decrypt_in_place_detached(
        &self,
        nonce: &GenericArray<u8, Self::NonceSize>,
        associated_data: &[u8],
        buffer: &mut [u8],
        tag: &Tag,
    ) -> Result<(), Error> {
        Cipher::new(ChaCha20::new(&self.key, nonce)).decrypt_in_place_detached(
            associated_data,
            buffer,
            tag,
        )
    }
}

impl Drop for ChaCha20Poly1305 {
    fn drop(&mut self) {
        self.key.as_mut_slice().zeroize();
    }
}
