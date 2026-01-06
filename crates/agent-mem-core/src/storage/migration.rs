// Data migration utilities for AgentMem storage backends

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use super::{CacheBackend, StorageBackend};
use crate::hierarchy::{HierarchicalMemory, MemoryLevel, MemoryScope};
use crate::{CoreError, CoreResult};

/// Migration progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationProgress {
    /// Total number of items to migrate
    pub total_items: u64,
    /// Number of items successfully migrated
    pub migrated_items: u64,
    /// Number of items that failed to migrate
    pub failed_items: u64,
    /// Migration start time
    pub started_at: chrono::DateTime<Utc>,
    /// Migration completion time (if finished)
    pub completed_at: Option<chrono::DateTime<Utc>>,
    /// Current migration status
    pub status: MigrationStatus,
    /// Error messages for failed items
    pub errors: Vec<String>,
}

/// Migration status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationStatus {
    /// Migration is in progress
    InProgress,
    /// Migration completed successfully
    Completed,
    /// Migration failed
    Failed,
    /// Migration was cancelled
    Cancelled,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Batch size for migration operations
    pub batch_size: usize,
    /// Delay between batches in milliseconds
    pub batch_delay_ms: u64,
    /// Maximum number of retry attempts for failed items
    pub max_retries: u32,
    /// Whether to continue migration on individual item failures
    pub continue_on_error: bool,
    /// Whether to verify migrated data
    pub verify_data: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_delay_ms: 100,
            max_retries: 3,
            continue_on_error: true,
            verify_data: true,
        }
    }
}

/// Data migration manager
pub struct MigrationManager {
    config: MigrationConfig,
}

impl MigrationManager {
    /// Create new migration manager
    pub fn new(config: MigrationConfig) -> Self {
        Self { config }
    }

