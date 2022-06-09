use flate2::Compression;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Nonce};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use std::convert::TryInto;
use std::io::Read;

fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn random_number(size: usize) -> Vec<u8> {
    let mut rng = StdRng::from_entropy();
    let mut result: Vec<u8> = vec![0; size];
    rng.fill(&mut result[..]);
    result
}

pub fn generate(random: fn(usize) -> Vec<u8>, alphabet: &[char], size: usize) -> String {
    assert!(
        alphabet.len() <= u8::max_value() as usize,
        "The alphabet cannot be longer than a `u8` (to comply with the `random` function)"
    );
    let mask = alphabet.len().next_power_of_two() - 1;
    let step: usize = 8 * size / 5;
    let mut id = String::with_capacity(size);
    loop {
        let bytes = random(step);
        for &byte in &bytes {
            let byte = byte as usize & mask;
            if alphabet.len() > byte {
                id.push(alphabet[byte]);
                if id.len() == size {
                    return id;
                }
            }
        }
    }
}

pub fn encode(buffer: Vec<u8>, compress: bool, encrypt: Option<Aes256Gcm>) -> Vec<u8> {
    if compress {
        let zlib = ZlibEncoder::new(buffer, Compression::best());
        let compressed = zlib.finish().unwrap();
        if encrypt.is_some() {
            let nonce_bytes = random_number(96);
            let nonce = Nonce::from_slice(&nonce_bytes);
            let no_size: &[u8] = &vec_to_array(compressed);
            let encrypted = encrypt.unwrap().encrypt(nonce, no_size).unwrap();
            let result = Vec::new();
            result.append(&mut nonce_bytes);
            result.append(&mut encrypted);
            result
        } else {
            compressed
        }
    } else {
        if encrypt.is_some() {
            let nonce_bytes = random_number(96);
            let nonce = Nonce::from_slice(&nonce_bytes);
            let no_size: &[u8] = &vec_to_array(buffer);
            let encrypted = encrypt.unwrap().encrypt(nonce, no_size).unwrap();
            let result = Vec::new();
            result.append(&mut nonce_bytes);
            result.append(&mut encrypted);
            result
        } else {
            buffer
        }
    }
}

pub fn decode(buffer: Vec<u8>, compress: bool, encrypt: Option<Aes256Gcm>) -> Vec<u8> {
    if encrypt.is_some() {
        let nonce_split = buffer.split_off(96);
        let nonce_bytes: &[u8] = &vec_to_array(nonce_split);
        let nonce = Nonce::from_slice(&buffer);
        let decrypted = encrypt.unwrap().decrypt(nonce, nonce_bytes).unwrap();
        if compress {
            let data_bytes: &[u8] = &vec_to_array(buffer);
            let zlib = ZlibDecoder::new(data_bytes);
            let mut buf = Vec::new();
            zlib.read_to_end(&mut buf).unwrap();
            buf
        } else {
            decrypted
        }
    } else {
        if compress {
            let data_bytes: &[u8] = &vec_to_array(buffer);
            let zlib = ZlibDecoder::new(data_bytes);
            let mut buf = Vec::new();
            zlib.read_to_end(&mut buf).unwrap();
            buf
        } else {
            buffer
        }
    }
}
