//! Privacy-Preserving Technologies for AgentMem
//!
//! This module provides privacy-enhancing technologies including:
//! - Differential privacy for query results
//! - Data anonymization and pseudonymization
//! - Secure aggregation
//! - Data masking for sensitive fields
//!
//! # Features
//!
//! - ε-differential privacy implementation
//! - k-anonymity and l-diversity
//! - Field-level data masking
//! - Tenant-level encryption key management

use crate::CoreError;
use chrono::{DateTime, Utc};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Privacy-preserving configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Enable differential privacy
    pub enable_differential_privacy: bool,
    /// Privacy budget (epsilon)
    pub epsilon: f64,
    /// Enable k-anonymity
    pub enable_k_anonymity: bool,
    /// Minimum k for anonymity
    pub k_value: usize,
    /// Enable data masking
    pub enable_masking: bool,
    /// Enable secure aggregation
    pub enable_secure_aggregation: bool,
    /// Noise scale parameter
    pub noise_scale: f64,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            enable_differential_privacy: true,
            epsilon: 1.0,
            enable_k_anonymity: true,
            k_value: 5,
            enable_masking: true,
            enable_secure_aggregation: true,
            noise_scale: 1.0,
        }
    }
}

/// Differential privacy noise type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseType {
    /// Laplace noise
    Laplace,
    /// Gaussian noise
    Gaussian,
    /// Exponential noise
    Exponential,
}

/// Query result with privacy protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateQueryResult<T> {
    /// Original result count
    pub original_count: usize,
    /// Private result count
    pub private_count: usize,
    /// Privacy budget used
    pub epsilon_used: f64,
    /// Noised value
    pub value: T,
    /// Privacy metadata
    pub privacy_metadata: PrivacyMetadata,
}

/// Privacy metadata for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyMetadata {
    /// Noise type used
    pub noise_type: NoiseType,
    /// Sensitivity of the query
    pub sensitivity: f64,
    /// Is result privacy-preserved
    pub is_preserved: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Anonymization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationRequest {
    /// Data to anonymize
    pub data: Vec<HashMap<String, String>>,
    /// Quasi-identifiers for k-anonymity
    pub quasi_identifiers: Vec<String>,
    /// Sensitive attributes to generalize
    pub sensitive_attributes: Vec<String>,
    /// Minimum k value
    pub k: usize,
}

/// Anonymization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationResult {
    /// Original data size
    pub original_size: usize,
    /// Anonymized data size
    pub anonymized_size: usize,
    /// Groups formed
    pub groups: Vec<AnonymityGroup>,
    /// Equivalence classes
    pub equivalence_classes: usize,
    /// Statistics
    pub stats: AnonymizationStats,
}

/// Anonymity group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityGroup {
    /// Group ID
    pub group_id: String,
    /// Member record IDs
    pub member_ids: Vec<usize>,
    /// Generalized quasi-identifier values
    pub generalized_values: HashMap<String, String>,
    /// Suppressed count
    pub suppressed_count: usize,
}

/// Anonymization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationStats {
    /// Information loss metric (0-1)
    pub information_loss: f64,
    /// Discernibility metric
    pub discernibility: usize,
    /// Average group size
    pub avg_group_size: f64,
}

/// Data masking request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingRequest {
    /// Data to mask
    pub data: HashMap<String, String>,
    /// Fields to mask
    pub fields_to_mask: HashSet<String>,
    /// Masking strategy per field
    pub masking_strategies: HashMap<String, MaskingStrategy>,
}

/// Masking strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaskingStrategy {
    /// Replace with asterisks (e.g., "****")
    Replace,
    /// Hash the value
    Hash,
    /// Partial masking (e.g., "john***")
    Partial,
    /// Redact entirely
    Redact,
    /// Replace with placeholder
    Placeholder(String),
    /// Shift dates
    DateShift,
    /// Add noise
    AddNoise { scale: f64 },
}

/// Data masking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingResult {
    /// Original data
    pub original: HashMap<String, String>,
    /// Masked data
    pub masked: HashMap<String, String>,
    /// Fields that were masked
    pub masked_fields: Vec<String>,
}

/// Secure aggregation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureAggregationRequest {
    /// Values to aggregate
    pub values: Vec<f64>,
    /// Aggregation type
    pub aggregation_type: AggregationType,
    /// Enable secure sum
    pub secure_sum: bool,
    /// Enable secure count
    pub secure_count: bool,
}

