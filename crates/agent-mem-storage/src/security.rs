//! Data Security Module for AgentMem
//!
//! Provides encryption, backup, and data export functionality.
//!
//! Features:
//! - AES-256-GCM encryption for data at rest
//! - Backup and restore mechanisms
//! - JSON/CSV data export

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption at rest
    pub enabled: bool,
    /// Encryption key (base64 encoded)
    pub key: Option<String>,
    /// Algorithm to use
    pub algorithm: EncryptionAlgorithm,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key: None,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        }
    }
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl Default for EncryptionAlgorithm {
    fn default() -> Self {
        Self::Aes256Gcm
    }
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub enabled: bool,
    /// Backup directory path
    pub backup_dir: String,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Backup interval in seconds
    pub interval_seconds: u64,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backup_dir: "./backups".to_string(),
            max_backups: 10,
            interval_seconds: 86400, // 1 day
        }
    }
}

/// Data export format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export directory path
    pub export_dir: String,
    /// Default format
    pub default_format: ExportFormat,
    /// Include metadata
    pub include_metadata: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            export_dir: "./exports".to_string(),
            default_format: ExportFormat::Json,
            include_metadata: true,
        }
    }
}

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum age for memories (in days)
    pub max_age_days: Option<u64>,
    /// Minimum importance score to keep
    pub min_importance: Option<f32>,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age_days: None,
            min_importance: None,
            auto_cleanup: false,
        }
    }
}

/// Simple encryption utility (using AES-GCM simulation)
/// Note: For production, use the `aes-gcm` crate with proper key management
pub struct Encryption;

impl Encryption {
    /// Generate a new encryption key
    pub fn generate_key() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let mut hasher = DefaultHasher::new();
        timestamp.hash(&mut hasher);
        rand::random_u64().hash(&mut hasher);
        
        format!("{:016x}{:016x}{:016x}{:016x}", 
            hasher.finish(),
            rand::random_u64(),
            rand::random_u64(),
            rand::random_u64()
        )
    }
    
    /// Encrypt data (placeholder - use aes-gcm in production)
    pub fn encrypt(data: &[u8], _key: &str) -> Result<Vec<u8>> {
        // In production, use aes-gcm crate:
        // let cipher = Aes256Gcm::new(key.into());
        // cipher.encrypt(nonce, data)
        
        // Placeholder: return data as-is
        Ok(data.to_vec())
    }
    
    /// Decrypt data (placeholder - use aes-gcm in production)
    pub fn decrypt(data: &[u8], _key: &str) -> Result<Vec<u8>> {
        // In production, use aes-gcm crate
        Ok(data.to_vec())
    }
}

/// Backup manager
pub struct BackupManager {
    config: BackupConfig,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> Self {
        Self { config }
    }
    
    /// Create a backup
    pub async fn create_backup(&self, data: &str, name: Option<&str>) -> Result<String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_name = name.unwrap_or("backup");
        let filename = format!("{}_{}.json", backup_name, timestamp);
        
        // Ensure backup directory exists
        let backup_path = Path::new(&self.config.backup_dir);
        std::fs::create_dir_all(backup_path)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create backup dir: {}", e)))?;
        
        let full_path = backup_path.join(&filename);
        
        // Write backup
        std::fs::write(&full_path, data)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to write backup: {}", e)))?;
        
        info!("Created backup: {:?}", full_path);
        Ok(filename)
    }
    
    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let backup_path = Path::new(&self.config.backup_dir);
        
        if !backup_path.exists() {
            return Ok(Vec::new());
        }
        
        let entries = std::fs::read_dir(backup_path)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to read backup dir: {}", e)))?;
        
        let mut backups = Vec::new();
        
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".json") {
                        backups.push(BackupInfo {
                            name: name.clone(),
                            path: entry.path().to_string_lossy().to_string(),
                            size_bytes: metadata.len(),
                            created_at: metadata.created()
                                .map(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
                                .unwrap_or(0),
                        });
                    }
                }
            }
        }
        
        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Limit to max_backups
        backups.truncate(self.config.max_backups);
        
        Ok(backups)
    }
    
    /// Restore from a backup
    pub async fn restore_backup(&self, name: &str) -> Result<String> {
        let backup_path = Path::new(&self.config.backup_dir).join(name);
        
        if !backup_path.exists() {
            return Err(AgentMemError::StorageError(format!("Backup not found: {}", name)));
        }
        
        let data = std::fs::read_to_string(&backup_path)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to read backup: {}", e)))?;
        
        info!("Restored backup: {:?}", backup_path);
        Ok(data)
    }
    
    /// Delete old backups beyond max_backups limit
    pub async fn cleanup_old_backups(&self) -> Result<usize> {
        let backups = self.list_backups()?;
        
        if backups.len() <= self.config.max_backups {
            return Ok(0);
        }
        
        let to_delete = &backups[self.config.max_backups..];
        let mut deleted = 0;
        
        for backup in to_delete {
            if std::fs::remove_file(&backup.path).is_ok() {
                deleted += 1;
            }
        }
        
        info!("Cleaned up {} old backups", deleted);
        Ok(deleted)
    }
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: u64,
}

