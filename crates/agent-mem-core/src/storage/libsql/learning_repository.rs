//! LibSQL Learning Repository
//!
//! Provides LibSQL implementation for learning feedback persistence

use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::search::adaptive::{QueryFeatures, SearchWeights};
use crate::search::learning::FeedbackRecord;

/// Trait for learning repository operations
#[async_trait]
pub trait LearningRepositoryTrait: Send + Sync {
    /// Create a new feedback record
    async fn create_feedback(&self, record: &FeedbackRecord) -> Result<()>;

    /// Get all feedback records
    async fn get_all_feedback(&self) -> Result<Vec<FeedbackRecord>>;

    /// Get feedback by pattern
    async fn get_feedback_by_pattern(&self, pattern: &str) -> Result<Vec<FeedbackRecord>>;

    /// Get recent feedback (last N records)
    async fn get_recent_feedback(&self, limit: usize) -> Result<Vec<FeedbackRecord>>;

    /// Delete old feedback records (older than timestamp)
    async fn delete_old_feedback(&self, before_timestamp: DateTime<Utc>) -> Result<usize>;

    /// Clear all feedback
    async fn clear_all_feedback(&self) -> Result<()>;

    /// Get feedback count by pattern
    async fn get_feedback_count_by_pattern(&self, pattern: &str) -> Result<usize>;
}

/// LibSQL implementation of Learning repository
pub struct LibSqlLearningRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlLearningRepository {
    /// Create a new LibSQL learning repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Helper function to convert row to FeedbackRecord
    fn row_to_feedback(row: &libsql::Row) -> Result<FeedbackRecord> {
        // Column order from SELECT:
        // 0: id, 1: query_pattern, 2: features, 3: vector_weight, 4: fulltext_weight,
        // 5: effectiveness, 6: timestamp, 7: user_id

        let features_json: String = row
            .get(2)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get features: {e}")))?;
        let features: QueryFeatures = serde_json::from_str(&features_json)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to parse features: {e}")))?;

        let vector_weight: f64 = row.get(3).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get vector_weight: {e}"))
        })?;
        let fulltext_weight: f64 = row.get(4).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get fulltext_weight: {e}"))
        })?;

        let effectiveness: f64 = row.get(5).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get effectiveness: {e}"))
        })?;

        let timestamp_ts: i64 = row
            .get(6)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get timestamp: {e}")))?;
        let timestamp = DateTime::from_timestamp(timestamp_ts, 0)
            .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?;

        Ok(FeedbackRecord {
            id: row
                .get(0)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
            query_pattern: row.get(1).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get query_pattern: {e}"))
            })?,
            features,
            weights: SearchWeights {
                vector_weight: vector_weight as f32,
                fulltext_weight: fulltext_weight as f32,
                confidence: 1.0,
            },
            effectiveness: effectiveness as f32,
            timestamp,
            user_id: row.get(7).ok(),
        })
    }
}