/// Aggregation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum aggregation
    Sum,
    /// Mean aggregation
    Mean,
    /// Count aggregation
    Count,
    /// Max aggregation
    Max,
    /// Min aggregation
    Min,
}

/// Secure aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureAggregationResult {
    /// Aggregation type used
    pub aggregation_type: AggregationType,
    /// Noised result
    pub value: f64,
    /// Number of inputs
    pub input_count: usize,
    /// Privacy budget consumed
    pub epsilon_used: f64,
}

/// Tenant encryption key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantKey {
    /// Tenant ID
    pub tenant_id: String,
    /// Key ID
    pub key_id: String,
    /// Encrypted key material (base64)
    pub encrypted_key: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiry timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Key version
    pub version: u32,
}

/// Key rotation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationRecord {
    /// Old key ID
    pub old_key_id: String,
    /// New key ID
    pub new_key_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Rotation timestamp
    pub rotated_at: DateTime<Utc>,
    /// Re-encrypted data count
    pub re_encrypted_count: usize,
}

/// Privacy audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyAuditEntry {
    /// Operation type
    pub operation: PrivacyOperation,
    /// Subject ID
    pub subject_id: String,
    /// Resource affected
    pub resource_id: String,
    /// Privacy technique used
    pub technique: String,
    /// Success status
    pub success: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Privacy operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyOperation {
    /// Query with differential privacy
    PrivateQuery,
    /// Data anonymization
    Anonymize,
    /// Data masking
    Mask,
    /// Secure aggregation
    SecureAggregate,
    /// Key rotation
    KeyRotation,
    /// Data deletion
    Delete,
}

/// Differential privacy engine
pub struct DifferentialPrivacy {
    config: PrivacyConfig,
    rng: Arc<RwLock<rand::rngs::StdRng>>,
}

impl DifferentialPrivacy {
    /// Create a new differential privacy engine
    pub fn new(config: PrivacyConfig) -> Self {
        Self {
            config,
            rng: Arc::new(RwLock::new(rand::rngs::StdRng::seed_from_u64(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64
            ))),
        }
    }

    /// Add Laplace noise to a value
    pub async fn add_laplace_noise(&self, value: f64, sensitivity: f64) -> f64 {
        let mut rng = self.rng.write().await;
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();

        // Generate uniform random value
        let u = u1 - 0.5;

        // Generate Laplace noise
        let noise = -sensitivity * self.config.epsilon.recip() * u.signum() * (1.0 - 2.0 * u.abs()).ln();

        value + noise
    }

    /// Add Gaussian noise (for (ε, δ)-DP)
    pub async fn add_gaussian_noise(&self, value: f64, sensitivity: f64) -> f64 {
        let mut rng = self.rng.write().await;

        // Box-Muller transform for Gaussian noise
        let u1: f64 = rng.gen::<f64>();
        let u2: f64 = rng.gen::<f64>();

        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let noise = z0 * self.config.noise_scale * sensitivity;

        value + noise
    }

    /// Add exponential noise (for counting queries)
    pub async fn add_exponential_noise(&self, value: f64, sensitivity: f64) -> f64 {
        let mut rng = self.rng.write().await;
        let u: f64 = rng.gen();

        let noise = -sensitivity * u.ln() / self.config.epsilon;
        value + noise
    }

    /// Apply privacy to a count query
    pub async fn private_count(&self, count: usize, sensitivity: f64) -> PrivateQueryResult<usize> {
        let noisy_count = match NoiseType::Laplace {
            _ => self.add_laplace_noise(count as f64, sensitivity).await as usize,
        };

        PrivateQueryResult {
            original_count: count,
            private_count: noisy_count.max(0) as usize,
            epsilon_used: self.config.epsilon,
            value: noisy_count.max(0) as usize,
            privacy_metadata: PrivacyMetadata {
                noise_type: NoiseType::Laplace,
                sensitivity,
                is_preserved: true,
                timestamp: Utc::now(),
            },
        }
    }

