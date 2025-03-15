use anyhow::Result;
use uniclipboard::encrypt::Encryptor;


#[test]
fn test_encryption_decryption() -> Result<()> {
    let key = [0u8; 32]; // 使用全零的密钥作为测试
    let encryptor = Encryptor::from_key(&key);

    let plaintext = b"Hello, World!";
    let encrypted = encryptor.encrypt(plaintext)?;
    let decrypted = encryptor.decrypt(&encrypted)?;

    assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    Ok(())
}

#[test]
fn test_different_plaintexts() -> Result<()> {
    let key = [1u8; 32]; // 使用全1的密钥作为测试
    let encryptor = Encryptor::from_key(&key);

    let plaintexts: Vec<&[u8]> = vec![
        b"Short text",
        b"A bit longer text with some numbers 12345",
        b"",
        &[0u8; 1000], // 1000字节的零
    ];

    for plaintext in plaintexts {
        let encrypted = encryptor.encrypt(plaintext)?;
        let decrypted = encryptor.decrypt(&encrypted)?;
        assert_eq!(plaintext, decrypted.as_slice());
    }

    Ok(())
}

#[test]
fn test_invalid_ciphertext() {
    let key = [2u8; 32];
    let encryptor = Encryptor::from_key(&key);

    let result = encryptor.decrypt(&[0u8; 11]); // 长度小于12的无效密文
    assert!(result.is_err());

    let result = encryptor.decrypt(&[0u8; 12]); // 只有nonce没有实际密文
    assert!(result.is_err());
}

#[test]
fn test_different_keys() -> Result<()> {
    let key1 = [3u8; 32];
    let key2 = [4u8; 32];
    let encryptor1 = Encryptor::from_key(&key1);
    let encryptor2 = Encryptor::from_key(&key2);

    let plaintext = b"Secret message";
    let encrypted = encryptor1.encrypt(plaintext)?;
    
    // 使用不同的密钥解密应该失败
    let result = encryptor2.decrypt(&encrypted);
    assert!(result.is_err());

    Ok(())
}