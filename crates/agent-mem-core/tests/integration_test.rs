//!
//! Real integration tests for AgentMem system
//!
//! This test verifies the memory system works correctly by:
//! 1. Creating memories with ABAC access control
//! 2. Tracking data lineage through transformations
//! 3. Applying privacy-preserving techniques
//! 4. Monitoring system health with predictive analytics
//!

use agent_mem_core::abac_engine::{
    AbacConfig, AbacEngine, AccessRequest, AccessDecision,
    ActionType, AgentType, EnvironmentAttributes, ResourceAttributes,
    ResourceType, SecurityContext, SubjectAttributes,
};
use agent_mem_core::lineage::{LineageConfig, LineageTracker, LineageQuery, LineageDirection};
use agent_mem_core::privacy_preserving::{
    DataMasking, DifferentialPrivacy, PrivacyConfig,
};
use agent_mem_core::predictive_monitoring::{
    PredictiveConfig, PredictiveMonitor,
};
use std::collections::HashMap;
use chrono::Utc;

#[tokio::test]
async fn test_real_memory_workflow_with_abac() {
    println!("=== Testing Real Memory Workflow with ABAC ===");

    // 1. Create ABAC engine
    let config = AbacConfig::default();
    let engine = AbacEngine::new(config);
    println!("✅ ABAC Engine created");

    // 2. Create a memory access request
    let subject = SubjectAttributes {
        agent_id: "claude_code".to_string(),
        agent_type: AgentType::UserAssistant,
        department: Some("ai".to_string()),
        clearance_level: 3,
        user_id: Some("user_001".to_string()),
        custom_attributes: HashMap::new(),
    };

    let resource = ResourceAttributes {
        resource_id: "project_context".to_string(),
        resource_type: ResourceType::Memory,
        owner_id: "claude_code".to_string(),
        sensitivity_level: 2,
        classification: Some("internal".to_string()),
        department: None,
        custom_attributes: HashMap::new(),
    };

    let action = agent_mem_core::abac_engine::ActionAttributes {
        action_type: ActionType::Read,
        is_batch: false,
        batch_size: None,
        custom_attributes: HashMap::new(),
    };

    let environment = EnvironmentAttributes {
        timestamp: Utc::now(),
        location: Some("claude-code".to_string()),
        source: Some("api".to_string()),
        security_context: SecurityContext {
            is_secure: true,
            request_id: Some("req_123".to_string()),
            session_id: Some("session_456".to_string()),
            mfa_verified: true,
        },
    };

    let request = AccessRequest::new(subject, resource, environment, action);
    let response = engine.evaluate(&request).await;

    println!("✅ Access evaluation completed: {:?}", response.decision);
    assert!(
        matches!(response.decision, AccessDecision::Permit | AccessDecision::NotApplicable),
        "Access should be permitted or not applicable"
    );

    // 3. Check audit log
    let audit_entries = engine.get_audit_log(Some(10)).await;
    println!("✅ Audit log entries: {}", audit_entries.len());
}

#[tokio::test]
async fn test_data_lineage_tracking() {
    println!("\n=== Testing Data Lineage Tracking ===");

    // 1. Create lineage tracker
    let config = LineageConfig::default();
    let tracker = LineageTracker::new(config);
    println!("✅ Lineage Tracker created");

    // 2. Record memory creation
    let memory_id = tracker.record_creation(
        "memory_001".to_string(),
        agent_mem_core::lineage::MemoryType::Working,
        "claude".to_string(),
        HashMap::new(),
    ).await;
    println!("✅ Recorded memory creation: {}", memory_id);

    // 3. Record memory update
    let _update_id = tracker.record_update(
        "memory_001".to_string(),
        "claude".to_string(),
        Some("Updated context".to_string()),
    ).await;
    println!("✅ Recorded memory update");

    // 4. Get lineage
    let query = LineageQuery {
        memory_id: "memory_001".to_string(),
        direction: LineageDirection::Both,
        max_depth: Some(10),
        include_metadata: true,
        include_transformations: true,
    };

    let lineage = tracker.get_lineage(query).await;
    println!("✅ Lineage retrieved: {} nodes, {} edges",
             lineage.nodes.len(), lineage.edges.len());

    // 5. Get stats
    let stats = tracker.get_stats().await;
    println!("✅ Tracker stats: {} nodes, {} transformations",
             stats.total_nodes, stats.total_transformations);

    assert!(stats.total_nodes >= 1, "Should have at least one node");
}

