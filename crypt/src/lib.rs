use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::Rng;
use rand::rngs::OsRng;

// create an alias for convenience
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

// This function will encrypt the given data using the provided key
pub fn encrypt(data: &str, key: &[u8; 16]) -> Vec<u8> {
    let iv: [u8; 16] = OsRng.gen();
    let cipher = Aes128Cbc::new_from_slices(key, &iv).unwrap();
    let cipher_text = cipher.encrypt_vec(data.as_bytes());
    cipher_text
}
