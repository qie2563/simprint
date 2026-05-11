//! 配置加密/解密模块
//!
//! 提供配置文件的多层加密和解密功能，使用 AES-256-GCM 加密算法。
//! 包含多层防御：密钥派生、数据变换、分块加密、认证标签。

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use sha2::{Digest, Sha256};

use super::key_derivation::derive_subkeys;

// 使用简单的 Result 类型，兼容 build.rs 和运行时
type Result<T> = std::result::Result<T, String>;

/// 加密配置数据（多层防御）
///
/// 加密流程：
/// 1. 数据预变换（XOR + 字节重排）
/// 2. 分块加密（每块使用不同的 nonce）
/// 3. 添加认证标签（防篡改）
/// 4. 数据后变换（字节混淆）
pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>> {
    let keys = derive_subkeys();

    // 第一层：数据预变换（混淆）
    let transformed = pre_transform(plaintext, &keys.transform_key);

    // 第二层：分块加密
    let encrypted = block_encrypt(&transformed, &keys.encryption_key)?;

    // 第三层：添加认证标签
    let authenticated = add_auth_tag(&encrypted, &keys.auth_key);

    // 第四层：数据后变换
    let final_data = post_transform(&authenticated, &keys.transform_key);

    Ok(final_data)
}

/// 解密配置数据（多层防御）
///
/// 解密流程（加密的逆过程）：
/// 1. 数据逆后变换
/// 2. 验证认证标签
/// 3. 分块解密
/// 4. 数据逆预变换
pub fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>> {
    let keys = derive_subkeys();

    // 第一层：数据逆变换
    let untransformed = post_transform_reverse(ciphertext, &keys.transform_key);

    // 第二层：验证认证标签
    let (data, expected_tag) = extract_auth_tag(&untransformed)?;
    verify_auth_tag(data, expected_tag, &keys.auth_key)?;

    // 第三层：分块解密
    let decrypted = block_decrypt(data, &keys.encryption_key)?;

    // 第四层：数据逆预变换
    let plaintext = pre_transform_reverse(&decrypted, &keys.transform_key);

    Ok(plaintext)
}

/// 数据预变换：XOR + 字节重排
fn pre_transform(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(data.len());

    // XOR 变换
    for (i, &byte) in data.iter().enumerate() {
        result.push(byte ^ key[i % 16]);
    }

    result.reverse();
    let shift = transform_shift(result.len(), key);
    result.rotate_left(shift);
    result
}

/// 数据预变换的逆操作
fn pre_transform_reverse(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut unpermuted = data.to_vec();
    let shift = transform_shift(unpermuted.len(), key);
    unpermuted.rotate_right(shift);
    unpermuted.reverse();

    // 逆 XOR
    let mut result = Vec::with_capacity(unpermuted.len());
    for (i, &byte) in unpermuted.iter().enumerate() {
        result.push(byte ^ key[i % 16]);
    }

    result
}

/// 分块加密：将数据分成多块，每块使用不同的 nonce
fn block_encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    const BLOCK_SIZE: usize = 4096; // 4KB 块

    let cipher = Aes256Gcm::new(key.into());
    let mut result = Vec::new();

    // 写入块数量
    let block_count = (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;
    result.extend_from_slice(&(block_count as u32).to_le_bytes());

    // 分块加密
    for (block_idx, chunk) in data.chunks(BLOCK_SIZE).enumerate() {
        // 为每个块派生不同的 nonce
        let nonce_bytes = derive_block_nonce(block_idx, key);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密块
        let encrypted_block =
            cipher.encrypt(nonce, chunk).map_err(|e| format!("Encryption failed: {}", e))?;

        // 写入块大小和数据
        result.extend_from_slice(&(encrypted_block.len() as u32).to_le_bytes());
        result.extend_from_slice(&encrypted_block);
    }

    Ok(result)
}