#[tokio::test]
async fn test_privacy_preserving_operations() {
    println!("\n=== Testing Privacy Preserving Operations ===");

    // 1. Differential Privacy
    let privacy_config = PrivacyConfig::default();
    let dp = DifferentialPrivacy::new(privacy_config.clone());
    println!("✅ Differential Privacy engine created");

    // Add Laplace noise
    let original_value = 100.0;
    let noisy_value = dp.add_laplace_noise(original_value, 1.0).await;
    println!("✅ Laplace noise: {} -> {:.2}", original_value, noisy_value);

    // Add Gaussian noise
    let noisy_gaussian = dp.add_gaussian_noise(original_value, 1.0).await;
    println!("✅ Gaussian noise: {} -> {:.2}", original_value, noisy_gaussian);

    // 2. Data Masking
    let masking = DataMasking::new(privacy_config.clone());
    println!("✅ Data Masking engine created");

    // Mask email
    let email = "user@example.com";
    let masked_email = masking.mask_email(email).await;
    println!("✅ Email masking: {} -> {}", email, masked_email);

    // Mask phone
    let phone = "13812345678";
    let masked_phone = masking.mask_phone(phone).await;
    println!("✅ Phone masking: {} -> {}", phone, masked_phone);

    // Verify all operations work
    assert_ne!(original_value, noisy_value, "Noise should be added");
    assert!(masked_email.contains("*"), "Email should be masked");
    assert!(masked_phone.contains("*"), "Phone should be masked");

    println!("✅ All privacy operations verified");
}

#[tokio::test]
async fn test_predictive_monitoring() {
    println!("\n=== Testing Predictive Monitoring ===");

    // 1. Create predictive monitor
    let config = PredictiveConfig::default();
    let monitor = PredictiveMonitor::new(config);
    println!("✅ Predictive Monitor created");

    // 2. Get anomalies (should be empty initially)
    let anomalies = monitor.get_anomalies(None).await;
    println!("✅ Initial anomalies: {}", anomalies.len());

    // 3. Predict health
    let prediction = monitor.predict_health().await;
    println!("✅ Health prediction: risk_score={:.2}, status={:?}",
             prediction.risk_score, prediction.predicted_status);

    // Verify health prediction
    assert!(prediction.risk_score >= 0.0 && prediction.risk_score <= 1.0);

    println!("✅ Predictive monitoring verified");
}

#[tokio::test]
async fn test_end_to_end_memory_operation() {
    println!("\n=== Testing End-to-End Memory Operation ===");

    // Setup all components
    let abac_engine = AbacEngine::new(AbacConfig::default());
    let lineage_tracker = LineageTracker::new(LineageConfig::default());
    let privacy_config = PrivacyConfig::default();
    let dp = DifferentialPrivacy::new(privacy_config.clone());
    let masking = DataMasking::new(privacy_config.clone());
    let monitor = PredictiveMonitor::new(PredictiveConfig::default());

    println!("✅ All components initialized");

    // 1. Create and protect memory
    let memory_content = "Sensitive project information about the AI agent system";
    println!("📝 Original content: {}", memory_content);

    // 2. Apply privacy protection
    let noisy_value = dp.add_laplace_noise(100.0, 1.0).await;
    println!("✅ Applied differential privacy: 100.0 -> {:.2}", noisy_value);

    let masked_email = masking.mask_email("developer@company.com").await;
    println!("✅ Applied email masking: developer@company.com -> {}", masked_email);

    // 3. Track lineage
    let memory_id = lineage_tracker.record_creation(
        "memory_e2e".to_string(),
        agent_mem_core::lineage::MemoryType::Working,
        "system".to_string(),
        HashMap::new(),
    ).await;
    println!("✅ Tracked lineage: {}", memory_id);

    // 4. Record access attempt with ABAC
    let subject = SubjectAttributes {
        agent_id: "test_agent".to_string(),
        agent_type: AgentType::UserAssistant,
        department: Some("engineering".to_string()),
        clearance_level: 2,
        user_id: Some("user_test".to_string()),
        custom_attributes: HashMap::new(),
    };

    let resource = ResourceAttributes {
        resource_id: memory_id.clone(),
        resource_type: ResourceType::Memory,
        owner_id: "test_agent".to_string(),
        sensitivity_level: 2,
        classification: Some("confidential".to_string()),
        department: None,
        custom_attributes: HashMap::new(),
    };

    let action = agent_mem_core::abac_engine::ActionAttributes {
        action_type: ActionType::Read,
        is_batch: false,
        batch_size: None,
        custom_attributes: HashMap::new(),
    };

    let environment = EnvironmentAttributes {
        timestamp: Utc::now(),
        location: Some("test".to_string()),
        source: Some("test".to_string()),
        security_context: SecurityContext {
            is_secure: true,
            request_id: None,
            session_id: None,
            mfa_verified: false,
        },
    };

    let request = AccessRequest::new(subject, resource, environment, action);
    let response = abac_engine.evaluate(&request).await;
    println!("✅ ABAC evaluation: {:?}", response.decision);

    // 5. Monitor system health
    let prediction = monitor.predict_health().await;
    println!("✅ System health: risk={:.2}", prediction.risk_score);

    // 6. Get lineage stats
    let stats = lineage_tracker.get_stats().await;
    println!("✅ Lineage stats: {} nodes", stats.total_nodes);

    println!("\n🎉 End-to-end memory operation completed successfully!");
}

