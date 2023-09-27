extern crate sha2;
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn hash_string(password: &str, salt: &str) -> String {
    // Concatenate the password and salt
    let salted_password = format!("{}{}", password, salt);

    // Create a SHA-256 hasher
    let mut hasher = Sha256::new();

    // Update the hasher with the salted password bytes
    hasher.update(salted_password.as_bytes());

    // Finalize the hash and get the result as a byte array
    let result = hasher.finalize();

    // Convert the result to a hexadecimal string
    let hash_hex = format!("{:x}", result);

    hash_hex
}

pub fn verify_string(stored_hash: &str, entered_password: &str, salt: &str) -> bool {
    let entered_hash = hash_string(entered_password, salt);
    entered_hash == stored_hash
}

pub fn generate_salt() -> String {
    let length = 16;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    let random_string: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    random_string
}