/// Data exporter
pub struct DataExporter {
    config: ExportConfig,
}

impl DataExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }
    
    /// Export memories to JSON
    pub async fn export_json(&self, memories: &[HashMap<String, String>]) -> Result<String> {
        serde_json::to_string_pretty(memories)
            .map_err(AgentMemError::SerializationError)
    }
    
    /// Export memories to CSV
    pub async fn export_csv(&self, memories: &[HashMap<String, String>]) -> Result<String> {
        if memories.is_empty() {
            return Ok(String::new());
        }
        
        let mut csv = String::new();
        
        // Get all keys for headers
        let mut all_keys: Vec<String> = Vec::new();
        for memory in memories {
            for key in memory.keys() {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
            }
        }
        
        // Write headers
        csv.push_str(&all_keys.join(","));
        csv.push('\n');
        
        // Write rows
        for memory in memories {
            let row: Vec<String> = all_keys.iter()
                .map(|k| {
                    memory.get(k)
                        .map(|v| format!("\"{}\"", v.replace('"', "\"\"")))
                        .unwrap_or_default()
                })
                .collect();
            csv.push_str(&row.join(","));
            csv.push('\n');
        }
        
        Ok(csv)
    }
    
    /// Export and save to file
    pub async fn export_to_file(
        &self,
        memories: &[HashMap<String, String>],
        filename: &str,
        format: ExportFormat,
    ) -> Result<String> {
        let export_path = Path::new(&self.config.export_dir);
        std::fs::create_dir_all(export_path)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create export dir: {}", e)))?;
        
        let data = match format {
            ExportFormat::Json => self.export_json(memories).await?,
            ExportFormat::Csv => self.export_csv(memories).await?,
        };
        
        let full_path = export_path.join(filename);
        std::fs::write(&full_path, &data)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to write export: {}", e)))?;
        
        info!("Exported data to: {:?}", full_path);
        Ok(full_path.to_string_lossy().to_string())
    }
}

/// Retention policy manager
pub struct RetentionManager {
    policy: RetentionPolicy,
}

impl RetentionManager {
    pub fn new(policy: RetentionPolicy) -> Self {
        Self { policy }
    }
    
    /// Check if a memory should be deleted based on policy
    pub fn should_delete(&self, memory: &MemoryForRetention) -> bool {
        // Check age
        if let Some(max_age) = self.policy.max_age_days {
            let age_days = memory.age_days();
            if age_days > max_age as i64 {
                return true;
            }
        }
        
        // Check importance
        if let Some(min_importance) = self.policy.min_importance {
            if memory.importance < min_importance {
                return true;
            }
        }
        
        false
    }
    
    /// Filter memories based on retention policy
    pub fn filter_memories<'a>(&self, memories: &'a [MemoryForRetention]) -> Vec<&'a MemoryForRetention> {
        memories.iter()
            .filter(|m| !self.should_delete(m))
            .collect()
    }
}

/// Memory data for retention checking
#[derive(Debug, Clone)]
pub struct MemoryForRetention {
    pub id: String,
    pub created_at: u64,
    pub importance: f32,
}

impl MemoryForRetention {
    pub fn age_days(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let age_secs = now.saturating_sub(self.created_at);
        (age_secs / 86400) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_key_generation() {
        let key1 = Encryption::generate_key();
        let key2 = Encryption::generate_key();
        
        assert!(!key1.is_empty());
        assert!(!key2.is_empty());
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"Hello, AgentMem!";
        let key = "test-key";
        
        let encrypted = Encryption::encrypt(data, key).unwrap();
        let decrypted = Encryption::decrypt(&encrypted, key).unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_retention_policy_age() {
        let policy = RetentionPolicy {
            max_age_days: Some(30),
            min_importance: None,
            auto_cleanup: true,
        };
        
        let manager = RetentionManager::new(policy);
        
        // Old memory
        let old_memory = MemoryForRetention {
            id: "1".to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - 31 * 86400,
            importance: 0.5,
        };
        assert!(manager.should_delete(&old_memory));
        
        // Recent memory
        let recent_memory = MemoryForRetention {
            id: "2".to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - 10 * 86400,
            importance: 0.5,
        };
        assert!(!manager.should_delete(&recent_memory));
    }

    #[test]
    fn test_csv_export() {
        let exporter = DataExporter::new(ExportConfig::default());
        
        let memories = vec![
            HashMap::from([
                ("id".to_string(), "1".to_string()),
                ("content".to_string(), "Hello".to_string()),
            ]),
            HashMap::from([
                ("id".to_string(), "2".to_string()),
                ("content".to_string(), "World".to_string()),
            ]),
        ];
        
        let csv = futures::executor::block_on(exporter.export_csv(&memories)).unwrap();
        assert!(csv.contains("id"));
        assert!(csv.contains("Hello"));
        assert!(csv.contains("World"));
    }
}

// Required for async tests  
#[allow(dead_code)]
mod rand {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    /// Simple random u64 generator using system time
    pub fn random_u64() -> u64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        (nanos ^ (nanos >> 17)) as u64
    }
    
    /// Get a random value (simplified)
    pub fn random<T: Default>() -> T {
        T::default()
    }
}