    /// Migrate data from in-memory storage to persistent storage
    pub async fn migrate_to_persistent(
        &self,
        source: &HashMap<String, HierarchicalMemory>,
        target: &dyn StorageBackend,
    ) -> CoreResult<MigrationProgress> {
        let total_items = source.len() as u64;
        let mut progress = MigrationProgress {
            total_items,
            migrated_items: 0,
            failed_items: 0,
            started_at: Utc::now(),
            completed_at: None,
            status: MigrationStatus::InProgress,
            errors: Vec::new(),
        };

        info!(
            "Starting migration of {} items to persistent storage",
            total_items
        );

        // Initialize target storage
        if let Err(e) = target.initialize().await {
            error!("Failed to initialize target storage: {}", e);
            progress.status = MigrationStatus::Failed;
            progress
                .errors
                .push(format!("Storage initialization failed: {}", e));
            return Ok(progress);
        }

        // Process items in batches
        let items: Vec<_> = source.iter().collect();
        let batches: Vec<_> = items.chunks(self.config.batch_size).collect();

        for (batch_idx, batch) in batches.iter().enumerate() {
            info!("Processing batch {} of {}", batch_idx + 1, batches.len());

            for (id, memory) in batch.iter() {
                let mut retries = 0;
                let mut success = false;

                while retries <= self.config.max_retries && !success {
                    match target.store_memory(memory).await {
                        Ok(_) => {
                            progress.migrated_items += 1;
                            success = true;

                            // Verify data if enabled
                            if self.config.verify_data {
                                match target.get_memory(id).await {
                                    Ok(Some(retrieved)) => {
                                        if retrieved.memory.id != memory.memory.id
                                            || retrieved.memory.content != memory.memory.content
                                        {
                                            warn!("Data verification failed for memory {}", id);
                                            progress
                                                .errors
                                                .push(format!("Verification failed for {}", id));
                                        }
                                    }
                                    Ok(None) => {
                                        warn!("Memory {} not found after migration", id);
                                        progress.errors.push(format!(
                                            "Memory {} not found after migration",
                                            id
                                        ));
                                    }
                                    Err(e) => {
                                        warn!("Failed to verify memory {}: {}", id, e);
                                        progress
                                            .errors
                                            .push(format!("Verification error for {}: {}", id, e));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            retries += 1;
                            if retries > self.config.max_retries {
                                error!(
                                    "Failed to migrate memory {} after {} retries: {}",
                                    id, self.config.max_retries, e
                                );
                                progress.failed_items += 1;
                                progress
                                    .errors
                                    .push(format!("Failed to migrate {}: {}", id, e));

                                if !self.config.continue_on_error {
                                    progress.status = MigrationStatus::Failed;
                                    return Ok(progress);
                                }
                            } else {
                                warn!("Retry {} for memory {}: {}", retries, id, e);
                                sleep(Duration::from_millis(100 * retries as u64)).await;
                            }
                        }
                    }
                }
            }

            // Delay between batches
            if batch_idx < batches.len() - 1 && self.config.batch_delay_ms > 0 {
                sleep(Duration::from_millis(self.config.batch_delay_ms)).await;
            }
        }

        progress.completed_at = Some(Utc::now());
        progress.status = if progress.failed_items == 0 {
            MigrationStatus::Completed
        } else if progress.migrated_items > 0 {
            MigrationStatus::Completed // Partial success
        } else {
            MigrationStatus::Failed
        };

        info!(
            "Migration completed: {} successful, {} failed out of {} total",
            progress.migrated_items, progress.failed_items, progress.total_items
        );

        Ok(progress)
    }

    /// Migrate data between storage backends
    pub async fn migrate_between_backends(
        &self,
        source: &dyn StorageBackend,
        target: &dyn StorageBackend,
    ) -> CoreResult<MigrationProgress> {
        info!("Starting migration between storage backends");

        // Initialize target storage
        if let Err(e) = target.initialize().await {
            error!("Failed to initialize target storage: {}", e);
            return Err(CoreError::MigrationError(format!(
                "Target initialization failed: {}",
                e
            )));
        }

        // Get statistics from source to estimate total items
        let source_stats = source.get_statistics().await?;
        let total_items = source_stats.total_memories;

        let mut progress = MigrationProgress {
            total_items,
            migrated_items: 0,
            failed_items: 0,
            started_at: Utc::now(),
            completed_at: None,
            status: MigrationStatus::InProgress,
            errors: Vec::new(),
        };

        // For now, we'll implement a simple approach
        // In a production system, you'd want to implement pagination
        // and more sophisticated batching strategies

        // This is a simplified implementation - in practice you'd need
        // to implement proper pagination for large datasets
        warn!("Backend-to-backend migration is not fully implemented yet");
        progress.status = MigrationStatus::Failed;
        progress
            .errors
            .push("Backend-to-backend migration not implemented".to_string());

        Ok(progress)
    }

    /// Warm up cache from persistent storage
    pub async fn warm_cache(
        &self,
        storage: &dyn StorageBackend,
        cache: &dyn CacheBackend,
        limit: Option<usize>,
    ) -> CoreResult<MigrationProgress> {
        info!("Starting cache warm-up from persistent storage");

        let mut progress = MigrationProgress {
            total_items: 0,
            migrated_items: 0,
            failed_items: 0,
            started_at: Utc::now(),
            completed_at: None,
            status: MigrationStatus::InProgress,
            errors: Vec::new(),
        };

        // Get recent memories to warm the cache
        // This is a simplified implementation - you might want to implement
        // more sophisticated cache warming strategies based on access patterns

        // For now, we'll get memories by different scopes and levels

        let scopes = [
            MemoryScope::Global,
            MemoryScope::Agent("default".to_string()),
            MemoryScope::User {
                agent_id: "default".to_string(),
                user_id: "default".to_string(),
            },
            MemoryScope::Session {
                agent_id: "default".to_string(),
                user_id: "default".to_string(),
                session_id: "default".to_string(),
            },
        ];
        let levels = [
            MemoryLevel::Strategic,
            MemoryLevel::Tactical,
            MemoryLevel::Operational,
        ];

        for scope in &scopes {
            match storage.get_memories_by_scope(scope.clone(), limit).await {
                Ok(memories) => {
                    progress.total_items += memories.len() as u64;

                    for memory in memories {
                        match cache.set(&memory.memory.id, &memory, None).await {
                            Ok(_) => progress.migrated_items += 1,
                            Err(e) => {
                                progress.failed_items += 1;
                                progress
                                    .errors
                                    .push(format!("Failed to cache {}: {}", memory.memory.id, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get memories for scope {:?}: {}", scope, e);
                    progress.errors.push(format!(
                        "Failed to get memories for scope {:?}: {}",
                        scope, e
                    ));
                }
            }
        }

        for level in &levels {
            match storage.get_memories_by_level(level.clone(), limit).await {
                Ok(memories) => {
                    for memory in memories {
                        // Only cache if not already cached
                        match cache.exists(&memory.memory.id).await {
                            Ok(false) => {
                                progress.total_items += 1;
                                match cache.set(&memory.memory.id, &memory, None).await {
                                    Ok(_) => progress.migrated_items += 1,
                                    Err(e) => {
                                        progress.failed_items += 1;
                                        progress.errors.push(format!(
                                            "Failed to cache {}: {}",
                                            memory.memory.id, e
                                        ));
                                    }
                                }
                            }
                            Ok(true) => {
                                // Already cached, skip
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to check cache existence for {}: {}",
                                    memory.memory.id, e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get memories for level {:?}: {}", level, e);
                    progress.errors.push(format!(
                        "Failed to get memories for level {:?}: {}",
                        level, e
                    ));
                }
            }
        }

        progress.completed_at = Some(Utc::now());
        progress.status = if progress.failed_items == 0 {
            MigrationStatus::Completed
        } else if progress.migrated_items > 0 {
            MigrationStatus::Completed // Partial success
        } else {
            MigrationStatus::Failed
        };

        info!(
            "Cache warm-up completed: {} items cached, {} failed",
            progress.migrated_items, progress.failed_items
        );

        Ok(progress)
    }

    /// Export data to JSON format
    pub async fn export_to_json(
        &self,
        storage: &dyn StorageBackend,
        file_path: &str,
    ) -> CoreResult<MigrationProgress> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        info!("Starting data export to JSON file: {}", file_path);

        let mut progress = MigrationProgress {
            total_items: 0,
            migrated_items: 0,
            failed_items: 0,
            started_at: Utc::now(),
            completed_at: None,
            status: MigrationStatus::InProgress,
            errors: Vec::new(),
        };

        // Get statistics to estimate total items
        let stats = storage.get_statistics().await?;
        progress.total_items = stats.total_memories;

        // Create output file
        let mut file = File::create(file_path)
            .await
            .map_err(|e| CoreError::IoError(format!("Failed to create export file: {}", e)))?;

        // Write JSON array start
        file.write_all(b"[\n")
            .await
            .map_err(|e| CoreError::IoError(format!("Failed to write to export file: {}", e)))?;

        // This is a simplified implementation
        // In practice, you'd want to implement proper streaming export
        // for large datasets to avoid memory issues

        warn!("JSON export is not fully implemented yet");
        progress.status = MigrationStatus::Failed;
        progress
            .errors
            .push("JSON export not implemented".to_string());

        Ok(progress)
    }
}