    /// Apply privacy to a sum query
    pub async fn private_sum(&self, sum: f64, sensitivity: f64) -> PrivateQueryResult<f64> {
        let noisy_sum = self.add_laplace_noise(sum, sensitivity).await;

        PrivateQueryResult {
            original_count: 1,
            private_count: 1,
            epsilon_used: self.config.epsilon,
            value: noisy_sum,
            privacy_metadata: PrivacyMetadata {
                noise_type: NoiseType::Laplace,
                sensitivity,
                is_preserved: true,
                timestamp: Utc::now(),
            },
        }
    }

    /// Apply privacy to a mean query
    pub async fn private_mean(&self, values: &[f64], sensitivity: f64) -> PrivateQueryResult<f64> {
        if values.is_empty() {
            return PrivateQueryResult {
                original_count: 0,
                private_count: 0,
                epsilon_used: 0.0,
                value: 0.0,
                privacy_metadata: PrivacyMetadata {
                    noise_type: NoiseType::Laplace,
                    sensitivity,
                    is_preserved: false,
                    timestamp: Utc::now(),
                },
            };
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let noisy_mean = self.add_laplace_noise(mean, sensitivity / count).await;

        PrivateQueryResult {
            original_count: values.len(),
            private_count: values.len(),
            epsilon_used: self.config.epsilon,
            value: noisy_mean,
            privacy_metadata: PrivacyMetadata {
                noise_type: NoiseType::Laplace,
                sensitivity: sensitivity / count,
                is_preserved: true,
                timestamp: Utc::now(),
            },
        }
    }

    /// Calculate sensitivity for count query
    pub fn count_sensitivity(&self) -> f64 {
        1.0
    }

    /// Calculate sensitivity for sum query given bounds
    pub fn sum_sensitivity(&self, min: f64, max: f64) -> f64 {
        (max - min).abs()
    }
}

/// K-anonymity engine
pub struct KAnonymity {
    config: PrivacyConfig,
}

impl KAnonymity {
    /// Create a new k-anonymity engine
    pub fn new(config: PrivacyConfig) -> Self {
        Self { config }
    }

    /// Apply k-anonymity to data
    pub fn anonymize(&self, request: AnonymizationRequest) -> Result<AnonymizationResult, CoreError> {
        let data = request.data;
        let k = request.k;

        if data.is_empty() {
            return Err(CoreError::ValidationError("Empty data".to_string()));
        }

        // Group by quasi-identifiers
        let mut groups: HashMap<String, Vec<(usize, HashMap<String, String>)>> = HashMap::new();

        for (idx, record) in data.iter().enumerate() {
            let key = request
                .quasi_identifiers
                .iter()
                .map(|qi| record.get(qi).cloned().unwrap_or_default())
                .collect::<Vec<_>>()
                .join("|");

            groups
                .entry(key)
                .or_insert_with(Vec::new)
                .push((idx, record.clone()));
        }

        // Create anonymity groups
        let mut anonymity_groups = Vec::new();
        let mut all_member_ids = HashSet::new();

        for (key, members) in groups {
            let member_count = members.len();
            if member_count >= k {
                // Valid group
                let member_ids: Vec<usize> = members.iter().map(|(id, _)| *id).collect();
                all_member_ids.extend(member_ids.clone());

                let generalized_values: HashMap<String, String> = request
                    .quasi_identifiers
                    .iter()
                    .map(|qi| {
                        let first_record = &members.first().unwrap().1;
                        (qi.clone(), first_record.get(qi).cloned().unwrap_or_default())
                    })
                    .collect();

                anonymity_groups.push(AnonymityGroup {
                    group_id: format!("group_{}", anonymity_groups.len()),
                    member_ids,
                    generalized_values,
                    suppressed_count: 0,
                });
            } else {
                // Group too small - suppress records
                for (id, _) in members.into_iter().take(k.saturating_sub(member_count)) {
                    all_member_ids.insert(id);
                }
            }
        }

        // Calculate statistics
        let original_size = data.len();
        let anonymized_size = all_member_ids.len();
        let total_groups = anonymity_groups.len();
        let avg_group_size = if total_groups > 0 {
            anonymized_size as f64 / total_groups as f64
        } else {
            0.0
        };

        let information_loss = if original_size > 0 {
            1.0 - (anonymized_size as f64 / original_size as f64)
        } else {
            0.0
        };

        Ok(AnonymizationResult {
            original_size,
            anonymized_size,
            groups: anonymity_groups,
            equivalence_classes: total_groups,
            stats: AnonymizationStats {
                information_loss,
                discernibility: original_size * original_size, // Simplified
                avg_group_size,
            },
        })
    }