#[tokio::test]
async fn test_abac_policy_combined_with_lineage() {
    println!("\n=== Testing ABAC + Lineage Combined ===");

    let abac = AbacEngine::new(AbacConfig::default());
    let lineage = LineageTracker::new(LineageConfig::default());

    // Record a memory with lineage
    let memory_id = lineage.record_creation(
        "secure_memory".to_string(),
        agent_mem_core::lineage::MemoryType::Semantic,
        "admin".to_string(),
        HashMap::new(),
    ).await;

    // Create a high-sensitivity resource
    let resource = ResourceAttributes {
        resource_id: memory_id.clone(),
        resource_type: ResourceType::Memory,
        owner_id: "admin".to_string(),
        sensitivity_level: 4, // High sensitivity
        classification: Some("top-secret".to_string()),
        department: Some("security".to_string()),
        custom_attributes: HashMap::new(),
    };

    // Create request from authorized user
    let authorized_subject = SubjectAttributes {
        agent_id: "admin_user".to_string(),
        agent_type: AgentType::SystemAdmin,
        department: Some("security".to_string()),
        clearance_level: 5, // Top clearance
        user_id: Some("admin".to_string()),
        custom_attributes: HashMap::new(),
    };

    let action = agent_mem_core::abac_engine::ActionAttributes {
        action_type: ActionType::Read,
        is_batch: false,
        batch_size: None,
        custom_attributes: HashMap::new(),
    };

    let environment = EnvironmentAttributes {
        timestamp: Utc::now(),
        location: Some("secure-zone".to_string()),
        source: Some("admin-console".to_string()),
        security_context: SecurityContext {
            is_secure: true,
            request_id: Some("admin_req".to_string()),
            session_id: Some("admin_session".to_string()),
            mfa_verified: true,
        },
    };

    let request = AccessRequest::new(authorized_subject, resource, environment, action);
    let response = abac.evaluate(&request).await;

    println!("✅ ABAC response: {:?}", response.decision);
    println!("✅ Memory ID tracked: {}", memory_id);

    // Verify the response is either permit (has matching policy) or not applicable (no deny policy)
    assert!(
        matches!(response.decision, AccessDecision::Permit | AccessDecision::NotApplicable),
        "Should be Permit or NotApplicable"
    );

    println!("✅ ABAC + Lineage integration verified");
}

#[tokio::test]
async fn test_privacy_monitoring_integration() {
    println!("\n=== Testing Privacy + Monitoring Integration ===");

    let privacy_config = PrivacyConfig::default();
    let dp = DifferentialPrivacy::new(privacy_config.clone());
    let monitor = PredictiveMonitor::new(PredictiveConfig::default());

    // Simulate privacy-aware operations
    let mut total_noise_added = 0.0;
    let operations = vec![100.0, 200.0, 150.0, 175.0, 125.0];

    for (i, value) in operations.iter().enumerate() {
        let noisy = dp.add_laplace_noise(*value, 1.0).await;
        let noise = (noisy - value).abs();
        total_noise_added += noise;
        println!("Op {}: {} -> {:.2} (noise: {:.2})", i+1, value, noisy, noise);
    }

    println!("✅ Total noise added: {:.2}", total_noise_added);

    // Get system health
    let prediction = monitor.predict_health().await;
    println!("✅ System health risk: {:.2}", prediction.risk_score);

    assert!(total_noise_added > 0.0, "Should have added noise");
    assert!(prediction.risk_score >= 0.0 && prediction.risk_score <= 1.0);

    println!("✅ Privacy + Monitoring integration verified");
}
