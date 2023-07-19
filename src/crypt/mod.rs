use aes_soft::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use hex_literal::hex;

// create an alias for convenience
type Aes256Cbc = Cbc<Aes256, Pkcs7>;

#[derive(Debug)]
struct Cipher {
    key: [u8; 32],
    iv: [u8; 16],
}

impl Cipher {
    fn new(key: [u8; 32], iv: [u8; 16]) -> Cipher {
        Cipher { key, iv }
    }

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let cipher = Aes256Cbc::new_var(&self.key, &self.iv).unwrap();
        cipher.encrypt_vec(data)
    }

    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        let cipher = Aes256Cbc::new_var(&self.key, &self.iv).unwrap();
        cipher.decrypt_vec(data).unwrap()
    }
}
