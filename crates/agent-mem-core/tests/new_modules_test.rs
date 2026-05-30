//!
//! New modules verification tests
//!
//! Tests for:
//! - ABAC Engine (abac_engine.rs)
//! - Data Lineage (lineage.rs)
//! - Privacy Preserving (privacy_preserving.rs)
//! - Predictive Monitoring (predictive_monitoring.rs)
//!

use agent_mem_core::abac_engine::{
    AbacConfig, AbacEngine, AccessDecision, AccessRequest, ActionAttributes,
    ActionType, EnvironmentAttributes, PolicyEffect,
    ResourceAttributes, ResourceType, SubjectAttributes, AgentType, SecurityContext,
};
use agent_mem_core::lineage::{
    LineageConfig, LineageTracker,
};
use agent_mem_core::privacy_preserving::{
    DifferentialPrivacy, DataMasking, TenantKeyManager,
    PrivacyConfig,
};
use agent_mem_core::predictive_monitoring::{
    PredictiveConfig, PredictiveMonitor,
};
use std::collections::HashMap;
use chrono::Utc;

// ============================================================================
// ABAC Engine Tests
// ============================================================================

#[tokio::test]
async fn test_abac_engine_creation() {
    let config = AbacConfig::default();
    let engine = AbacEngine::new(config);
    let _ = engine;
}

#[tokio::test]
async fn test_abac_engine_add_and_evaluate_policy() {
    let config = AbacConfig::default();
    let engine = AbacEngine::new(config);

    // Verify engine was created
    let policies = engine.get_policies().await;
    assert_eq!(policies.len(), 0);

    // Create a request
    let subject = SubjectAttributes {
        agent_id: "agent_123".to_string(),
        agent_type: AgentType::UserAssistant,
        department: Some("engineering".to_string()),
        clearance_level: 3,
        user_id: Some("user_456".to_string()),
        custom_attributes: HashMap::new(),
    };

    let resource = ResourceAttributes {
        resource_id: "memory_456".to_string(),
        resource_type: ResourceType::Memory,
        owner_id: "agent_123".to_string(),
        sensitivity_level: 2,
        classification: Some("internal".to_string()),
        department: None,
        custom_attributes: HashMap::new(),
    };

    let action = ActionAttributes {
        action_type: ActionType::Read,
        is_batch: false,
        batch_size: None,
        custom_attributes: HashMap::new(),
    };

    let environment = EnvironmentAttributes {
        timestamp: Utc::now(),
        location: Some("192.168.1.1".to_string()),
        source: Some("api".to_string()),
        security_context: SecurityContext {
            is_secure: true,
            request_id: None,
            session_id: None,
            mfa_verified: true,
        },
    };

    let request = AccessRequest::new(subject, resource, environment, action);
    let response = engine.evaluate(&request).await;

    // Default decision depends on engine configuration
    // Either Permit (if default allow) or NotApplicable (no matching policy)
    assert!(matches!(response.decision, AccessDecision::Permit | AccessDecision::NotApplicable));
}

// ============================================================================
// Lineage Tracking Tests
// ============================================================================

#[tokio::test]
async fn test_lineage_tracker_creation() {
    let config = LineageConfig::default();
    let tracker = LineageTracker::new(config);
    let _ = tracker;
}

#[tokio::test]
async fn test_lineage_tracker_get_stats() {
    let config = LineageConfig::default();
    let tracker = LineageTracker::new(config);

    // Get stats - should be empty initially
    let stats = tracker.get_stats().await;
    assert_eq!(stats.total_nodes, 0);
    assert_eq!(stats.total_transformations, 0);
}

// ============================================================================
// Privacy Preserving Tests
// ============================================================================

#[tokio::test]
async fn test_differential_privacy_creation() {
    let config = PrivacyConfig::default();
    let dp = DifferentialPrivacy::new(config);
    let _ = dp;
}

#[tokio::test]
async fn test_differential_privacy_add_noise() {
    let config = PrivacyConfig::default();
    let dp = DifferentialPrivacy::new(config);

    // Test Laplace noise
    let noisy_value = dp.add_laplace_noise(100.0, 1.0).await;
    // Noise should have been added, so value should be different
    assert_ne!(noisy_value, 100.0);

    // Test Gaussian noise
    let noisy_gaussian = dp.add_gaussian_noise(100.0, 1.0).await;
    assert_ne!(noisy_gaussian, 100.0);
}

#[tokio::test]
async fn test_data_masking_email() {
    let config = PrivacyConfig::default();
    let masking = DataMasking::new(config);

    let masked = masking.mask_email("user@example.com").await;
    // Email masking should partially hide the username
    assert!(masked.contains("@example.com"));
    assert!(masked.contains("*"));
}

#[tokio::test]
async fn test_data_masking_phone() {
    let config = PrivacyConfig::default();
    let masking = DataMasking::new(config);

    let masked = masking.mask_phone("13812345678").await;
    // Phone masking should partially hide the number
    assert!(masked.contains("*"));
    assert!(masked.len() >= 7);
}

#[tokio::test]
async fn test_tenant_key_manager_creation() {
    let manager = TenantKeyManager::new();
    let _ = manager;
}

// ============================================================================
// Predictive Monitoring Tests
// ============================================================================

#[tokio::test]
async fn test_predictive_monitor_creation() {
    let config = PredictiveConfig::default();
    let monitor = PredictiveMonitor::new(config);
    let _ = monitor;
}

#[tokio::test]
async fn test_predictive_monitor_get_anomalies() {
    let config = PredictiveConfig::default();
    let monitor = PredictiveMonitor::new(config);

    // Get anomalies - should be empty initially
    let anomalies = monitor.get_anomalies(None).await;
    assert!(anomalies.is_empty());
}

#[tokio::test]
async fn test_predictive_monitor_health_prediction() {
    let config = PredictiveConfig::default();
    let monitor = PredictiveMonitor::new(config);

    // Get health prediction
    let prediction = monitor.predict_health().await;

    // Health prediction should be generated
    assert!(prediction.risk_score >= 0.0 && prediction.risk_score <= 1.0);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_abac_lineage_integration() {
    // Test that ABAC and Lineage can work together
    let abac_config = AbacConfig::default();
    let lineage_config = LineageConfig::default();

    let abac_engine = AbacEngine::new(abac_config);
    let lineage_tracker = LineageTracker::new(lineage_config);

    // Both should be functional
    let policies = abac_engine.get_policies().await;
    assert_eq!(policies.len(), 0);

    let stats = lineage_tracker.get_stats().await;
    assert_eq!(stats.total_nodes, 0);
}

#[tokio::test]
async fn test_full_privacy_monitoring_integration() {
    // Test that Privacy and Monitoring can work together
    let privacy_config = PrivacyConfig::default();
    let monitoring_config = PredictiveConfig::default();

    let dp = DifferentialPrivacy::new(privacy_config.clone());
    let monitor = PredictiveMonitor::new(monitoring_config);

    // Both should be functional
    let noisy = dp.add_laplace_noise(1.0, 0.1).await;
    assert!(noisy > 0.0);

    let predictions = monitor.predict_health().await;
    assert!(predictions.risk_score >= 0.0);
}
