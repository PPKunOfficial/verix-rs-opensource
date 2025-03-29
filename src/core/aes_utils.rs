use std::io;
use rand::Rng;
use openssl::error::ErrorStack;
use openssl::symm::{Cipher, Crypter, Mode};


#[allow(dead_code)]
pub fn aes_encrypt(key: &Vec<u8>, iv: &Vec<u8>, data: &Vec<u8>) -> Result<Vec<u8>, ErrorStack> {
    let cipher = Cipher::aes_256_cbc(); // 使用 AES-256-CBC 算法
    let mut crypter = Crypter::new(cipher, Mode::Encrypt, key.as_slice(), Some(iv.as_slice()))?;

    let mut ciphertext = vec![0; data.len() + cipher.block_size()];
    let mut pos = crypter.update(data.as_slice(), &mut ciphertext)?;
    pos += crypter.finalize(&mut ciphertext[pos..])?;

    ciphertext.truncate(pos);

    // 将 IV 和密文合并，形成最终的密文
    let mut encrypted_data = iv.clone();
    encrypted_data.extend_from_slice(&ciphertext);

    Ok(encrypted_data)
}

#[allow(dead_code)]
pub fn aes_decrypt(key: &Vec<u8>, ciphertext: &Vec<u8>) -> Result<Vec<u8>, ErrorStack> {
    let iv_bytes = &ciphertext[..16];
    let ciphertext = &ciphertext[16..];

    let cipher = Cipher::aes_256_cbc(); // 使用 AES-256-CBC 算法
    let mut crypter = Crypter::new(cipher, Mode::Decrypt, key.as_slice(), Some(iv_bytes))?;

    let mut plaintext = vec![0; ciphertext.len() + cipher.block_size()];
    let mut pos = crypter.update(ciphertext, &mut plaintext)?;
    pos += crypter.finalize(&mut plaintext[pos..])?;

    plaintext.truncate(pos);
    Ok(plaintext)
}

pub fn generate_aes_iv() -> io::Result<Vec<u8>> {
    const IV_SIZE: usize = 16; // 128位初始化向量
    let mut rng = rand::thread_rng();
    let iv: Vec<u8> = (0..IV_SIZE).map(|_| rng.gen()).collect();

    Ok(iv)
}