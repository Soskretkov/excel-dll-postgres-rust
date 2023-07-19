use std::fs::File;
use std::io::Write;
use crypt::encrypt;
use rand::Rng;


// cargo run --bin crypt
fn main() {
    let config_path = "crypt\\unencrypted.txt";
    let output_path = "crypt\\encrypted.txt";
    let key_output_path = "crypt\\encryption_key.txt";

    let encryption_key: [u8; 16] = rand::thread_rng().gen();
    let mut key_file = File::create(key_output_path).expect("Could not create key file");
    key_file.write_all(&encryption_key).expect("Could not write to key file");

    let config_data = std::fs::read_to_string(config_path).expect("Could not read config file");
    let encrypted_data = encrypt(&config_data, &encryption_key);

    let mut file = File::create(output_path).expect("Could not create output file");
    file.write_all(&encrypted_data).expect("Could not write to output file");
}