#[async_trait]
impl LearningRepositoryTrait for LibSqlLearningRepository {
    async fn create_feedback(&self, record: &FeedbackRecord) -> Result<()> {
        let conn = self.conn.lock().await;

        let features_json = serde_json::to_string(&record.features).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to serialize features: {e}"))
        })?;

        conn.execute(
            "INSERT INTO learning_feedback (
                id, query_pattern, features, vector_weight, fulltext_weight,
                effectiveness, timestamp, user_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                record.id.clone(),
                record.query_pattern.clone(),
                features_json,
                record.weights.vector_weight as f64,
                record.weights.fulltext_weight as f64,
                record.effectiveness as f64,
                record.timestamp.timestamp(),
                record.user_id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to insert feedback: {e}")))?;

        Ok(())
    }

    async fn get_all_feedback(&self) -> Result<Vec<FeedbackRecord>> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT id, query_pattern, features, vector_weight, fulltext_weight,
                        effectiveness, timestamp, user_id
                 FROM learning_feedback
                 ORDER BY timestamp DESC",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query feedback: {e}")))?;

        let mut records = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(e.to_string()))?
        {
            records.push(Self::row_to_feedback(&row)?);
        }

        Ok(records)
    }

    async fn get_feedback_by_pattern(&self, pattern: &str) -> Result<Vec<FeedbackRecord>> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT id, query_pattern, features, vector_weight, fulltext_weight,
                        effectiveness, timestamp, user_id
                 FROM learning_feedback
                 WHERE query_pattern = ?
                 ORDER BY timestamp DESC",
                libsql::params![pattern],
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to query feedback by pattern: {e}"))
            })?;

        let mut records = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(e.to_string()))?
        {
            records.push(Self::row_to_feedback(&row)?);
        }

        Ok(records)
    }

    async fn get_recent_feedback(&self, limit: usize) -> Result<Vec<FeedbackRecord>> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT id, query_pattern, features, vector_weight, fulltext_weight,
                        effectiveness, timestamp, user_id
                 FROM learning_feedback
                 ORDER BY timestamp DESC
                 LIMIT ?",
                libsql::params![limit as i64],
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to query recent feedback: {e}"))
            })?;

        let mut records = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(e.to_string()))?
        {
            records.push(Self::row_to_feedback(&row)?);
        }

        Ok(records)
    }

    async fn delete_old_feedback(&self, before_timestamp: DateTime<Utc>) -> Result<usize> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "DELETE FROM learning_feedback WHERE timestamp < ?",
                libsql::params![before_timestamp.timestamp()],
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to delete old feedback: {e}"))
            })?;

        Ok(result as usize)
    }

    async fn clear_all_feedback(&self) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute("DELETE FROM learning_feedback", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to clear feedback: {e}")))?;

        Ok(())
    }

    async fn get_feedback_count_by_pattern(&self, pattern: &str) -> Result<usize> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT COUNT(*) as count FROM learning_feedback WHERE query_pattern = ?",
                libsql::params![pattern],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to count feedback: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(e.to_string()))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get count: {e}")))?;
            Ok(count as usize)
        } else {
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Arc<Mutex<Connection>> {
        let db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .unwrap();
        let conn = db.connect().unwrap();

        // Create learning_feedback table
        conn.execute(
            "CREATE TABLE learning_feedback (
                id TEXT PRIMARY KEY,
                query_pattern TEXT NOT NULL,
                features TEXT NOT NULL,
                vector_weight REAL NOT NULL,
                fulltext_weight REAL NOT NULL,
                effectiveness REAL NOT NULL,
                timestamp INTEGER NOT NULL,
                user_id TEXT
            )",
            (),
        )
        .await
        .unwrap();

        // Create index
        conn.execute(
            "CREATE INDEX idx_learning_feedback_pattern ON learning_feedback(query_pattern)",
            (),
        )
        .await
        .unwrap();

        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn test_create_and_get_feedback() -> anyhow::Result<()> {
        let conn = setup_test_db().await;
        let repo = LibSqlLearningRepository::new(conn);

        let record = FeedbackRecord {
            id: "test-1".to_string(),
            query_pattern: "exact_match".to_string(),
            features: QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 20,
                is_question: false,
            },
            weights: SearchWeights {
                vector_weight: 0.3,
                fulltext_weight: 0.7,
                confidence: 0.8,
            },
            effectiveness: 0.9,
            timestamp: Utc::now(),
            user_id: Some("user1".to_string()),
        };

        // Create
        repo.create_feedback(&record).await?;

        // Get all
        let all_feedback = repo.get_all_feedback().await?;
        assert_eq!(all_feedback.len(), 1);
        assert_eq!(all_feedback[0].id, "test-1");

        // Get by pattern
        let pattern_feedback = repo.get_feedback_by_pattern("exact_match").await?;
        assert_eq!(pattern_feedback.len(), 1);

        // Get count
        let count = repo
            .get_feedback_count_by_pattern("exact_match")
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_delete_old_feedback() -> anyhow::Result<()> {
        let conn = setup_test_db().await;
        let repo = LibSqlLearningRepository::new(conn);

        // Create old record
        let old_record = FeedbackRecord {
            id: "old-1".to_string(),
            query_pattern: "exact_match".to_string(),
            features: QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 20,
                is_question: false,
            },
            weights: SearchWeights {
                vector_weight: 0.5,
                fulltext_weight: 0.5,
                confidence: 0.8,
            },
            effectiveness: 0.8,
            timestamp: Utc::now() - chrono::Duration::days(30),
            user_id: None,
        };

        // Create new record
        let new_record = FeedbackRecord {
            id: "new-1".to_string(),
            query_pattern: "exact_match".to_string(),
            features: QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 20,
                is_question: false,
            },
            weights: SearchWeights {
                vector_weight: 0.5,
                fulltext_weight: 0.5,
                confidence: 0.8,
            },
            effectiveness: 0.9,
            timestamp: Utc::now(),
            user_id: None,
        };

        repo.create_feedback(&old_record).await?;
        repo.create_feedback(&new_record).await?;

        // Delete old feedback (older than 7 days)
        let deleted_count = repo
            .delete_old_feedback(Utc::now() - chrono::Duration::days(7))
            .await
            .unwrap();

        assert_eq!(deleted_count, 1);

        // Verify only new record remains
        let remaining = repo.get_all_feedback().await?;
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "new-1");
    }
}
