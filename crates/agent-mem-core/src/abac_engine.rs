//! ABAC (Attribute-Based Access Control) Engine for AgentMem
//!
//! This module provides fine-grained, dynamic access control based on subject attributes,
//! resource attributes, environment attributes, and action attributes.
//!
//! # Features
//!
//! - Subject-based policies (agent roles, clearance levels, departments)
//! - Resource-based policies (memory types, sensitivity levels, ownership)
//! - Environment-aware policies (time-based, location-based)
//! - Dynamic policy evaluation with caching
//! - Delegation and inheritance support

use crate::{collaboration::AgentPermissionLevel, CoreError, CoreResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// ABAC Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacConfig {
    /// Enable policy caching
    pub enable_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum policy depth for delegation chains
    pub max_delegation_depth: u32,
    /// Enable audit logging
    pub enable_audit: bool,
}

impl Default for AbacConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_ttl_seconds: 300,
            max_delegation_depth: 5,
            enable_audit: true,
        }
    }
}

/// Subject attributes (who is making the request)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubjectAttributes {
    /// Agent ID
    pub agent_id: String,
    /// Agent type/role
    pub agent_type: AgentType,
    /// Department or team
    pub department: Option<String>,
    /// Security clearance level (0-5)
    pub clearance_level: u8,
    /// Associated user ID
    pub user_id: Option<String>,
    /// Custom attributes
    pub custom_attributes: HashMap<String, String>,
}

/// Agent type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    /// System administrator
    SystemAdmin,
    /// Data scientist agent
    DataScientist,
    /// User assistant agent
    UserAssistant,
    /// Research agent
    ResearchAgent,
    /// General purpose agent
    General,
}

impl Default for AgentType {
    fn default() -> Self {
        AgentType::General
    }
}

/// Resource attributes (what is being accessed)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceAttributes {
    /// Memory/resource ID
    pub resource_id: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Owner agent ID
    pub owner_id: String,
    /// Sensitivity level (0-5)
    pub sensitivity_level: u8,
    /// Classification (e.g., "confidential", "internal", "public")
    pub classification: Option<String>,
    /// Department ownership
    pub department: Option<String>,
    /// Custom attributes
    pub custom_attributes: HashMap<String, String>,
}

/// Resource type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Memory item
    Memory,
    /// Agent configuration
    AgentConfig,
    /// System configuration
    SystemConfig,
    /// User data
    UserData,
    /// Audit log
    AuditLog,
    /// Collaboration resource
    Collaboration,
}

impl Default for ResourceType {
    fn default() -> Self {
        ResourceType::Memory
    }
}

/// Environment attributes (context of the request)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentAttributes {
    /// Current timestamp
    pub timestamp: DateTime<Utc>,
    /// IP address or location identifier
    pub location: Option<String>,
    /// Request source (e.g., "api", "cli", "internal")
    pub source: Option<String>,
    /// Security context
    pub security_context: SecurityContext,
}

/// Security context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityContext {
    /// Is the request over secure channel
    pub is_secure: bool,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// MFA status
    pub mfa_verified: bool,
}

/// Action attributes (what operation is being performed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActionType {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Delete operation
    Delete,
    /// Share operation
    Share,
    /// Execute operation
    Execute,
    /// Admin operation
    Admin,
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::Read
    }
}

/// Action attributes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionAttributes {
    /// Action type
    pub action_type: ActionType,
    /// Is this a batch operation
    pub is_batch: bool,
    /// Number of resources affected (for batch operations)
    pub batch_size: Option<usize>,
    /// Custom attributes
    pub custom_attributes: HashMap<String, String>,
}

/// Access request containing all attributes for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    /// Subject making the request
    pub subject: SubjectAttributes,
    /// Resource being accessed
    pub resource: ResourceAttributes,
    /// Environment context
    pub environment: EnvironmentAttributes,
    /// Action being performed
    pub action: ActionAttributes,
}