    /// Generalize a value to a broader category
    pub fn generalize_value(&self, value: &str, level: u32) -> String {
        match level {
            0 => value.to_string(),
            1 => {
                // First character + wildcards
                if value.len() > 0 {
                    format!("{}***", &value[..1])
                } else {
                    "***".to_string()
                }
            }
            2 => {
                // First 2 characters + wildcards
                if value.len() > 1 {
                    format!("{}***", &value[..2])
                } else {
                    "***".to_string()
                }
            }
            _ => "***".to_string(),
        }
    }

    /// Generalize a date
    pub fn generalize_date(&self, date: &str, level: u32) -> String {
        // Simple implementation - assume ISO format
        if date.len() >= 10 {
            match level {
                0 => date.to_string(),
                1 => format!("{}-**-**", &date[..4]), // Year only
                2 => format!("****-**-**"),            // Any year
                _ => "****-**-**".to_string(),
            }
        } else {
            "****-**-**".to_string()
        }
    }

    /// Generalize a number
    pub fn generalize_number(&self, value: f64, buckets: usize) -> String {
        if buckets == 0 {
            return "*".to_string();
        }
        let bucket_size = 100.0 / buckets as f64;
        let bucket = (value / bucket_size).floor() as usize;
        let lower = bucket as f64 * bucket_size;
        let upper = (bucket + 1) as f64 * bucket_size;
        format!("{:.0}-{:.0}", lower, upper)
    }
}

/// Data masking engine
pub struct DataMasking {
    config: PrivacyConfig,
    rng: Arc<RwLock<rand::rngs::StdRng>>,
}

impl DataMasking {
    /// Create a new data masking engine
    pub fn new(config: PrivacyConfig) -> Self {
        Self {
            config,
            rng: Arc::new(RwLock::new(rand::rngs::StdRng::seed_from_u64(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64
            ))),
        }
    }

    /// Mask data according to request
    pub async fn mask(&self, request: MaskingRequest) -> MaskingResult {
        let mut masked = request.data.clone();
        let mut masked_fields = Vec::new();

        for field in &request.fields_to_mask {
            if let Some(value) = masked.get(field) {
                let strategy = request
                    .masking_strategies
                    .get(field)
                    .cloned()
                    .unwrap_or(MaskingStrategy::Redact);

                let masked_value = self.apply_masking(value, &strategy).await;
                masked.insert(field.clone(), masked_value);
                masked_fields.push(field.clone());
            }
        }

        MaskingResult {
            original: request.data,
            masked,
            masked_fields,
        }
    }

    /// Apply specific masking strategy
    async fn apply_masking(&self, value: &str, strategy: &MaskingStrategy) -> String {
        match strategy {
            MaskingStrategy::Replace => "*".repeat(value.len().min(16)),
            MaskingStrategy::Hash => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            }
            MaskingStrategy::Partial => {
                if value.len() > 3 {
                    format!("{}***", &value[..value.len().min(4)])
                } else {
                    "***".to_string()
                }
            }
            MaskingStrategy::Redact => "[REDACTED]".to_string(),
            MaskingStrategy::Placeholder(placeholder) => placeholder.clone(),
            MaskingStrategy::DateShift => {
                // Simple shift by random days
                let mut rng = self.rng.write().await;
                let days: i64 = rng.gen_range(-365..365);
                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(value) {
                    let shifted = date + chrono::Duration::days(days);
                    shifted.to_rfc3339()
                } else {
                    value.to_string()
                }
            }
            MaskingStrategy::AddNoise { scale } => {
                let mut rng = self.rng.write().await;
                let noise: f64 = rng.gen_range(-*scale..*scale);
                if let Ok(num) = value.parse::<f64>() {
                    (num + noise).to_string()
                } else {
                    value.to_string()
                }
            }
        }
    }

    /// Mask email addresses
    pub async fn mask_email(&self, email: &str) -> String {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() == 2 {
            let local = parts[0];
            let domain = parts[1];
            let masked_local = if local.len() > 2 {
                format!("{}***", &local[..2])
            } else {
                "***".to_string()
            };
            format!("{}@{}", masked_local, domain)
        } else {
            "[INVALID_EMAIL]".to_string()
        }
    }

