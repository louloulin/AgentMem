//! Knowledge Vault 安全存储管理器
//!
//! 提供敏感信息的加密存储、访问控制和权限管理功能。
//! 支持多级敏感度分类和完整的审计日志记录。

use crate::{CoreError, CoreResult};
use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Knowledge Vault 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeVaultConfig {
    /// 最大存储条目数
    pub max_entries: usize,
    /// 默认加密密钥（生产环境应从安全存储获取）
    pub encryption_key: String,
    /// 访问控制启用
    pub access_control_enabled: bool,
    /// 审计日志启用
    pub audit_logging_enabled: bool,
    /// 敏感度自动分类启用
    pub auto_classification_enabled: bool,
}

impl Default for KnowledgeVaultConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            encryption_key: "default_key_change_in_production".to_string(),
            access_control_enabled: true,
            audit_logging_enabled: true,
            auto_classification_enabled: true,
        }
    }
}

/// 敏感度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensitivityLevel {
    /// 公开信息
    Public,
    /// 内部信息
    Internal,
    /// 机密信息
    Confidential,
    /// 绝密信息
    TopSecret,
}

impl SensitivityLevel {
    /// 获取所有敏感度级别
    pub fn all_levels() -> Vec<SensitivityLevel> {
        vec![
            SensitivityLevel::Public,
            SensitivityLevel::Internal,
            SensitivityLevel::Confidential,
            SensitivityLevel::TopSecret,
        ]
    }

    /// 获取级别描述
    pub fn description(&self) -> &'static str {
        match self {
            SensitivityLevel::Public => "公开信息，无访问限制",
            SensitivityLevel::Internal => "内部信息，需要基本权限",
            SensitivityLevel::Confidential => "机密信息，需要特殊权限",
            SensitivityLevel::TopSecret => "绝密信息，需要最高权限",
        }
    }

    /// 获取数值等级（用于权限比较）
    pub fn level_value(&self) -> u8 {
        match self {
            SensitivityLevel::Public => 0,
            SensitivityLevel::Internal => 1,
            SensitivityLevel::Confidential => 2,
            SensitivityLevel::TopSecret => 3,
        }
    }
}

/// 访问权限
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessPermission {
    /// 读取权限
    Read,
    /// 写入权限
    Write,
    /// 删除权限
    Delete,
    /// 管理权限（包含所有权限）
    Admin,
}

/// 用户权限信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    /// 用户ID
    pub user_id: String,
    /// 用户角色
    pub role: String,
    /// 最大访问敏感度级别
    pub max_sensitivity_level: SensitivityLevel,
    /// 具体权限列表
    pub permissions: Vec<AccessPermission>,
    /// 权限创建时间
    pub created_at: DateTime<Utc>,
    /// 权限过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 是否激活
    pub active: bool,
}

/// 知识条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// 条目ID
    pub id: String,
    /// 条目标题
    pub title: String,
    /// 加密内容
    pub encrypted_content: Vec<u8>,
    /// 加密随机数
    pub nonce: Vec<u8>,
    /// 敏感度级别
    pub sensitivity_level: SensitivityLevel,
    /// 创建者ID
    pub creator_id: String,
    /// 标签
    pub tags: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后修改时间
    pub updated_at: DateTime<Utc>,
    /// 最后访问时间
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// 访问次数
    pub access_count: u64,
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// 日志ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 操作类型
    pub action: AuditAction,
    /// 目标知识条目ID
    pub knowledge_id: Option<String>,
    /// 操作结果
    pub success: bool,
    /// 错误信息（如果失败）
    pub error_message: Option<String>,
    /// 操作时间
    pub timestamp: DateTime<Utc>,
    /// 客户端IP
    pub client_ip: Option<String>,
    /// 用户代理
    pub user_agent: Option<String>,
    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// 审计操作类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditAction {
    /// 创建知识条目
    CreateKnowledge,
    /// 读取知识条目
    ReadKnowledge,
    /// 更新知识条目
    UpdateKnowledge,
    /// 删除知识条目
    DeleteKnowledge,
    /// 搜索知识
    SearchKnowledge,
    /// 权限检查
    PermissionCheck,
    /// 登录
    Login,
    /// 登出
    Logout,
    /// 权限变更
    PermissionChange,
}

/// Knowledge Vault 统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeVaultStats {
    /// 总知识条目数
    pub total_entries: usize,
    /// 按敏感度级别分布
    pub entries_by_sensitivity: HashMap<SensitivityLevel, usize>,
    /// 按创建者分布
    pub entries_by_creator: HashMap<String, usize>,
    /// 总用户数
    pub total_users: usize,
    /// 活跃用户数
    pub active_users: usize,
    /// 总访问次数
    pub total_accesses: u64,
    /// 审计日志条目数
    pub audit_log_entries: usize,
    /// 平均访问次数
    pub average_access_count: f64,
    /// 最后统计时间
    pub last_updated: DateTime<Utc>,
}