impl AccessRequest {
    /// Create a cache key based on request content
    fn cache_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.subject.agent_id.hash(&mut hasher);
        self.resource.resource_id.hash(&mut hasher);
        self.action.action_type.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl AccessRequest {
    /// Create a new access request
    pub fn new(
        subject: SubjectAttributes,
        resource: ResourceAttributes,
        environment: EnvironmentAttributes,
        action: ActionAttributes,
    ) -> Self {
        Self {
            subject,
            resource,
            environment,
            action,
        }
    }
}

/// Access decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessDecision {
    /// Access is permitted
    Permit,
    /// Access is denied
    Deny,
    /// Access decision is indeterminate
    Indeterminate,
    /// Access is not applicable
    NotApplicable,
}

/// Access decision with reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessResponse {
    /// The access decision
    pub decision: AccessDecision,
    /// Human-readable reason for the decision
    pub reason: String,
    /// Policies that matched
    pub matched_policies: Vec<String>,
    /// Obligations that must be fulfilled
    pub obligations: Vec<Obligation>,
    /// Decision timestamp
    pub timestamp: DateTime<Utc>,
}

/// Obligation that must be fulfilled alongside the access decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    /// Obligation type
    pub obligation_type: ObligationType,
    /// Obligation parameters
    pub parameters: HashMap<String, String>,
}

/// Obligation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObligationType {
    /// Log the access
    LogAccess,
    /// Notify security team
    NotifySecurity,
    /// Require additional verification
    RequireMfa,
    /// Mask sensitive data
    MaskData,
    /// Rate limit the request
    RateLimit,
}

/// ABAC Policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacPolicy {
    /// Policy ID
    pub policy_id: String,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: Option<String>,
    /// Target resources/actions (None means all)
    pub target: Option<PolicyTarget>,
    /// Policy conditions
    pub conditions: Vec<PolicyCondition>,
    /// Effect (permit or deny)
    pub effect: PolicyEffect,
    /// Priority (higher = evaluated first)
    pub priority: u32,
    /// Is policy active
    pub is_active: bool,
}

/// Policy target
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyTarget {
    /// Subject attributes that must match
    pub subject_attrs: Option<HashMap<String, AttributeMatcher>>,
    /// Resource types that must match
    pub resource_types: Option<HashSet<ResourceType>>,
    /// Action types that must match
    pub action_types: Option<HashSet<ActionType>>,
}

/// Attribute matcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeMatcher {
    /// Exact match
    Equals(String),
    /// Contains substring
    Contains(String),
    /// Greater than
    GreaterThan(i32),
    /// Less than
    LessThan(i32),
    /// In a set of values
    In(Vec<String>),
    /// Regex match
    Matches(String),
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    /// Attribute category (subject, resource, environment, action)
    pub category: AttributeCategory,
    /// Attribute name
    pub attribute_name: String,
    /// Comparison operation
    pub operation: ConditionOperation,
    /// Expected value
    pub value: ConditionValue,
}

/// Attribute category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AttributeCategory {
    /// Subject attribute
    Subject,
    /// Resource attribute
    Resource,
    /// Environment attribute
    Environment,
    /// Action attribute
    Action,
}

/// Condition operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperation {
    /// Equality check
    Equals,
    /// Not equal
    NotEquals,
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// Greater or equal
    GreaterOrEqual,
    /// Less or equal
    LessOrEqual,
    /// Contains
    Contains,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Regex match
    Matches,
    /// Set membership
    In,
}

/// Condition value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i32),
    /// Boolean value
    Boolean(bool),
    /// String list
    StringList(Vec<String>),
}

/// Policy effect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyEffect {
    /// Policy permits access
    Permit,
    /// Policy denies access
    Deny,
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    /// Policy that matched
    pub policy: AbacPolicy,
    /// Whether all conditions were satisfied
    pub conditions_satisfied: bool,
    /// Reason for evaluation
    pub reason: String,
}

/// Delegation chain entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationEntry {
    /// Delegator (who grants permission)
    pub delegator: String,
    /// Delegatee (who receives permission)
    pub delegatee: String,
    /// Permissions being delegated
    pub permissions: Vec<DelegatedPermission>,
    /// Delegation timestamp
    pub delegated_at: DateTime<Utc>,
    /// Delegation expiry
    pub expires_at: Option<DateTime<Utc>>,
    /// Is delegation active
    pub is_active: bool,
}