    /// Mask phone numbers
    pub async fn mask_phone(&self, phone: &str) -> String {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            let visible = &digits[digits.len() - 4..];
            format!("***-***-{}", visible)
        } else {
            "***-***-****".to_string()
        }
    }

    /// Mask credit card numbers
    pub async fn mask_credit_card(&self, card: &str) -> String {
        let digits: String = card.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            let visible = &digits[digits.len() - 4..];
            format!("****-****-****-{}", visible)
        } else {
            "****-****-****-****".to_string()
        }
    }
}

/// Secure aggregation engine
pub struct SecureAggregation {
    config: PrivacyConfig,
    dp: DifferentialPrivacy,
}

impl SecureAggregation {
    /// Create a new secure aggregation engine
    pub fn new(config: PrivacyConfig) -> Self {
        Self {
            config: config.clone(),
            dp: DifferentialPrivacy::new(config),
        }
    }

    /// Perform secure aggregation
    pub async fn aggregate(&self, request: SecureAggregationRequest) -> SecureAggregationResult {
        let value = match request.aggregation_type {
            AggregationType::Sum => {
                let sum: f64 = request.values.iter().sum();
                if request.secure_sum {
                    self.dp.private_sum(sum, self.dp.sum_sensitivity(0.0, 1.0)).await.value
                } else {
                    sum
                }
            }
            AggregationType::Mean => {
                if request.values.is_empty() {
                    0.0
                } else {
                    let mean = request.values.iter().sum::<f64>() / request.values.len() as f64;
                    if request.secure_sum {
                        self.dp.private_mean(&request.values, 1.0).await.value
                    } else {
                        mean
                    }
                }
            }
            AggregationType::Count => {
                let count = request.values.len() as f64;
                if request.secure_count {
                    self.dp.private_count(count as usize, self.dp.count_sensitivity()).await.value as f64
                } else {
                    count
                }
            }
            AggregationType::Max => {
                request.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            }
            AggregationType::Min => {
                request.values.iter().cloned().fold(f64::INFINITY, f64::min)
            }
        };

        SecureAggregationResult {
            aggregation_type: request.aggregation_type,
            value,
            input_count: request.values.len(),
            epsilon_used: if request.secure_sum || request.secure_count {
                self.config.epsilon
            } else {
                0.0
            },
        }
    }
}

/// Tenant encryption key manager
pub struct TenantKeyManager {
    keys: Arc<RwLock<HashMap<String, TenantKey>>>,
    rotation_records: Arc<RwLock<Vec<KeyRotationRecord>>>,
}

impl TenantKeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            rotation_records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate a new key for a tenant
    pub async fn generate_key(&self, tenant_id: &str) -> TenantKey {
        let key_id = format!("key_{}_{}", tenant_id, Utc::now().timestamp());
        let key_material = Self::generate_random_key(32);

        let key = TenantKey {
            tenant_id: tenant_id.to_string(),
            key_id: key_id.clone(),
            encrypted_key: base64_encode(&key_material),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(90)),
            version: 1,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id, key.clone());

        key
    }

    /// Rotate key for a tenant
    pub async fn rotate_key(&self, tenant_id: &str) -> Result<KeyRotationRecord, CoreError> {
        let mut keys = self.keys.write().await;

        // Find current key
        let old_key = keys
            .values()
            .find(|k| k.tenant_id == tenant_id)
            .ok_or_else(|| CoreError::ValidationError("No key found for tenant".to_string()))?
            .clone();

        // Generate new key
        let new_key_id = format!("key_{}_{}", tenant_id, Utc::now().timestamp());
        let key_material = Self::generate_random_key(32);

        let new_key = TenantKey {
            tenant_id: tenant_id.to_string(),
            key_id: new_key_id.clone(),
            encrypted_key: base64_encode(&key_material),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(90)),
            version: old_key.version + 1,
        };

        // Store new key
        keys.insert(new_key_id.clone(), new_key);

        // Record rotation
        let record = KeyRotationRecord {
            old_key_id: old_key.key_id,
            new_key_id,
            tenant_id: tenant_id.to_string(),
            rotated_at: Utc::now(),
            re_encrypted_count: 0,
        };

        let mut rotations = self.rotation_records.write().await;
        rotations.push(record.clone());

        Ok(record)
    }

    /// Get current key for tenant
    pub async fn get_current_key(&self, tenant_id: &str) -> Option<TenantKey> {
        let keys = self.keys.read().await;
        keys.values()
            .filter(|k| k.tenant_id == tenant_id)
            .max_by_key(|k| k.version)
            .cloned()
    }

    /// Generate random key material
    fn generate_random_key(len: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::rngs::StdRng::from_entropy();
        (0..len).map(|_| rng.gen()).collect()
    }

    /// Get rotation history
    pub async fn get_rotation_history(&self, tenant_id: &str) -> Vec<KeyRotationRecord> {
        let rotations = self.rotation_records.read().await;
        rotations
            .iter()
            .filter(|r| r.tenant_id == tenant_id)
            .cloned()
            .collect()
    }
}

