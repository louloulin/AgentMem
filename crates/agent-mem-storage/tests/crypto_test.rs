use agent_mem_storage::crypto::{generate_encryption_key, encrypt_data, decrypt_data};

#[test]
fn test_crypto_generates_valid_key() {
    let key = generate_encryption_key();
    assert!(!key.is_empty());
    assert!(key.len() > 40); // base64编码后应该很长
}

#[test]
fn test_crypto_roundtrip() {
    let key = generate_encryption_key();
    let data = b"Hello, AgentMem!";
    
    let encrypted = encrypt_data(data, &key).unwrap();
    let decrypted = decrypt_data(&encrypted, &key).unwrap();
    
    assert_eq!(data.to_vec(), decrypted);
}

#[test]
fn test_crypto_different_nonces() {
    let key = generate_encryption_key();
    let data = b"Test";
    
    let e1 = encrypt_data(data, &key).unwrap();
    let e2 = encrypt_data(data, &key).unwrap();
    
    assert_ne!(e1, e2); // 相同数据不同加密结果
}

#[test]
fn test_crypto_rejects_invalid_key() {
    let result = encrypt_data(b"test", "invalid!!!");
    assert!(result.is_err());
}