/// Delegated permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedPermission {
    /// Resource pattern (supports wildcards)
    pub resource_pattern: String,
    /// Actions allowed
    pub actions: Vec<ActionType>,
}

/// ABAC Policy cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Cached response
    response: AccessResponse,
    /// Cache expiry time
    expires_at: DateTime<Utc>,
}

/// ABAC Engine for policy evaluation
pub struct AbacEngine {
    /// Configuration
    config: AbacConfig,
    /// Policy storage
    policies: Arc<RwLock<HashMap<String, AbacPolicy>>>,
    /// Delegation chains
    delegations: Arc<RwLock<HashMap<String, DelegationEntry>>>,
    /// Decision cache
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Audit log
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
}

/// Audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Request that was evaluated
    pub request: AccessRequest,
    /// Response decision
    pub response: AccessResponse,
    /// Evaluation time in milliseconds
    pub evaluation_time_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl AbacEngine {
    /// Create a new ABAC engine
    pub fn new(config: AbacConfig) -> Self {
        Self {
            config,
            policies: Arc::new(RwLock::new(HashMap::new())),
            delegations: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default configuration
    pub fn default_engine() -> Self {
        Self::new(AbacConfig::default())
    }

    /// Add a policy to the engine
    pub async fn add_policy(&self, policy: AbacPolicy) -> CoreResult<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.policy_id.clone(), policy);
        // Clear cache when policies change
        self.clear_cache().await;
        Ok(())
    }

    /// Remove a policy
    pub async fn remove_policy(&self, policy_id: &str) -> CoreResult<bool> {
        let mut policies = self.policies.write().await;
        let removed = policies.remove(policy_id).is_some();
        if removed {
            self.clear_cache().await;
        }
        Ok(removed)
    }

    /// Get all policies
    pub async fn get_policies(&self) -> Vec<AbacPolicy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    /// Add a delegation
    pub async fn add_delegation(&self, delegation: DelegationEntry) -> CoreResult<()> {
        let mut delegations = self.delegations.write().await;
        let key = format!("{}->{}", delegation.delegator, delegation.delegatee);
        delegations.insert(key, delegation);
        Ok(())
    }

    /// Evaluate an access request
    pub async fn evaluate(&self, request: &AccessRequest) -> AccessResponse {
        let start_time = std::time::Instant::now();

        // Check cache first
        if self.config.enable_cache {
            if let Some(cached) = self.get_cached_response(request).await {
                let evaluation_time_ms = start_time.elapsed().as_millis() as u64;
                self.record_audit(request, cached.clone(), evaluation_time_ms).await;
                return cached;
            }
        }

        // Get all active policies sorted by priority
        let policies = {
            let policies = self.policies.read().await;
            let mut active: Vec<_> = policies
                .values()
                .filter(|p| p.is_active)
                .cloned()
                .collect();
            active.sort_by(|a, b| b.priority.cmp(&a.priority));
            active
        };

        // Evaluate policies
        let mut matched_policies = Vec::new();
        let mut applicable_policies: Vec<PolicyEvaluationResult> = Vec::new();

        for policy in &policies {
            if let Some(result) = self.evaluate_policy(policy, request).await {
                if result.conditions_satisfied {
                    applicable_policies.push(result);
                }
            }
        }

        // Check delegation chains
        let delegation_result = self.check_delegation(request).await;
        if delegation_result.1 {
            matched_policies.push("delegation_chain".to_string());
        }

        // Apply policy combining algorithm (deny-overrides)
        let decision = self.combine_decisions(&applicable_policies, delegation_result.1);

        for result in &applicable_policies {
            matched_policies.push(result.policy.policy_id.clone());
        }

        let reason = self.generate_decision_reason(&decision, &applicable_policies, delegation_result.1);
        let obligations = self.generate_obligations(&decision);

        let response = AccessResponse {
            decision,
            reason,
            matched_policies,
            obligations,
            timestamp: Utc::now(),
        };

        // Cache the response
        if self.config.enable_cache {
            self.cache_response(request, &response).await;
        }

        // Record audit
        let evaluation_time_ms = start_time.elapsed().as_millis() as u64;
        if self.config.enable_audit {
            self.record_audit(request, response.clone(), evaluation_time_ms).await;
        }

        response
    }

    /// Evaluate a single policy against a request
    async fn evaluate_policy(
        &self,
        policy: &AbacPolicy,
        request: &AccessRequest,
    ) -> Option<PolicyEvaluationResult> {
        // Check target match
        if let Some(ref target) = policy.target {
            if !self.matches_target(target, request) {
                return None;
            }
        }

        // Evaluate conditions
        let all_conditions_met = self.evaluate_conditions(&policy.conditions, request).await;

        Some(PolicyEvaluationResult {
            policy: policy.clone(),
            conditions_satisfied: all_conditions_met,
            reason: if all_conditions_met {
                "All conditions satisfied".to_string()
            } else {
                "Some conditions not satisfied".to_string()
            },
        })
    }

    /// Check if request matches policy target
    fn matches_target(&self, target: &PolicyTarget, request: &AccessRequest) -> bool {
        // Check subject attributes
        if let Some(ref subject_matchers) = target.subject_attrs {
            for (attr, matcher) in subject_matchers {
                if let Some(value) = self.get_subject_attribute(&request.subject, attr) {
                    if !self.matches(&value, matcher) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        // Check resource types
        if let Some(ref resource_types) = target.resource_types {
            if !resource_types.is_empty() && !resource_types.contains(&request.resource.resource_type) {
                return false;
            }
        }

        // Check action types
        if let Some(ref action_types) = target.action_types {
            if !action_types.is_empty() && !action_types.contains(&request.action.action_type) {
                return false;
            }
        }

        true
    }

    /// Get subject attribute value
    fn get_subject_attribute(&self, subject: &SubjectAttributes, attr: &str) -> Option<String> {
        match attr {
            "agent_id" => Some(subject.agent_id.clone()),
            "agent_type" => Some(format!("{:?}", subject.agent_type)),
            "department" => subject.department.clone(),
            "user_id" => subject.user_id.clone(),
            _ => subject.custom_attributes.get(attr).cloned(),
        }
    }

    /// Get resource attribute value
    fn get_resource_attribute(&self, resource: &ResourceAttributes, attr: &str) -> Option<String> {
        match attr {
            "resource_id" => Some(resource.resource_id.clone()),
            "resource_type" => Some(format!("{:?}", resource.resource_type)),
            "owner_id" => Some(resource.owner_id.clone()),
            "sensitivity_level" => Some(resource.sensitivity_level.to_string()),
            "classification" => resource.classification.clone(),
            "department" => resource.department.clone(),
            _ => resource.custom_attributes.get(attr).cloned(),
        }
    }

    /// Get environment attribute value
    fn get_environment_attribute(&self, env: &EnvironmentAttributes, attr: &str) -> Option<String> {
        match attr {
            "timestamp" => Some(env.timestamp.to_rfc3339()),
            "location" => env.location.clone(),
            "source" => env.source.clone(),
            "is_secure" => Some(env.security_context.is_secure.to_string()),
            "mfa_verified" => Some(env.security_context.mfa_verified.to_string()),
            _ => None,
        }
    }

    /// Get action attribute value
    fn get_action_attribute(&self, action: &ActionAttributes, attr: &str) -> Option<String> {
        match attr {
            "action_type" => Some(format!("{:?}", action.action_type)),
            "is_batch" => Some(action.is_batch.to_string()),
            "batch_size" => action.batch_size.map(|s| s.to_string()),
            _ => action.custom_attributes.get(attr).cloned(),
        }
    }

    /// Match a value against an attribute matcher
    fn matches(&self, value: &str, matcher: &AttributeMatcher) -> bool {
        match matcher {
            AttributeMatcher::Equals(expected) => value == expected,
            AttributeMatcher::Contains(substr) => value.contains(substr),
            AttributeMatcher::GreaterThan(threshold) => {
                value.parse::<i32>().map(|v| v > *threshold).unwrap_or(false)
            }
            AttributeMatcher::LessThan(threshold) => {
                value.parse::<i32>().map(|v| v < *threshold).unwrap_or(false)
            }
            AttributeMatcher::In(values) => values.contains(&value.to_string()),
            AttributeMatcher::Matches(pattern) => {
                regex::Regex::new(pattern)
                    .map(|r| r.is_match(value))
                    .unwrap_or(false)
            }
        }
    }

    /// Evaluate policy conditions
    async fn evaluate_conditions(&self, conditions: &[PolicyCondition], request: &AccessRequest) -> bool {
        for condition in conditions {
            let value_opt = match condition.category {
                AttributeCategory::Subject => self.get_subject_attribute(&request.subject, &condition.attribute_name),
                AttributeCategory::Resource => self.get_resource_attribute(&request.resource, &condition.attribute_name),
                AttributeCategory::Environment => self.get_environment_attribute(&request.environment, &condition.attribute_name),
                AttributeCategory::Action => self.get_action_attribute(&request.action, &condition.attribute_name),
            };

            let value = match value_opt {
                Some(v) => v,
                None => return false,
            };

            if !self.evaluate_condition(&value, &condition.operation, &condition.value) {
                return false;
            }
        }
        true
    }

    /// Evaluate a single condition
    fn evaluate_condition(&self, value: &str, operation: &ConditionOperation, expected: &ConditionValue) -> bool {
        match operation {
            ConditionOperation::Equals => {
                match expected {
                    ConditionValue::String(s) => value == s,
                    ConditionValue::Integer(i) => value == &i.to_string(),
                    ConditionValue::Boolean(b) => value == &b.to_string(),
                    ConditionValue::StringList(list) => list.contains(&value.to_string()),
                }
            }
            ConditionOperation::NotEquals => !self.evaluate_condition(value, &ConditionOperation::Equals, expected),
            ConditionOperation::GreaterThan => {
                if let ConditionValue::Integer(i) = expected {
                    value.parse::<i32>().map(|v| v > *i).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionOperation::LessThan => {
                if let ConditionValue::Integer(i) = expected {
                    value.parse::<i32>().map(|v| v < *i).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionOperation::GreaterOrEqual => {
                if let ConditionValue::Integer(i) = expected {
                    value.parse::<i32>().map(|v| v >= *i).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionOperation::LessOrEqual => {
                if let ConditionValue::Integer(i) = expected {
                    value.parse::<i32>().map(|v| v <= *i).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionOperation::Contains => {
                if let ConditionValue::String(s) = expected {
                    value.contains(s)
                } else {
                    false
                }
            }
            ConditionOperation::StartsWith => {
                if let ConditionValue::String(s) = expected {
                    value.starts_with(s)
                } else {
                    false
                }
            }
            ConditionOperation::EndsWith => {
                if let ConditionValue::String(s) = expected {
                    value.ends_with(s)
                } else {
                    false
                }
            }
            ConditionOperation::Matches => {
                if let ConditionValue::String(pattern) = expected {
                    regex::Regex::new(pattern)
                        .map(|r| r.is_match(value))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionOperation::In => {
                if let ConditionValue::StringList(list) = expected {
                    list.contains(&value.to_string())
                } else {
                    false
                }
            }
        }
    }

    /// Check delegation chain for permissions
    async fn check_delegation(&self, request: &AccessRequest) -> (Vec<DelegationEntry>, bool) {
        let delegations = self.delegations.read().await;
        let mut chain = Vec::new();
        let mut current_agent = &request.subject.agent_id;

        for _ in 0..self.config.max_delegation_depth {
            // Find delegations where this agent is the delegatee
            for delegation in delegations.values() {
                if delegation.delegatee == *current_agent && delegation.is_active {
                    // Check expiry
                    if let Some(expires) = delegation.expires_at {
                        if expires < Utc::now() {
                            continue;
                        }
                    }

                    // Check if delegation grants access to this resource
                    let mut has_permission = false;
                    for perm in &delegation.permissions {
                        let matches_resource = glob_match(&perm.resource_pattern, &request.resource.resource_id);
                        let matches_action = perm.actions.contains(&request.action.action_type);

                        if matches_resource && matches_action {
                            has_permission = true;
                            break;
                        }
                    }

                    if has_permission {
                        chain.push(delegation.clone());
                        current_agent = &delegation.delegator;

                        // Found a path to owner or admin
                        if *current_agent == request.resource.owner_id {
                            return (chain, true);
                        }
                    }
                }
            }
        }

        (chain, false)
    }

    /// Combine policy decisions using deny-overrides algorithm
    fn combine_decisions(&self, results: &[PolicyEvaluationResult], delegation_grants_access: bool) -> AccessDecision {
        // Deny-overrides: if any applicable policy denies, deny access
        for result in results {
            if result.policy.effect == PolicyEffect::Deny {
                return AccessDecision::Deny;
            }
        }

        // If any policy permits, permit access
        for result in results {
            if result.policy.effect == PolicyEffect::Permit {
                return AccessDecision::Permit;
            }
        }

        // Check delegation
        if delegation_grants_access {
            return AccessDecision::Permit;
        }

        // No applicable policies
        AccessDecision::NotApplicable
    }

    /// Generate human-readable decision reason
    fn generate_decision_reason(
        &self,
        decision: &AccessDecision,
        applicable: &[PolicyEvaluationResult],
        delegation: bool,
    ) -> String {
        match decision {
            AccessDecision::Permit => {
                if !applicable.is_empty() {
                    format!(
                        "Access permitted by {} applicable policy(ies)",
                        applicable.len()
                    )
                } else if delegation {
                    "Access permitted via delegation chain".to_string()
                } else {
                    "Access permitted (no applicable deny policies)".to_string()
                }
            }
            AccessDecision::Deny => {
                let deny_policies: Vec<_> = applicable
                    .iter()
                    .filter(|r| r.policy.effect == PolicyEffect::Deny)
                    .collect();
                if let Some(first) = deny_policies.first() {
                    format!("Access denied by policy: {}", first.policy.name)
                } else {
                    "Access denied".to_string()
                }
            }
            AccessDecision::NotApplicable => "No applicable policies found".to_string(),
            AccessDecision::Indeterminate => "Unable to determine access decision".to_string(),
        }
    }

    /// Generate obligations based on decision
    fn generate_obligations(&self, decision: &AccessDecision) -> Vec<Obligation> {
        let mut obligations = Vec::new();

        match decision {
            AccessDecision::Permit => {
                obligations.push(Obligation {
                    obligation_type: ObligationType::LogAccess,
                    parameters: HashMap::new(),
                });
            }
            AccessDecision::Deny => {
                obligations.push(Obligation {
                    obligation_type: ObligationType::LogAccess,
                    parameters: HashMap::new(),
                });
                obligations.push(Obligation {
                    obligation_type: ObligationType::NotifySecurity,
                    parameters: HashMap::new(),
                });
            }
            _ => {}
        }

        obligations
    }

    /// Get cached response
    async fn get_cached_response(&self, request: &AccessRequest) -> Option<AccessResponse> {
        let cache = self.cache.read().await;
        let key = self.generate_cache_key(request);

        if let Some(entry) = cache.get(&key) {
            if entry.expires_at > Utc::now() {
                return Some(entry.response.clone());
            }
        }
        None
    }

    /// Cache a response
    async fn cache_response(&self, request: &AccessRequest, response: &AccessResponse) {
        let key = self.generate_cache_key(request);
        let mut cache = self.cache.write().await;

        cache.insert(
            key,
            CacheEntry {
                response: response.clone(),
                expires_at: Utc::now() + chrono::Duration::seconds(self.config.cache_ttl_seconds as i64),
            },
        );
    }

    /// Generate cache key for request
    fn generate_cache_key(&self, request: &AccessRequest) -> String {
        request.cache_key()
    }

    /// Clear the decision cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Record audit entry
    async fn record_audit(&self, request: &AccessRequest, response: AccessResponse, evaluation_time_ms: u64) {
        let mut audit = self.audit_log.write().await;

        audit.push(AuditEntry {
            request: request.clone(),
            response,
            evaluation_time_ms,
            timestamp: Utc::now(),
        });

        // Keep only last 10000 entries
        while audit.len() > 10000 {
            audit.remove(0);
        }
    }

    /// Get audit log
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditEntry> {
        let audit = self.audit_log.read().await;
        let limit = limit.unwrap_or(1000);
        audit.iter().rev().take(limit).cloned().collect()
    }

    /// Check if subject has specific clearance level
    pub fn check_clearance(&self, subject: &SubjectAttributes, required_level: u8) -> bool {
        subject.clearance_level >= required_level
    }

    /// Check if subject owns the resource
    pub fn is_owner(&self, subject: &SubjectAttributes, resource: &ResourceAttributes) -> bool {
        subject.agent_id == resource.owner_id
    }

    /// Check if subject is in the same department as resource
    pub fn is_same_department(
        &self,
        subject: &SubjectAttributes,
        resource: &ResourceAttributes,
    ) -> bool {
        match (&subject.department, &resource.department) {
            (Some(sd), Some(rd)) => sd == rd,
            _ => false,
        }
    }
}

/// Simple glob matching (supports * and ?)
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    let mut dp = vec![vec![false; text_chars.len() + 1]; pattern_chars.len() + 1];
    dp[0][0] = true;

    // Handle patterns starting with *
    for i in 1..=pattern_chars.len() {
        if pattern_chars[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        }
    }

    for i in 1..=pattern_chars.len() {
        for j in 1..=text_chars.len() {
            match pattern_chars[i - 1] {
                '*' => {
                    dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
                }
                '?' => {
                    dp[i][j] = dp[i - 1][j - 1];
                }
                c => {
                    if c == text_chars[j - 1] {
                        dp[i][j] = dp[i - 1][j - 1];
                    }
                }
            }
        }
    }

    dp[pattern_chars.len()][text_chars.len()]
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    fn create_test_request(
        agent_id: &str,
        agent_type: AgentType,
        clearance_level: u8,
        resource_id: &str,
        owner_id: &str,
        sensitivity_level: u8,
        action: ActionType,
    ) -> AccessRequest {
        AccessRequest {
            subject: SubjectAttributes {
                agent_id: agent_id.to_string(),
                agent_type,
                clearance_level,
                custom_attributes: HashMap::new(),
                department: None,
                user_id: None,
            },
            resource: ResourceAttributes {
                resource_id: resource_id.to_string(),
                resource_type: ResourceType::Memory,
                owner_id: owner_id.to_string(),
                sensitivity_level,
                custom_attributes: HashMap::new(),
                classification: None,
                department: None,
            },
            environment: EnvironmentAttributes {
                timestamp: Utc::now(),
                location: None,
                source: Some("api".to_string()),
                security_context: SecurityContext {
                    is_secure: true,
                    request_id: None,
                    session_id: None,
                    mfa_verified: true,
                },
            },
            action: ActionAttributes {
                action_type: action,
                is_batch: false,
                batch_size: None,
                custom_attributes: HashMap::new(),
            },
        }
    }

    #[tokio::test]
    async fn test_abac_engine_basic_evaluation() {
        let engine = AbacEngine::default_engine();

        // Add a deny policy for high sensitivity resources
        let policy = AbacPolicy {
            policy_id: "deny_high_sensitivity".to_string(),
            name: "Deny High Sensitivity".to_string(),
            description: Some("Deny access to sensitivity level 5".to_string()),
            target: None,
            conditions: vec![PolicyCondition {
                category: AttributeCategory::Resource,
                attribute_name: "sensitivity_level".to_string(),
                operation: ConditionOperation::GreaterOrEqual,
                value: ConditionValue::Integer(5),
            }],
            effect: PolicyEffect::Deny,
            priority: 100,
            is_active: true,
        };

        engine.add_policy(policy).await.unwrap();

        // Test: Agent with clearance 3 trying to access sensitivity 5 resource
        let request = create_test_request(
            "agent1",
            AgentType::General,
            3, // clearance level
            "memory1",
            "owner1",
            5, // sensitivity level
            ActionType::Read,
        );

        let response = engine.evaluate(&request).await;
        assert_eq!(response.decision, AccessDecision::Deny);
    }

    #[tokio::test]
    async fn test_abac_engine_owner_access() {
        let engine = AbacEngine::default_engine();

        // Owner can access their own resources
        let request = create_test_request(
            "owner1",
            AgentType::General,
            1,
            "memory1",
            "owner1", // same as subject
            3,
            ActionType::Read,
        );

        let response = engine.evaluate(&request).await;
        // No applicable deny policies, so NotApplicable (or Permit via delegation)
        assert!(matches!(
            response.decision,
            AccessDecision::Permit | AccessDecision::NotApplicable
        ));
    }

    #[tokio::test]
    async fn test_abac_engine_clearance_level() {
        let engine = AbacEngine::default_engine();

        // Policy requiring clearance 4 for admin operations
        let policy = AbacPolicy {
            policy_id: "admin_requires_clearance".to_string(),
            name: "Admin Requires Clearance".to_string(),
            description: None,
            target: Some(PolicyTarget {
                subject_attrs: None,
                resource_types: Some(vec![ResourceType::Memory].into_iter().collect()),
                action_types: Some(vec![ActionType::Admin].into_iter().collect()),
            }),
            conditions: vec![PolicyCondition {
                category: AttributeCategory::Subject,
                attribute_name: "clearance_level".to_string(),
                operation: ConditionOperation::LessThan,
                value: ConditionValue::Integer(4),
            }],
            effect: PolicyEffect::Deny,
            priority: 100,
            is_active: true,
        };

        engine.add_policy(policy).await.unwrap();

        // Test: Agent with clearance 3 trying to do admin operation
        let request = create_test_request(
            "agent1",
            AgentType::General,
            3, // clearance level
            "memory1",
            "owner1",
            3,
            ActionType::Admin,
        );

        let response = engine.evaluate(&request).await;
        assert_eq!(response.decision, AccessDecision::Deny);
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("memory*", "memory123"));
        assert!(glob_match("memory*", "memory"));
        assert!(glob_match("*", "anything"));
        assert!(!glob_match("memory*", "other"));
        assert!(glob_match("mem_?", "mem_1"));
        assert!(!glob_match("mem_?", "mem_12"));
    }

    #[tokio::test]
    async fn test_delegation_chain() {
        let engine = AbacEngine::default_engine();

        // Add delegation: owner1 delegates to agent1
        let delegation = DelegationEntry {
            delegator: "owner1".to_string(),
            delegatee: "agent1".to_string(),
            permissions: vec![DelegatedPermission {
                resource_pattern: "memory*".to_string(),
                actions: vec![ActionType::Read, ActionType::Write],
            }],
            delegated_at: Utc::now(),
            expires_at: None,
            is_active: true,
        };

        engine.add_delegation(delegation).await.unwrap();

        // Test: Agent1 accessing owner's memory via delegation
        let request = create_test_request(
            "agent1",
            AgentType::General,
            1,
            "memory1",
            "owner1", // owner
            3,
            ActionType::Read,
        );

        let response = engine.evaluate(&request).await;
        assert_eq!(response.decision, AccessDecision::Permit);
    }

    #[tokio::test]
    async fn test_deny_overrides() {
        let engine = AbacEngine::default_engine();

        // Add permit policy
        let permit_policy = AbacPolicy {
            policy_id: "permit_read".to_string(),
            name: "Permit Read".to_string(),
            description: None,
            target: Some(PolicyTarget {
                subject_attrs: None,
                resource_types: Some(vec![ResourceType::Memory].into_iter().collect()),
                action_types: Some(vec![ActionType::Read].into_iter().collect()),
            }),
            conditions: vec![],
            effect: PolicyEffect::Permit,
            priority: 50,
            is_active: true,
        };

        // Add deny policy (higher priority)
        let deny_policy = AbacPolicy {
            policy_id: "deny_confidential".to_string(),
            name: "Deny Confidential".to_string(),
            description: None,
            target: None,
            conditions: vec![PolicyCondition {
                category: AttributeCategory::Resource,
                attribute_name: "classification".to_string(),
                operation: ConditionOperation::Equals,
                value: ConditionValue::String("confidential".to_string()),
            }],
            effect: PolicyEffect::Deny,
            priority: 100,
            is_active: true,
        };

        engine.add_policy(permit_policy).await.unwrap();
        engine.add_policy(deny_policy).await.unwrap();

        // Test: Read on confidential resource should be denied
        let request = create_test_request("agent1", AgentType::General, 5, "mem1", "owner1", 3, ActionType::Read);
        let mut req_with_class = request;
        req_with_class.resource.classification = Some("confidential".to_string());

        let response = engine.evaluate(&req_with_class).await;
        assert_eq!(response.decision, AccessDecision::Deny); // Deny overrides permit
    }
}