/// 分块解密
fn block_decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut result = Vec::new();

    // 读取块数量
    if data.len() < 4 {
        return Err("Invalid data format".to_string());
    }
    let block_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    let mut offset = 4;
    for block_idx in 0..block_count {
        // 读取块大小
        if offset + 4 > data.len() {
            return Err("Invalid block format".to_string());
        }
        let block_size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        // 读取块数据
        if offset + block_size > data.len() {
            return Err("Invalid block data".to_string());
        }
        let block_data = &data[offset..offset + block_size];
        offset += block_size;

        // 派生 nonce 并解密
        let nonce_bytes = derive_block_nonce(block_idx, key);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let decrypted_block = cipher
            .decrypt(nonce, block_data)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        result.extend_from_slice(&decrypted_block);
    }

    Ok(result)
}

/// 为每个块派生不同的 nonce
fn derive_block_nonce(block_idx: usize, key: &[u8; 32]) -> [u8; 12] {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(b"block-nonce");
    hasher.update(&(block_idx as u64).to_le_bytes());
    let result = hasher.finalize();

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&result[..12]);
    nonce
}

/// 添加认证标签
fn add_auth_tag(data: &[u8], auth_key: &[u8; 32]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(auth_key);
    hasher.update(data);
    let tag = hasher.finalize();

    let mut result = Vec::with_capacity(data.len() + 32);
    result.extend_from_slice(data);
    result.extend_from_slice(&tag);
    result
}

/// 提取认证标签
fn extract_auth_tag(data: &[u8]) -> Result<(&[u8], &[u8])> {
    if data.len() < 32 {
        return Err("Data too short for auth tag".to_string());
    }
    let split_pos = data.len() - 32;
    Ok((&data[..split_pos], &data[split_pos..]))
}

/// 验证认证标签
fn verify_auth_tag(data: &[u8], expected_tag: &[u8], auth_key: &[u8; 32]) -> Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(auth_key);
    hasher.update(data);
    let computed_tag = hasher.finalize();

    if computed_tag.as_slice() != expected_tag {
        return Err("Authentication failed".to_string());
    }

    Ok(())
}

/// 数据后变换：简单的字节混淆
fn post_transform(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    data.iter().enumerate().map(|(i, &b)| b.wrapping_add(key[i % 16])).collect()
}

/// 数据后变换的逆操作
fn post_transform_reverse(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    data.iter().enumerate().map(|(i, &b)| b.wrapping_sub(key[i % 16])).collect()
}

fn transform_shift(len: usize, key: &[u8; 16]) -> usize {
    if len <= 1 {
        return 0;
    }

    let seed = key
        .iter()
        .take(4)
        .fold(0usize, |acc, byte| (acc << 8) | (*byte as usize));
    (seed.wrapping_add(len).wrapping_add(13)) % len
}

#[cfg(test)]
mod tests {
    use super::{decrypt, encrypt};

    #[test]
    fn roundtrip_preserves_all_lengths_up_to_1024() {
        for len in 0..=1024usize {
            let input = (0..len)
                .map(|index| ((index * 31 + 17) % 256) as u8)
                .collect::<Vec<_>>();
            let encrypted = encrypt(&input).expect("encrypt should succeed");
            let decrypted = decrypt(&encrypted).expect("decrypt should succeed");
            assert_eq!(decrypted, input, "roundtrip mismatch for len={len}");
        }
    }

    #[test]
    fn roundtrip_preserves_lengths_that_are_multiples_of_seven() {
        for len in [7usize, 14, 497, 504, 4095, 4096, 4097] {
            let input = (0..len)
                .map(|index| ((index * 13 + 29) % 256) as u8)
                .collect::<Vec<_>>();
            let encrypted = encrypt(&input).expect("encrypt should succeed");
            let decrypted = decrypt(&encrypted).expect("decrypt should succeed");
            assert_eq!(decrypted, input, "roundtrip mismatch for len={len}");
        }
    }
}