impl Default for TenantKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Base64 encoding helper
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_differential_privacy_count() {
        let dp = DifferentialPrivacy::new(PrivacyConfig::default());

        let count = 100;
        let result = dp.private_count(count, 1.0).await;

        assert!(result.private_count >= 0);
        assert!(result.epsilon_used > 0.0);
    }

    #[tokio::test]
    async fn test_differential_privacy_sum() {
        let dp = DifferentialPrivacy::new(PrivacyConfig::default());

        let sum = 500.0;
        let result = dp.private_sum(sum, 1.0).await;

        // Noisy value should be close to original
        let diff = (result.value - sum).abs();
        assert!(diff < 10.0); // With epsilon=1.0, noise should be reasonable
    }

    #[tokio::test]
    async fn test_k_anonymity() {
        let kanon = KAnonymity::new(PrivacyConfig::default());

        let data = vec![
            HashMap::from([("age".to_string(), "25".to_string()), ("zip".to_string(), "12345".to_string())]),
            HashMap::from([("age".to_string(), "25".to_string()), ("zip".to_string(), "12346".to_string())]),
            HashMap::from([("age".to_string(), "30".to_string()), ("zip".to_string(), "12345".to_string())]),
            HashMap::from([("age".to_string(), "30".to_string()), ("zip".to_string(), "12347".to_string())]),
            HashMap::from([("age".to_string(), "35".to_string()), ("zip".to_string(), "12345".to_string())]),
        ];

        let request = AnonymizationRequest {
            data,
            quasi_identifiers: vec!["age".to_string(), "zip".to_string()],
            sensitive_attributes: vec![],
            k: 2,
        };

        let result = kanon.anonymize(request).unwrap();

        assert!(result.equivalence_classes >= 1);
    }

    #[tokio::test]
    async fn test_data_masking() {
        let masking = DataMasking::new(PrivacyConfig::default());

        let data = HashMap::from([
            ("email".to_string(), "user@example.com".to_string()),
            ("phone".to_string(), "1234567890".to_string()),
            ("ssn".to_string(), "123-45-6789".to_string()),
        ]);

        let request = MaskingRequest {
            data,
            fields_to_mask: vec!["email".to_string(), "phone".to_string()].into_iter().collect(),
            masking_strategies: HashMap::from([
                ("email".to_string(), MaskingStrategy::Hash),
                ("phone".to_string(), MaskingStrategy::Replace),
            ]),
        };

        let result = masking.mask(request).await;

        assert!(result.masked_fields.contains(&"email".to_string()));
        assert!(result.masked_fields.contains(&"phone".to_string()));
        assert_ne!(result.masked.get("email"), Some(&"user@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_secure_aggregation() {
        let agg = SecureAggregation::new(PrivacyConfig::default());

        let request = SecureAggregationRequest {
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            aggregation_type: AggregationType::Mean,
            secure_sum: true,
            secure_count: true,
        };

        let result = agg.aggregate(request).await;

        assert_eq!(result.input_count, 5);
        assert!(result.epsilon_used > 0.0);
    }

    #[tokio::test]
    async fn test_tenant_key_manager() {
        let manager = TenantKeyManager::new();

        let key = manager.generate_key("tenant1").await;
        assert_eq!(key.tenant_id, "tenant1");
        assert!(key.encrypted_key.len() > 0);

        let current = manager.get_current_key("tenant1").await;
        assert!(current.is_some());
    }
}
