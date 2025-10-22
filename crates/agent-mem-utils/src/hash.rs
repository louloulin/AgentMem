//! 内容 Hash 计算模块
//!
//! 用于记忆内容的去重和唯一标识

use sha2::{Digest, Sha256};

/// 计算内容的 SHA256 hash
///
/// 用于：
/// - 去重检测（相同内容返回相同 hash）
/// - 唯一标识（hash 作为内容指纹）
/// - 变更检测（hash 不同表示内容变化）
///
/// # 示例
///
/// ```
/// use agent_mem_utils::hash::compute_content_hash;
///
/// let hash1 = compute_content_hash("Hello, World!");
/// let hash2 = compute_content_hash("Hello, World!");
/// let hash3 = compute_content_hash("Different content");
///
/// assert_eq!(hash1, hash2);  // 相同内容 hash 相同
/// assert_ne!(hash1, hash3);  // 不同内容 hash 不同
/// ```
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 计算短 hash（用于日志和遥测）
///
/// 返回 hash 的前 8 个字符，用于简短标识
///
/// # 示例
///
/// ```
/// use agent_mem_utils::hash::short_hash;
///
/// let short = short_hash("test");
/// assert_eq!(short.len(), 8);
/// ```
pub fn short_hash(content: &str) -> String {
    let full_hash = compute_content_hash(content);
    full_hash.chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_hash() {
        let hash1 = compute_content_hash("test content");
        let hash2 = compute_content_hash("test content");
        let hash3 = compute_content_hash("different content");

        // 相同内容应该产生相同的 hash
        assert_eq!(hash1, hash2);

        // 不同内容应该产生不同的 hash
        assert_ne!(hash1, hash3);

        // hash 应该是 64 个字符的十六进制字符串（SHA256）
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_short_hash() {
        let short1 = short_hash("test content");
        let short2 = short_hash("test content");
        let short3 = short_hash("different content");

        // 相同内容应该产生相同的短 hash
        assert_eq!(short1, short2);

        // 不同内容应该产生不同的短 hash
        assert_ne!(short1, short3);

        // 短 hash 应该是 8 个字符
        assert_eq!(short1.len(), 8);
    }

    #[test]
    fn test_hash_consistency() {
        // 测试多次调用的一致性
        let content = "AgentMem 是一个智能记忆管理平台";

        let hash1 = compute_content_hash(content);
        let hash2 = compute_content_hash(content);
        let hash3 = compute_content_hash(content);

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_empty_content() {
        let hash_empty = compute_content_hash("");
        let hash_space = compute_content_hash(" ");

        // 空字符串和空格应该产生不同的 hash
        assert_ne!(hash_empty, hash_space);
    }

    #[test]
    fn test_unicode_content() {
        let hash1 = compute_content_hash("你好世界");
        let hash2 = compute_content_hash("Hello World");

        // 不同语言应该产生不同的 hash
        assert_ne!(hash1, hash2);
    }
}