/// Knowledge Vault 管理器
#[derive(Debug)]
pub struct KnowledgeVaultManager {
    /// 配置
    config: KnowledgeVaultConfig,
    /// 知识条目存储
    knowledge_entries: Arc<RwLock<HashMap<String, KnowledgeEntry>>>,
    /// 用户权限存储
    user_permissions: Arc<RwLock<HashMap<String, UserPermissions>>>,
    /// 审计日志存储
    audit_logs: Arc<RwLock<Vec<AuditLogEntry>>>,
    /// 统计信息
    stats: Arc<RwLock<KnowledgeVaultStats>>,
    /// 加密密钥
    encryption_key: Arc<Key<Aes256Gcm>>,
}

impl KnowledgeVaultManager {
    /// 创建新的 Knowledge Vault 管理器
    pub fn new(config: KnowledgeVaultConfig) -> CoreResult<Self> {
        // 生成加密密钥
        let key_bytes = Self::derive_key(&config.encryption_key)?;
        let encryption_key = Arc::new(*Key::<Aes256Gcm>::from_slice(&key_bytes));

        let stats = KnowledgeVaultStats {
            total_entries: 0,
            entries_by_sensitivity: HashMap::new(),
            entries_by_creator: HashMap::new(),
            total_users: 0,
            active_users: 0,
            total_accesses: 0,
            audit_log_entries: 0,
            average_access_count: 0.0,
            last_updated: Utc::now(),
        };

        Ok(Self {
            config,
            knowledge_entries: Arc::new(RwLock::new(HashMap::new())),
            user_permissions: Arc::new(RwLock::new(HashMap::new())),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(stats)),
            encryption_key,
        })
    }

    /// 从密码派生加密密钥
    fn derive_key(password: &str) -> CoreResult<[u8; 32]> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(b"knowledge_vault_salt"); // 添加盐值
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result[..32]);
        Ok(key)
    }

    /// 生成随机nonce
    fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        thread_rng().fill(&mut nonce);
        nonce
    }

    /// 加密内容
    fn encrypt_content(&self, content: &str) -> CoreResult<(Vec<u8>, Vec<u8>)> {
        let cipher = Aes256Gcm::new(&self.encryption_key);
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher
            .encrypt(nonce, content.as_bytes())
            .map_err(|e| CoreError::InvalidInput(format!("加密失败: {}", e)))?;

        Ok((encrypted, nonce_bytes.to_vec()))
    }

    /// 解密内容
    fn decrypt_content(&self, encrypted_content: &[u8], nonce: &[u8]) -> CoreResult<String> {
        let cipher = Aes256Gcm::new(&self.encryption_key);
        let nonce = Nonce::from_slice(nonce);

        let decrypted = cipher
            .decrypt(nonce, encrypted_content)
            .map_err(|e| CoreError::InvalidInput(format!("解密失败: {}", e)))?;

        String::from_utf8(decrypted)
            .map_err(|e| CoreError::InvalidInput(format!("解密内容不是有效UTF-8: {}", e)))
    }

    /// 添加用户权限
    pub fn add_user_permissions(&self, permissions: UserPermissions) -> CoreResult<()> {
        let user_id = permissions.user_id.clone();

        let mut user_perms = self
            .user_permissions
            .write()
            .map_err(|_| CoreError::InvalidInput("获取用户权限写锁失败".to_string()))?;

        user_perms.insert(user_id.clone(), permissions);
        drop(user_perms);

        // 更新统计信息
        self.update_user_stats()?;

        // 记录审计日志
        if self.config.audit_logging_enabled {
            self.log_audit_action(
                &user_id,
                AuditAction::PermissionChange,
                None,
                true,
                None,
                None,
                None,
                HashMap::new(),
            )?;
        }

        Ok(())
    }

    /// 检查用户权限
    pub fn check_permission(
        &self,
        user_id: &str,
        required_permission: &AccessPermission,
        required_sensitivity: SensitivityLevel,
    ) -> CoreResult<bool> {
        let user_perms = self
            .user_permissions
            .read()
            .map_err(|_| CoreError::InvalidInput("获取用户权限读锁失败".to_string()))?;

        let permissions = user_perms
            .get(user_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("用户 {} 不存在", user_id)))?;

        // 检查用户是否激活
        if !permissions.active {
            return Ok(false);
        }

        // 检查权限是否过期
        if let Some(expires_at) = permissions.expires_at {
            if Utc::now() > expires_at {
                return Ok(false);
            }
        }

        // 检查敏感度级别权限
        if permissions.max_sensitivity_level.level_value() < required_sensitivity.level_value() {
            return Ok(false);
        }

        // 检查具体权限
        let has_permission = permissions.permissions.contains(required_permission)
            || permissions.permissions.contains(&AccessPermission::Admin);

        // 记录权限检查审计日志
        if self.config.audit_logging_enabled {
            self.log_audit_action(
                user_id,
                AuditAction::PermissionCheck,
                None,
                has_permission,
                if has_permission {
                    None
                } else {
                    Some("权限不足".to_string())
                },
                None,
                None,
                HashMap::new(),
            )?;
        }

        Ok(has_permission)
    }

    /// 自动分类敏感度级别
    fn auto_classify_sensitivity(&self, content: &str) -> SensitivityLevel {
        if !self.config.auto_classification_enabled {
            return SensitivityLevel::Internal;
        }

        let content_lower = content.to_lowercase();

        // 绝密关键词
        let top_secret_keywords = [
            "密码",
            "password",
            "secret",
            "private_key",
            "api_key",
            "token",
            "机密",
            "绝密",
            "confidential",
            "classified",
            "restricted",
        ];

        // 机密关键词
        let confidential_keywords = [
            "内部",
            "internal",
            "proprietary",
            "sensitive",
            "personal",
            "财务",
            "financial",
            "salary",
            "revenue",
            "profit",
        ];

        // 内部关键词
        let internal_keywords = [
            "员工",
            "employee",
            "staff",
            "team",
            "department",
            "项目",
            "project",
            "plan",
            "strategy",
            "roadmap",
        ];

        // 检查绝密关键词
        for keyword in &top_secret_keywords {
            if content_lower.contains(keyword) {
                return SensitivityLevel::TopSecret;
            }
        }

        // 检查机密关键词
        for keyword in &confidential_keywords {
            if content_lower.contains(keyword) {
                return SensitivityLevel::Confidential;
            }
        }

        // 检查内部关键词
        for keyword in &internal_keywords {
            if content_lower.contains(keyword) {
                return SensitivityLevel::Internal;
            }
        }

        // 默认为公开
        SensitivityLevel::Public
    }

    /// 创建知识条目
    pub fn create_knowledge_entry(
        &self,
        user_id: &str,
        title: String,
        content: String,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
        sensitivity_level: Option<SensitivityLevel>,
    ) -> CoreResult<String> {
        // 检查创建权限
        if self.config.access_control_enabled {
            let required_sensitivity =
                sensitivity_level.unwrap_or_else(|| self.auto_classify_sensitivity(&content));
            if !self.check_permission(user_id, &AccessPermission::Write, required_sensitivity)? {
                return Err(CoreError::InvalidInput("用户没有创建权限".to_string()));
            }
        }

        // 生成ID
        let id = uuid::Uuid::new_v4().to_string();

        // 确定敏感度级别
        let final_sensitivity =
            sensitivity_level.unwrap_or_else(|| self.auto_classify_sensitivity(&content));

        // 加密内容
        let (encrypted_content, nonce) = self.encrypt_content(&content)?;

        // 创建知识条目
        let entry = KnowledgeEntry {
            id: id.clone(),
            title,
            encrypted_content,
            nonce,
            sensitivity_level: final_sensitivity,
            creator_id: user_id.to_string(),
            tags,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            access_count: 0,
        };

        // 存储条目
        let mut entries = self
            .knowledge_entries
            .write()
            .map_err(|_| CoreError::InvalidInput("获取知识条目写锁失败".to_string()))?;

        // 检查容量限制
        if entries.len() >= self.config.max_entries {
            return Err(CoreError::InvalidInput("知识库已达到最大容量".to_string()));
        }

        entries.insert(id.clone(), entry);
        drop(entries);

        // 更新统计信息
        self.update_knowledge_stats()?;

        // 记录审计日志
        if self.config.audit_logging_enabled {
            self.log_audit_action(
                user_id,
                AuditAction::CreateKnowledge,
                Some(id.clone()),
                true,
                None,
                None,
                None,
                HashMap::new(),
            )?;
        }

        Ok(id)
    }

    /// 读取知识条目
    pub fn read_knowledge_entry(
        &self,
        user_id: &str,
        knowledge_id: &str,
    ) -> CoreResult<(String, String)> {
        let mut entries = self
            .knowledge_entries
            .write()
            .map_err(|_| CoreError::InvalidInput("获取知识条目写锁失败".to_string()))?;

        let entry = entries
            .get_mut(knowledge_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("知识条目 {} 不存在", knowledge_id)))?;

        // 检查读取权限
        if self.config.access_control_enabled {
            if !self.check_permission(user_id, &AccessPermission::Read, entry.sensitivity_level)? {
                return Err(CoreError::InvalidInput("用户没有读取权限".to_string()));
            }
        }

        // 解密内容
        let content = self.decrypt_content(&entry.encrypted_content, &entry.nonce)?;

        // 更新访问信息
        entry.last_accessed_at = Some(Utc::now());
        entry.access_count += 1;

        let title = entry.title.clone();
        drop(entries);

        // 更新统计信息
        self.update_access_stats()?;

        // 记录审计日志
        if self.config.audit_logging_enabled {
            self.log_audit_action(
                user_id,
                AuditAction::ReadKnowledge,
                Some(knowledge_id.to_string()),
                true,
                None,
                None,
                None,
                HashMap::new(),
            )?;
        }

        Ok((title, content))
    }

    /// 更新知识条目
    pub fn update_knowledge_entry(
        &self,
        user_id: &str,
        knowledge_id: &str,
        title: Option<String>,
        content: Option<String>,
        tags: Option<Vec<String>>,
        metadata: Option<HashMap<String, String>>,
    ) -> CoreResult<()> {
        let mut entries = self
            .knowledge_entries
            .write()
            .map_err(|_| CoreError::InvalidInput("获取知识条目写锁失败".to_string()))?;

        let entry = entries
            .get_mut(knowledge_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("知识条目 {} 不存在", knowledge_id)))?;

        // 检查写入权限
        if self.config.access_control_enabled {
            if !self.check_permission(user_id, &AccessPermission::Write, entry.sensitivity_level)? {
                return Err(CoreError::InvalidInput("用户没有更新权限".to_string()));
            }
        }

        // 更新字段
        if let Some(new_title) = title {
            entry.title = new_title;
        }

        if let Some(new_content) = content {
            // 重新加密内容
            let (encrypted_content, nonce) = self.encrypt_content(&new_content)?;
            entry.encrypted_content = encrypted_content;
            entry.nonce = nonce;

            // 重新分类敏感度（如果启用）
            if self.config.auto_classification_enabled {
                entry.sensitivity_level = self.auto_classify_sensitivity(&new_content);
            }
        }

        if let Some(new_tags) = tags {
            entry.tags = new_tags;
        }

        if let Some(new_metadata) = metadata {
            entry.metadata = new_metadata;
        }

        entry.updated_at = Utc::now();
        drop(entries);

        // 记录审计日志
        if self.config.audit_logging_enabled {
            self.log_audit_action(
                user_id,
                AuditAction::UpdateKnowledge,
                Some(knowledge_id.to_string()),
                true,
                None,
                None,
                None,
                HashMap::new(),
            )?;
        }

        Ok(())
    }

    /// 删除知识条目
    pub fn delete_knowledge_entry(&self, user_id: &str, knowledge_id: &str) -> CoreResult<()> {
        let mut entries = self
            .knowledge_entries
            .write()
            .map_err(|_| CoreError::InvalidInput("获取知识条目写锁失败".to_string()))?;

        let entry = entries
            .get(knowledge_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("知识条目 {} 不存在", knowledge_id)))?;

        // 检查删除权限
        if self.config.access_control_enabled {
            if !self.check_permission(
                user_id,
                &AccessPermission::Delete,
                entry.sensitivity_level,
            )? {
                return Err(CoreError::InvalidInput("用户没有删除权限".to_string()));
            }
        }

        entries.remove(knowledge_id);
        drop(entries);

        // 更新统计信息
        self.update_knowledge_stats()?;

        // 记录审计日志
        if self.config.audit_logging_enabled {
            self.log_audit_action(
                user_id,
                AuditAction::DeleteKnowledge,
                Some(knowledge_id.to_string()),
                true,
                None,
                None,
                None,
                HashMap::new(),
            )?;
        }

        Ok(())
    }

    /// 搜索知识条目
    pub fn search_knowledge_entries(
        &self,
        user_id: &str,
        query: &str,
        sensitivity_filter: Option<SensitivityLevel>,
        tag_filter: Option<&str>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<(String, String, SensitivityLevel)>> {
        let entries = self
            .knowledge_entries
            .read()
            .map_err(|_| CoreError::InvalidInput("获取知识条目读锁失败".to_string()))?;

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for (id, entry) in entries.iter() {
            // 检查读取权限
            if self.config.access_control_enabled {
                if !self.check_permission(
                    user_id,
                    &AccessPermission::Read,
                    entry.sensitivity_level,
                )? {
                    continue;
                }
            }

            // 应用敏感度过滤器
            if let Some(filter_level) = sensitivity_filter {
                if entry.sensitivity_level != filter_level {
                    continue;
                }
            }

            // 应用标签过滤器
            if let Some(tag_filter) = tag_filter {
                if !entry.tags.iter().any(|tag| tag.contains(tag_filter)) {
                    continue;
                }
            }

            // 搜索标题和标签
            let title_match = entry.title.to_lowercase().contains(&query_lower);
            let tag_match = entry
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&query_lower));

            if title_match || tag_match {
                results.push((id.clone(), entry.title.clone(), entry.sensitivity_level));

                // 应用限制
                if let Some(limit) = limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        drop(entries);

        // 记录审计日志
        if self.config.audit_logging_enabled {
            let mut metadata = HashMap::new();
            metadata.insert("query".to_string(), query.to_string());
            metadata.insert("results_count".to_string(), results.len().to_string());

            self.log_audit_action(
                user_id,
                AuditAction::SearchKnowledge,
                None,
                true,
                None,
                None,
                None,
                metadata,
            )?;
        }

        Ok(results)
    }

    /// 记录审计日志
    fn log_audit_action(
        &self,
        user_id: &str,
        action: AuditAction,
        knowledge_id: Option<String>,
        success: bool,
        error_message: Option<String>,
        client_ip: Option<String>,
        user_agent: Option<String>,
        metadata: HashMap<String, String>,
    ) -> CoreResult<()> {
        let mut logs = self
            .audit_logs
            .write()
            .map_err(|_| CoreError::InvalidInput("获取审计日志写锁失败".to_string()))?;

        let log_entry = AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            action,
            knowledge_id,
            success,
            error_message,
            timestamp: Utc::now(),
            client_ip,
            user_agent,
            metadata,
        };

        logs.push(log_entry);

        // 更新统计信息
        let mut stats = self
            .stats
            .write()
            .map_err(|_| CoreError::InvalidInput("获取统计信息写锁失败".to_string()))?;
        stats.audit_log_entries = logs.len();

        Ok(())
    }

    /// 更新知识统计信息
    fn update_knowledge_stats(&self) -> CoreResult<()> {
        let entries = self
            .knowledge_entries
            .read()
            .map_err(|_| CoreError::InvalidInput("获取知识条目读锁失败".to_string()))?;

        let mut stats = self
            .stats
            .write()
            .map_err(|_| CoreError::InvalidInput("获取统计信息写锁失败".to_string()))?;

        stats.total_entries = entries.len();
        stats.entries_by_sensitivity.clear();
        stats.entries_by_creator.clear();

        let mut total_accesses = 0u64;

        for entry in entries.values() {
            // 按敏感度统计
            *stats
                .entries_by_sensitivity
                .entry(entry.sensitivity_level)
                .or_insert(0) += 1;

            // 按创建者统计
            *stats
                .entries_by_creator
                .entry(entry.creator_id.clone())
                .or_insert(0) += 1;

            // 累计访问次数
            total_accesses += entry.access_count;
        }

        stats.total_accesses = total_accesses;
        stats.average_access_count = if stats.total_entries > 0 {
            total_accesses as f64 / stats.total_entries as f64
        } else {
            0.0
        };
        stats.last_updated = Utc::now();

        Ok(())
    }

    /// 更新用户统计信息
    fn update_user_stats(&self) -> CoreResult<()> {
        let user_perms = self
            .user_permissions
            .read()
            .map_err(|_| CoreError::InvalidInput("获取用户权限读锁失败".to_string()))?;

        let mut stats = self
            .stats
            .write()
            .map_err(|_| CoreError::InvalidInput("获取统计信息写锁失败".to_string()))?;

        stats.total_users = user_perms.len();
        stats.active_users = user_perms.values().filter(|p| p.active).count();

        Ok(())
    }

    /// 更新访问统计信息
    fn update_access_stats(&self) -> CoreResult<()> {
        let entries = self
            .knowledge_entries
            .read()
            .map_err(|_| CoreError::InvalidInput("获取知识条目读锁失败".to_string()))?;

        let mut stats = self
            .stats
            .write()
            .map_err(|_| CoreError::InvalidInput("获取统计信息写锁失败".to_string()))?;

        let total_accesses: u64 = entries.values().map(|e| e.access_count).sum();
        stats.total_accesses = total_accesses;
        stats.average_access_count = if stats.total_entries > 0 {
            total_accesses as f64 / stats.total_entries as f64
        } else {
            0.0
        };

        Ok(())
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CoreResult<KnowledgeVaultStats> {
        let stats = self
            .stats
            .read()
            .map_err(|_| CoreError::InvalidInput("获取统计信息读锁失败".to_string()))?;
        Ok(stats.clone())
    }

    /// 获取审计日志
    pub fn get_audit_logs(
        &self,
        user_id: Option<&str>,
        action_filter: Option<AuditAction>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<AuditLogEntry>> {
        let logs = self
            .audit_logs
            .read()
            .map_err(|_| CoreError::InvalidInput("获取审计日志读锁失败".to_string()))?;

        let mut filtered_logs: Vec<AuditLogEntry> = logs
            .iter()
            .filter(|log| {
                if let Some(user_filter) = user_id {
                    if log.user_id != user_filter {
                        return false;
                    }
                }

                if let Some(action_filter) = &action_filter {
                    if std::mem::discriminant(&log.action) != std::mem::discriminant(action_filter)
                    {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 按时间倒序排列
        filtered_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 应用限制
        if let Some(limit) = limit {
            filtered_logs.truncate(limit);
        }

        Ok(filtered_logs)
    }

    /// 列出用户权限
    pub fn list_user_permissions(&self) -> CoreResult<Vec<UserPermissions>> {
        let user_perms = self
            .user_permissions
            .read()
            .map_err(|_| CoreError::InvalidInput("获取用户权限读锁失败".to_string()))?;

        Ok(user_perms.values().cloned().collect())
    }

    /// 列出知识条目（仅元数据）
    pub fn list_knowledge_entries(
        &self,
        user_id: &str,
        sensitivity_filter: Option<SensitivityLevel>,
    ) -> CoreResult<Vec<(String, String, SensitivityLevel, DateTime<Utc>)>> {
        let entries = self
            .knowledge_entries
            .read()
            .map_err(|_| CoreError::InvalidInput("获取知识条目读锁失败".to_string()))?;

        let mut results = Vec::new();

        for (id, entry) in entries.iter() {
            // 检查读取权限
            if self.config.access_control_enabled {
                if !self.check_permission(
                    user_id,
                    &AccessPermission::Read,
                    entry.sensitivity_level,
                )? {
                    continue;
                }
            }

            // 应用敏感度过滤器
            if let Some(filter_level) = sensitivity_filter {
                if entry.sensitivity_level != filter_level {
                    continue;
                }
            }

            results.push((
                id.clone(),
                entry.title.clone(),
                entry.sensitivity_level,
                entry.created_at,
            ));
        }

        // 按创建时间倒序排列
        results.sort_by(|a, b| b.3.cmp(&a.3));

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> KnowledgeVaultManager {
        let config = KnowledgeVaultConfig::default();
        KnowledgeVaultManager::new(config).unwrap()
    }

    fn create_test_user_permissions() -> UserPermissions {
        UserPermissions {
            user_id: "test_user".to_string(),
            role: "admin".to_string(),
            max_sensitivity_level: SensitivityLevel::TopSecret,
            permissions: vec![AccessPermission::Admin],
            created_at: Utc::now(),
            expires_at: None,
            active: true,
        }
    }

    #[test]
    fn test_sensitivity_level_operations() {
        assert_eq!(SensitivityLevel::all_levels().len(), 4);
        assert_eq!(SensitivityLevel::Public.level_value(), 0);
        assert_eq!(SensitivityLevel::TopSecret.level_value(), 3);
        assert!(SensitivityLevel::TopSecret.description().contains("绝密"));
    }

    #[test]
    fn test_manager_creation() {
        let manager = create_test_manager();
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_users, 0);
    }

    #[test]
    fn test_user_permissions_management() {
        let manager = create_test_manager();
        let permissions = create_test_user_permissions();

        // 添加用户权限
        manager.add_user_permissions(permissions.clone()).unwrap();

        // 检查权限
        assert!(manager
            .check_permission(
                "test_user",
                &AccessPermission::Read,
                SensitivityLevel::Confidential
            )
            .unwrap());

        assert!(manager
            .check_permission(
                "test_user",
                &AccessPermission::Write,
                SensitivityLevel::TopSecret
            )
            .unwrap());

        // 列出用户权限
        let user_list = manager.list_user_permissions().unwrap();
        assert_eq!(user_list.len(), 1);
        assert_eq!(user_list[0].user_id, "test_user");
    }

    #[test]
    fn test_encryption_decryption() {
        let manager = create_test_manager();
        let content = "这是一个测试内容";

        let (encrypted, nonce) = manager.encrypt_content(content).unwrap();
        let decrypted = manager.decrypt_content(&encrypted, &nonce).unwrap();

        assert_eq!(content, decrypted);
        assert_ne!(encrypted, content.as_bytes());
    }

    #[test]
    fn test_auto_classification() {
        let manager = create_test_manager();

        // 测试绝密分类
        let top_secret_content = "这是一个包含密码的文档";
        assert_eq!(
            manager.auto_classify_sensitivity(top_secret_content),
            SensitivityLevel::TopSecret
        );

        // 测试机密分类
        let confidential_content = "这是内部财务报告";
        assert_eq!(
            manager.auto_classify_sensitivity(confidential_content),
            SensitivityLevel::Confidential
        );

        // 测试内部分类
        let internal_content = "这是员工项目计划";
        assert_eq!(
            manager.auto_classify_sensitivity(internal_content),
            SensitivityLevel::Internal
        );

        // 测试公开分类
        let public_content = "这是一般信息";
        assert_eq!(
            manager.auto_classify_sensitivity(public_content),
            SensitivityLevel::Public
        );
    }

    #[test]
    fn test_knowledge_entry_lifecycle() {
        let manager = create_test_manager();
        let permissions = create_test_user_permissions();
        manager.add_user_permissions(permissions).unwrap();

        // 创建知识条目
        let entry_id = manager
            .create_knowledge_entry(
                "test_user",
                "测试标题".to_string(),
                "测试内容".to_string(),
                vec!["tag1".to_string(), "tag2".to_string()],
                HashMap::new(),
                Some(SensitivityLevel::Internal),
            )
            .unwrap();

        // 读取知识条目
        let (title, content) = manager
            .read_knowledge_entry("test_user", &entry_id)
            .unwrap();
        assert_eq!(title, "测试标题");
        assert_eq!(content, "测试内容");

        // 更新知识条目
        manager
            .update_knowledge_entry(
                "test_user",
                &entry_id,
                Some("更新标题".to_string()),
                Some("更新内容".to_string()),
                None,
                None,
            )
            .unwrap();

        // 验证更新
        let (updated_title, updated_content) = manager
            .read_knowledge_entry("test_user", &entry_id)
            .unwrap();
        assert_eq!(updated_title, "更新标题");
        assert_eq!(updated_content, "更新内容");

        // 删除知识条目
        manager
            .delete_knowledge_entry("test_user", &entry_id)
            .unwrap();

        // 验证删除
        assert!(manager
            .read_knowledge_entry("test_user", &entry_id)
            .is_err());
    }

    #[test]
    fn test_search_functionality() {
        let manager = create_test_manager();
        let permissions = create_test_user_permissions();
        manager.add_user_permissions(permissions).unwrap();

        // 创建多个知识条目
        let _id1 = manager
            .create_knowledge_entry(
                "test_user",
                "Rust编程指南".to_string(),
                "关于Rust的内容".to_string(),
                vec!["rust".to_string(), "programming".to_string()],
                HashMap::new(),
                Some(SensitivityLevel::Public),
            )
            .unwrap();

        let _id2 = manager
            .create_knowledge_entry(
                "test_user",
                "Python教程".to_string(),
                "关于Python的内容".to_string(),
                vec!["python".to_string(), "tutorial".to_string()],
                HashMap::new(),
                Some(SensitivityLevel::Internal),
            )
            .unwrap();

        // 搜索测试
        let results = manager
            .search_knowledge_entries("test_user", "rust", None, None, None)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].1.contains("Rust"));

        // 按敏感度过滤搜索
        let public_results = manager
            .search_knowledge_entries("test_user", "", Some(SensitivityLevel::Public), None, None)
            .unwrap();
        assert_eq!(public_results.len(), 1);

        // 按标签过滤搜索
        let tag_results = manager
            .search_knowledge_entries("test_user", "", None, Some("python"), None)
            .unwrap();
        assert_eq!(tag_results.len(), 1);
    }

    #[test]
    fn test_access_control() {
        let manager = create_test_manager();

        // 创建受限用户权限
        let limited_permissions = UserPermissions {
            user_id: "limited_user".to_string(),
            role: "viewer".to_string(),
            max_sensitivity_level: SensitivityLevel::Internal,
            permissions: vec![AccessPermission::Read],
            created_at: Utc::now(),
            expires_at: None,
            active: true,
        };
        manager.add_user_permissions(limited_permissions).unwrap();

        // 管理员用户
        let admin_permissions = create_test_user_permissions();
        manager.add_user_permissions(admin_permissions).unwrap();

        // 管理员创建绝密条目
        let entry_id = manager
            .create_knowledge_entry(
                "test_user",
                "绝密文档".to_string(),
                "这是绝密内容".to_string(),
                vec![],
                HashMap::new(),
                Some(SensitivityLevel::TopSecret),
            )
            .unwrap();

        // 受限用户无法读取绝密条目
        assert!(manager
            .read_knowledge_entry("limited_user", &entry_id)
            .is_err());

        // 受限用户无法写入
        assert!(manager
            .create_knowledge_entry(
                "limited_user",
                "测试".to_string(),
                "测试".to_string(),
                vec![],
                HashMap::new(),
                Some(SensitivityLevel::Internal),
            )
            .is_err());
    }

    #[test]
    fn test_audit_logging() {
        let manager = create_test_manager();
        let permissions = create_test_user_permissions();
        manager.add_user_permissions(permissions).unwrap();

        // 执行一些操作
        let entry_id = manager
            .create_knowledge_entry(
                "test_user",
                "审计测试".to_string(),
                "测试内容".to_string(),
                vec![],
                HashMap::new(),
                None,
            )
            .unwrap();

        let _ = manager
            .read_knowledge_entry("test_user", &entry_id)
            .unwrap();

        // 检查审计日志
        let logs = manager.get_audit_logs(None, None, None).unwrap();
        assert!(logs.len() >= 3); // 权限变更 + 创建 + 读取

        // 按用户过滤
        let user_logs = manager
            .get_audit_logs(Some("test_user"), None, None)
            .unwrap();
        assert!(user_logs.len() >= 2); // 创建 + 读取

        // 按操作类型过滤
        let create_logs = manager
            .get_audit_logs(None, Some(AuditAction::CreateKnowledge), None)
            .unwrap();
        assert_eq!(create_logs.len(), 1);
    }

    #[test]
    fn test_statistics() {
        let manager = create_test_manager();
        let permissions = create_test_user_permissions();
        manager.add_user_permissions(permissions).unwrap();

        // 创建不同敏感度的条目
        let _id1 = manager
            .create_knowledge_entry(
                "test_user",
                "公开文档".to_string(),
                "公开内容".to_string(),
                vec![],
                HashMap::new(),
                Some(SensitivityLevel::Public),
            )
            .unwrap();

        let _id2 = manager
            .create_knowledge_entry(
                "test_user",
                "内部文档".to_string(),
                "内部内容".to_string(),
                vec![],
                HashMap::new(),
                Some(SensitivityLevel::Internal),
            )
            .unwrap();

        // 检查统计信息
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_users, 1);
        assert_eq!(stats.active_users, 1);
        assert_eq!(
            stats.entries_by_sensitivity.get(&SensitivityLevel::Public),
            Some(&1)
        );
        assert_eq!(
            stats
                .entries_by_sensitivity
                .get(&SensitivityLevel::Internal),
            Some(&1)
        );
        assert_eq!(stats.entries_by_creator.get("test_user"), Some(&2));
    }

    #[test]
    fn test_knowledge_vault_config_default() {
        let config = KnowledgeVaultConfig::default();

        assert_eq!(config.max_entries, 10000);
        assert!(config.access_control_enabled);
        assert!(config.audit_logging_enabled);
        assert!(config.auto_classification_enabled);
        assert!(!config.encryption_key.is_empty());
    }

    #[test]
    fn test_knowledge_vault_config_custom() {
        let config = KnowledgeVaultConfig {
            max_entries: 5000,
            encryption_key: "custom_key".to_string(),
            access_control_enabled: false,
            audit_logging_enabled: false,
            auto_classification_enabled: false,
        };

        assert_eq!(config.max_entries, 5000);
        assert!(!config.access_control_enabled);
        assert!(!config.audit_logging_enabled);
    }

    #[test]
    fn test_sensitivity_level_ordering() {
        assert!(SensitivityLevel::Public.level_value() < SensitivityLevel::Internal.level_value());
        assert!(SensitivityLevel::Internal.level_value() < SensitivityLevel::Confidential.level_value());
        assert!(SensitivityLevel::Confidential.level_value() < SensitivityLevel::TopSecret.level_value());
    }

    #[test]
    fn test_access_permission_types() {
        let read = AccessPermission::Read;
        let write = AccessPermission::Write;
        let admin = AccessPermission::Admin;

        assert_ne!(read, write);
        assert_ne!(write, admin);
        assert_ne!(read, admin);
    }

    #[test]
    fn test_user_permissions_creation() {
        let perms = UserPermissions {
            user_id: "user-123".to_string(),
            role: "developer".to_string(),
            max_sensitivity_level: SensitivityLevel::Confidential,
            permissions: vec![AccessPermission::Read, AccessPermission::Write],
            created_at: Utc::now(),
            expires_at: None,
            active: true,
        };

        assert_eq!(perms.user_id, "user-123");
        assert_eq!(perms.permissions.len(), 2);
        assert!(perms.active);
        assert!(perms.expires_at.is_none());
    }

    #[test]
    fn test_user_permissions_with_expiry() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);

        let perms = UserPermissions {
            user_id: "temp-user".to_string(),
            role: "contractor".to_string(),
            max_sensitivity_level: SensitivityLevel::Internal,
            permissions: vec![AccessPermission::Read],
            created_at: now,
            expires_at: Some(future),
            active: true,
        };

        assert!(perms.expires_at.is_some());
        assert!(perms.expires_at.unwrap() > now);
    }

    #[test]
    fn test_audit_action_types() {
        let create = AuditAction::CreateKnowledge;
        let read = AuditAction::ReadKnowledge;
        let update = AuditAction::UpdateKnowledge;
        let delete = AuditAction::DeleteKnowledge;
        let permission_change = AuditAction::PermissionChange;

        assert_ne!(create, read);
        assert_ne!(read, update);
        assert_ne!(update, delete);
        assert_ne!(delete, permission_change);
    }

    #[test]
    fn test_audit_log_entry_creation() {
        let now = Utc::now();

        let mut metadata = HashMap::new();
        metadata.insert("details".to_string(), "User accessed confidential document".to_string());

        let log = AuditLogEntry {
            id: "log-1".to_string(),
            user_id: "user-123".to_string(),
            action: AuditAction::ReadKnowledge,
            knowledge_id: Some("entry-456".to_string()),
            timestamp: now,
            client_ip: Some("192.168.1.1".to_string()),
            user_agent: None,
            success: true,
            error_message: None,
            metadata,
        };

        assert_eq!(log.user_id, "user-123");
        assert!(log.success);
        assert!(log.knowledge_id.is_some());
        assert!(log.metadata.contains_key("details"));
    }

    #[test]
    fn test_vault_statistics_empty() {
        let stats = KnowledgeVaultStats {
            total_entries: 0,
            total_users: 0,
            active_users: 0,
            entries_by_sensitivity: HashMap::new(),
            entries_by_creator: HashMap::new(),
            total_accesses: 0,
            audit_log_entries: 0,
            average_access_count: 0.0,
            last_updated: Utc::now(),
        };

        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.total_accesses, 0);
    }

    #[test]
    fn test_vault_statistics_with_data() {
        let mut entries_by_sensitivity = HashMap::new();
        entries_by_sensitivity.insert(SensitivityLevel::Public, 5);
        entries_by_sensitivity.insert(SensitivityLevel::Internal, 3);

        let mut entries_by_creator = HashMap::new();
        entries_by_creator.insert("user1".to_string(), 4);
        entries_by_creator.insert("user2".to_string(), 4);

        let stats = KnowledgeVaultStats {
            total_entries: 8,
            total_users: 2,
            active_users: 2,
            entries_by_sensitivity,
            entries_by_creator,
            total_accesses: 100,
            audit_log_entries: 15,
            average_access_count: 12.5,
            last_updated: Utc::now(),
        };

        assert_eq!(stats.total_entries, 8);
        assert_eq!(stats.total_users, 2);
        assert_eq!(stats.total_accesses, 100);
    }
}
