//! Contextual Memory 环境感知管理器
//!
//! 负责管理与环境、上下文相关的记忆，包括时间、地点、情境、环境状态等信息。
//! 支持环境变化检测、上下文关联分析和智能环境适应。

use crate::{CoreError, CoreResult};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Contextual Memory 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualMemoryConfig {
    /// 最大上下文条目数
    pub max_contexts: usize,
    /// 上下文过期时间（秒）
    pub context_expiry_seconds: u64,
    /// 环境变化检测阈值
    pub change_detection_threshold: f32,
    /// 自动清理过期上下文
    pub auto_cleanup_enabled: bool,
    /// 上下文关联分析启用
    pub correlation_analysis_enabled: bool,
    /// 环境适应学习启用
    pub adaptive_learning_enabled: bool,
}

impl Default for ContextualMemoryConfig {
    fn default() -> Self {
        Self {
            max_contexts: 50000,
            context_expiry_seconds: 86400 * 7, // 7天
            change_detection_threshold: 0.3,
            auto_cleanup_enabled: true,
            correlation_analysis_enabled: true,
            adaptive_learning_enabled: true,
        }
    }
}

/// 环境类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvironmentType {
    /// 物理环境
    Physical,
    /// 数字环境
    Digital,
    /// 社交环境
    Social,
    /// 工作环境
    Work,
    /// 学习环境
    Learning,
    /// 娱乐环境
    Entertainment,
    /// 其他环境
    Other,
}

impl EnvironmentType {
    /// 获取所有环境类型
    pub fn all_types() -> Vec<EnvironmentType> {
        vec![
            EnvironmentType::Physical,
            EnvironmentType::Digital,
            EnvironmentType::Social,
            EnvironmentType::Work,
            EnvironmentType::Learning,
            EnvironmentType::Entertainment,
            EnvironmentType::Other,
        ]
    }

    /// 获取环境类型描述
    pub fn description(&self) -> &'static str {
        match self {
            EnvironmentType::Physical => "物理环境：位置、天气、物理空间",
            EnvironmentType::Digital => "数字环境：应用、网站、数字平台",
            EnvironmentType::Social => "社交环境：人际交往、社交场合",
            EnvironmentType::Work => "工作环境：办公场所、工作任务",
            EnvironmentType::Learning => "学习环境：教育场所、学习活动",
            EnvironmentType::Entertainment => "娱乐环境：休闲活动、娱乐场所",
            EnvironmentType::Other => "其他环境：未分类的环境类型",
        }
    }
}

/// 上下文状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    /// 状态ID
    pub id: String,
    /// 环境类型
    pub environment_type: EnvironmentType,
    /// 位置信息
    pub location: Option<LocationInfo>,
    /// 时间信息
    pub temporal_info: TemporalInfo,
    /// 环境参数
    pub environment_params: HashMap<String, serde_json::Value>,
    /// 用户状态
    pub user_state: UserState,
    /// 设备信息
    pub device_info: Option<DeviceInfo>,
    /// 网络环境
    pub network_info: Option<NetworkInfo>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 重要性评分
    pub importance_score: f32,
    /// 访问次数
    pub access_count: u64,
}

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    /// 纬度
    pub latitude: Option<f64>,
    /// 经度
    pub longitude: Option<f64>,
    /// 地址描述
    pub address: Option<String>,
    /// 地点名称
    pub place_name: Option<String>,
    /// 地点类型
    pub place_type: Option<String>,
    /// 精度（米）
    pub accuracy: Option<f32>,
}

/// 时间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalInfo {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 时区
    pub timezone: String,
    /// 一天中的时间段
    pub time_of_day: TimeOfDay,
    /// 星期几
    pub day_of_week: u8, // 1-7, 1=Monday
    /// 是否工作日
    pub is_workday: bool,
    /// 季节
    pub season: Season,
}

/// 一天中的时间段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeOfDay {
    /// 早晨 (6-12)
    Morning,
    /// 下午 (12-18)
    Afternoon,
    /// 晚上 (18-22)
    Evening,
    /// 夜晚 (22-6)
    Night,
}

/// 季节
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    /// 春季
    Spring,
    /// 夏季
    Summer,
    /// 秋季
    Autumn,
    /// 冬季
    Winter,
}

/// 用户状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    /// 活动状态
    pub activity_state: ActivityState,
    /// 注意力水平
    pub attention_level: f32, // 0.0-1.0
    /// 情绪状态
    pub mood_state: Option<String>,
    /// 认知负荷
    pub cognitive_load: f32, // 0.0-1.0
    /// 疲劳程度
    pub fatigue_level: f32, // 0.0-1.0
    /// 压力水平
    pub stress_level: f32, // 0.0-1.0
}

/// 活动状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityState {
    /// 活跃
    Active,
    /// 空闲
    Idle,
    /// 忙碌
    Busy,
    /// 专注
    Focused,
    /// 休息
    Resting,
    /// 离开
    Away,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备类型
    pub device_type: String,
    /// 操作系统
    pub operating_system: String,
    /// 浏览器信息
    pub browser_info: Option<String>,
    /// 屏幕分辨率
    pub screen_resolution: Option<String>,
    /// 设备方向
    pub orientation: Option<String>,
    /// 电池状态
    pub battery_level: Option<f32>,
    /// 网络类型
    pub connection_type: Option<String>,
}

/// 网络信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// IP地址
    pub ip_address: Option<String>,
    /// 网络类型
    pub network_type: String,
    /// 连接速度
    pub connection_speed: Option<f32>,
    /// 延迟
    pub latency: Option<f32>,
    /// 地理位置
    pub geo_location: Option<String>,
}

/// 上下文关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCorrelation {
    /// 关联ID
    pub id: String,
    /// 源上下文ID
    pub source_context_id: String,
    /// 目标上下文ID
    pub target_context_id: String,
    /// 关联类型
    pub correlation_type: CorrelationType,
    /// 关联强度
    pub strength: f32, // 0.0-1.0
    /// 置信度
    pub confidence: f32, // 0.0-1.0
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后验证时间
    pub last_verified_at: DateTime<Utc>,
}

/// 关联类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationType {
    /// 时间关联
    Temporal,
    /// 空间关联
    Spatial,
    /// 因果关联
    Causal,
    /// 相似关联
    Similar,
    /// 序列关联
    Sequential,
    /// 条件关联
    Conditional,
}

/// 环境变化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentChangeEvent {
    /// 事件ID
    pub id: String,
    /// 变化类型
    pub change_type: ChangeType,
    /// 变化前状态
    pub before_state: Option<String>,
    /// 变化后状态
    pub after_state: String,
    /// 变化幅度
    pub change_magnitude: f32,
    /// 检测时间
    pub detected_at: DateTime<Utc>,
    /// 影响的上下文ID列表
    pub affected_contexts: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 变化类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// 位置变化
    LocationChange,
    /// 时间变化
    TimeChange,
    /// 环境参数变化
    EnvironmentChange,
    /// 用户状态变化
    UserStateChange,
    /// 设备变化
    DeviceChange,
    /// 网络变化
    NetworkChange,
}

/// Contextual Memory 统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextualMemoryStats {
    /// 总上下文数
    pub total_contexts: usize,
    /// 按环境类型分布
    pub contexts_by_environment: HashMap<EnvironmentType, usize>,
    /// 按时间段分布
    pub contexts_by_time_of_day: HashMap<TimeOfDay, usize>,
    /// 活跃上下文数
    pub active_contexts: usize,
    /// 过期上下文数
    pub expired_contexts: usize,
    /// 总关联数
    pub total_correlations: usize,
    /// 环境变化事件数
    pub change_events_count: usize,
    /// 平均重要性评分
    pub average_importance: f64,
    /// 平均访问次数
    pub average_access_count: f64,
    /// 最后统计时间
    pub last_updated: DateTime<Utc>,
}

/// Contextual Memory 管理器
#[derive(Debug)]
pub struct ContextualMemoryManager {
    /// 配置
    config: ContextualMemoryConfig,
    /// 上下文状态存储
    contexts: Arc<RwLock<HashMap<String, ContextState>>>,
    /// 上下文关联存储
    correlations: Arc<RwLock<HashMap<String, ContextCorrelation>>>,
    /// 环境变化事件存储
    change_events: Arc<RwLock<Vec<EnvironmentChangeEvent>>>,
    /// 统计信息
    stats: Arc<RwLock<ContextualMemoryStats>>,
    /// 当前活跃上下文ID
    current_context_id: Arc<RwLock<Option<String>>>,
}

impl ContextualMemoryManager {
    /// 创建新的 Contextual Memory 管理器
    pub fn new(config: ContextualMemoryConfig) -> Self {
        let stats = ContextualMemoryStats {
            total_contexts: 0,
            contexts_by_environment: HashMap::new(),
            contexts_by_time_of_day: HashMap::new(),
            active_contexts: 0,
            expired_contexts: 0,
            total_correlations: 0,
            change_events_count: 0,
            average_importance: 0.0,
            average_access_count: 0.0,
            last_updated: Utc::now(),
        };

        Self {
            config,
            contexts: Arc::new(RwLock::new(HashMap::new())),
            correlations: Arc::new(RwLock::new(HashMap::new())),
            change_events: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(stats)),
            current_context_id: Arc::new(RwLock::new(None)),
        }
    }

    /// 创建新的上下文状态
    pub fn create_context_state(
        &self,
        environment_type: EnvironmentType,
        location: Option<LocationInfo>,
        environment_params: HashMap<String, serde_json::Value>,
        user_state: UserState,
        device_info: Option<DeviceInfo>,
        network_info: Option<NetworkInfo>,
    ) -> CoreResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        // 计算过期时间
        let expires_at = if self.config.context_expiry_seconds > 0 {
            Some(now + chrono::Duration::seconds(self.config.context_expiry_seconds as i64))
        } else {
            None
        };

        // 生成时间信息
        let temporal_info = self.generate_temporal_info(now);

        // 计算重要性评分
        let importance_score =
            self.calculate_importance_score(&environment_type, &user_state, &environment_params);

        let context_state = ContextState {
            id: id.clone(),
            environment_type,
            location,
            temporal_info,
            environment_params,
            user_state,
            device_info,
            network_info,
            created_at: now,
            updated_at: now,
            expires_at,
            importance_score,
            access_count: 0,
        };

        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| CoreError::InvalidInput("获取上下文写锁失败".to_string()))?;

        // 检查容量限制
        if contexts.len() >= self.config.max_contexts {
            // 清理最旧的低重要性上下文
            self.cleanup_old_contexts(&mut contexts)?;
        }

        contexts.insert(id.clone(), context_state);
        drop(contexts);

        // 设置为当前活跃上下文
        self.set_current_context(&id)?;

        // 更新统计信息
        self.update_stats()?;

        // 检测环境变化
        if self.config.correlation_analysis_enabled {
            self.detect_environment_changes(&id)?;
        }

        Ok(id)
    }

    /// 获取上下文状态
    pub fn get_context_state(&self, context_id: &str) -> CoreResult<ContextState> {
        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| CoreError::InvalidInput("获取上下文写锁失败".to_string()))?;

        let context = contexts
            .get_mut(context_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("上下文 {context_id} 不存在")))?;

        // 检查是否过期
        if let Some(expires_at) = context.expires_at {
            if Utc::now() > expires_at {
                return Err(CoreError::InvalidInput("上下文已过期".to_string()));
            }
        }

        // 更新访问信息
        context.access_count += 1;
        context.updated_at = Utc::now();

        let result = context.clone();
        drop(contexts);

        // 更新访问统计
        self.update_access_stats()?;

        Ok(result)
    }

    /// 更新上下文状态
    pub fn update_context_state(
        &self,
        context_id: &str,
        environment_params: Option<HashMap<String, serde_json::Value>>,
        user_state: Option<UserState>,
        device_info: Option<DeviceInfo>,
        network_info: Option<NetworkInfo>,
    ) -> CoreResult<()> {
        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| CoreError::InvalidInput("获取上下文写锁失败".to_string()))?;

        let context = contexts
            .get_mut(context_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("上下文 {context_id} 不存在")))?;

        // 检查是否过期
        if let Some(expires_at) = context.expires_at {
            if Utc::now() > expires_at {
                return Err(CoreError::InvalidInput("上下文已过期".to_string()));
            }
        }

        // 记录变化前的状态
        let old_state = context.clone();

        // 更新字段
        if let Some(new_params) = environment_params {
            context.environment_params = new_params;
        }

        if let Some(new_user_state) = user_state {
            context.user_state = new_user_state;
        }

        if let Some(new_device_info) = device_info {
            context.device_info = Some(new_device_info);
        }

        if let Some(new_network_info) = network_info {
            context.network_info = Some(new_network_info);
        }

        // 重新计算重要性评分
        context.importance_score = self.calculate_importance_score(
            &context.environment_type,
            &context.user_state,
            &context.environment_params,
        );

        context.updated_at = Utc::now();
        drop(contexts);

        // 检测环境变化
        if self.config.correlation_analysis_enabled {
            self.detect_state_changes(&old_state, context_id)?;
        }

        Ok(())
    }

    /// 删除上下文状态
    pub fn delete_context_state(&self, context_id: &str) -> CoreResult<()> {
        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| CoreError::InvalidInput("获取上下文写锁失败".to_string()))?;

        contexts
            .remove(context_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("上下文 {context_id} 不存在")))?;

        drop(contexts);

        // 清理相关的关联
        self.cleanup_correlations_for_context(context_id)?;

        // 如果是当前活跃上下文，清除引用
        {
            let mut current_id = self
                .current_context_id
                .write()
                .map_err(|_| CoreError::InvalidInput("获取当前上下文写锁失败".to_string()))?;

            if let Some(ref current) = *current_id {
                if current == context_id {
                    *current_id = None;
                }
            }
        } // 释放写锁

        // 更新统计信息
        self.update_stats()?;

        Ok(())
    }

    /// 设置当前活跃上下文
    pub fn set_current_context(&self, context_id: &str) -> CoreResult<()> {
        // 验证上下文存在
        let contexts = self
            .contexts
            .read()
            .map_err(|_| CoreError::InvalidInput("获取上下文读锁失败".to_string()))?;

        if !contexts.contains_key(context_id) {
            return Err(CoreError::InvalidInput(format!(
                "上下文 {context_id} 不存在"
            )));
        }
        drop(contexts);

        let mut current_id = self
            .current_context_id
            .write()
            .map_err(|_| CoreError::InvalidInput("获取当前上下文写锁失败".to_string()))?;

        *current_id = Some(context_id.to_string());

        Ok(())
    }

    /// 获取当前活跃上下文
    pub fn get_current_context(&self) -> CoreResult<Option<ContextState>> {
        let current_id = self
            .current_context_id
            .read()
            .map_err(|_| CoreError::InvalidInput("获取当前上下文读锁失败".to_string()))?;

        if let Some(ref context_id) = *current_id {
            Ok(Some(self.get_context_state(context_id)?))
        } else {
            Ok(None)
        }
    }

    /// 搜索上下文状态
    pub fn search_contexts(
        &self,
        environment_type: Option<EnvironmentType>,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
        location_radius: Option<(f64, f64, f64)>, // (lat, lon, radius_km)
        min_importance: Option<f32>,
        user_activity: Option<ActivityState>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<ContextState>> {
        let contexts = self
            .contexts
            .read()
            .map_err(|_| CoreError::InvalidInput("获取上下文读锁失败".to_string()))?;

        let mut results: Vec<ContextState> = contexts
            .values()
            .filter(|context| {
                // 检查过期
                if let Some(expires_at) = context.expires_at {
                    if Utc::now() > expires_at {
                        return false;
                    }
                }

                // 环境类型过滤
                if let Some(env_type) = environment_type {
                    if context.environment_type != env_type {
                        return false;
                    }
                }

                // 时间范围过滤
                if let Some((start, end)) = time_range {
                    if context.created_at < start || context.created_at > end {
                        return false;
                    }
                }

                // 位置半径过滤
                if let Some((target_lat, target_lon, radius_km)) = location_radius {
                    if let Some(ref location) = context.location {
                        if let (Some(lat), Some(lon)) = (location.latitude, location.longitude) {
                            let distance =
                                self.calculate_distance(lat, lon, target_lat, target_lon);
                            if distance > radius_km {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // 重要性过滤
                if let Some(min_imp) = min_importance {
                    if context.importance_score < min_imp {
                        return false;
                    }
                }

                // 用户活动状态过滤
                if let Some(activity) = user_activity {
                    if context.user_state.activity_state != activity {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 按重要性和访问次数排序
        results.sort_by(|a, b| {
            let score_a = a.importance_score + (a.access_count as f32 * 0.01);
            let score_b = b.importance_score + (b.access_count as f32 * 0.01);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制结果数量
        if let Some(limit_count) = limit {
            results.truncate(limit_count);
        }

        Ok(results)
    }

    /// 创建上下文关联
    pub fn create_correlation(
        &self,
        source_context_id: &str,
        target_context_id: &str,
        correlation_type: CorrelationType,
        strength: f32,
        confidence: f32,
    ) -> CoreResult<String> {
        // 验证上下文存在
        let contexts = self
            .contexts
            .read()
            .map_err(|_| CoreError::InvalidInput("获取上下文读锁失败".to_string()))?;

        if !contexts.contains_key(source_context_id) {
            return Err(CoreError::InvalidInput(format!(
                "源上下文 {source_context_id} 不存在"
            )));
        }

        if !contexts.contains_key(target_context_id) {
            return Err(CoreError::InvalidInput(format!(
                "目标上下文 {target_context_id} 不存在"
            )));
        }
        drop(contexts);

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let correlation = ContextCorrelation {
            id: id.clone(),
            source_context_id: source_context_id.to_string(),
            target_context_id: target_context_id.to_string(),
            correlation_type,
            strength: strength.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            created_at: now,
            last_verified_at: now,
        };

        {
            let mut correlations = self
                .correlations
                .write()
                .map_err(|_| CoreError::InvalidInput("获取关联写锁失败".to_string()))?;

            correlations.insert(id.clone(), correlation);
        } // 释放写锁

        // 更新统计信息
        self.update_stats()?;

        Ok(id)
    }

    /// 获取上下文关联
    pub fn get_correlations_for_context(
        &self,
        context_id: &str,
    ) -> CoreResult<Vec<ContextCorrelation>> {
        let correlations = self
            .correlations
            .read()
            .map_err(|_| CoreError::InvalidInput("获取关联读锁失败".to_string()))?;

        let results: Vec<ContextCorrelation> = correlations
            .values()
            .filter(|correlation| {
                correlation.source_context_id == context_id
                    || correlation.target_context_id == context_id
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// 自动清理过期上下文
    pub fn cleanup_expired_contexts(&self) -> CoreResult<usize> {
        if !self.config.auto_cleanup_enabled {
            return Ok(0);
        }

        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| CoreError::InvalidInput("获取上下文写锁失败".to_string()))?;

        let now = Utc::now();
        let mut removed_count = 0;
        let mut to_remove = Vec::new();

        for (id, context) in contexts.iter() {
            if let Some(expires_at) = context.expires_at {
                if now > expires_at {
                    to_remove.push(id.clone());
                }
            }
        }

        for id in to_remove {
            contexts.remove(&id);
            removed_count += 1;

            // 清理相关关联
            self.cleanup_correlations_for_context(&id)?;
        }

        drop(contexts);

        if removed_count > 0 {
            self.update_stats()?;
        }

        Ok(removed_count)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CoreResult<ContextualMemoryStats> {
        let stats = self
            .stats
            .read()
            .map_err(|_| CoreError::InvalidInput("获取统计读锁失败".to_string()))?;

        Ok(stats.clone())
    }

    // 私有辅助方法

    /// 生成时间信息
    fn generate_temporal_info(&self, timestamp: DateTime<Utc>) -> TemporalInfo {
        let hour = timestamp.hour();
        let time_of_day = match hour {
            6..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=21 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        };

        let day_of_week = timestamp.weekday().number_from_monday() as u8;
        let is_workday = day_of_week <= 5;

        let month = timestamp.month();
        let season = match month {
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            _ => Season::Winter,
        };

        TemporalInfo {
            timestamp,
            timezone: "UTC".to_string(),
            time_of_day,
            day_of_week,
            is_workday,
            season,
        }
    }

    /// 计算重要性评分
    fn calculate_importance_score(
        &self,
        environment_type: &EnvironmentType,
        user_state: &UserState,
        environment_params: &HashMap<String, serde_json::Value>,
    ) -> f32 {
        let mut score = 0.5; // 基础分数

        // 环境类型权重
        score += match environment_type {
            EnvironmentType::Work => 0.3,
            EnvironmentType::Learning => 0.25,
            EnvironmentType::Social => 0.2,
            EnvironmentType::Physical => 0.15,
            EnvironmentType::Digital => 0.1,
            EnvironmentType::Entertainment => 0.05,
            EnvironmentType::Other => 0.0,
        };

        // 用户状态权重
        score += match user_state.activity_state {
            ActivityState::Focused => 0.2,
            ActivityState::Busy => 0.15,
            ActivityState::Active => 0.1,
            ActivityState::Idle => 0.05,
            ActivityState::Resting => 0.02,
            ActivityState::Away => 0.0,
        };

        // 注意力水平权重
        score += user_state.attention_level * 0.1;

        // 认知负荷权重（负荷高时重要性高）
        score += user_state.cognitive_load * 0.05;

        // 环境参数数量权重
        score += (environment_params.len() as f32 * 0.01).min(0.1);

        score.clamp(0.0, 1.0)
    }

    /// 计算两点间距离（公里）
    fn calculate_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371.0; // 地球半径（公里）
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

    /// 清理旧上下文
    fn cleanup_old_contexts(&self, contexts: &mut HashMap<String, ContextState>) -> CoreResult<()> {
        let mut contexts_vec: Vec<_> = contexts
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // 按重要性和访问次数排序，保留重要的
        contexts_vec.sort_by(|a, b| {
            let score_a = a.1.importance_score + (a.1.access_count as f32 * 0.01);
            let score_b = b.1.importance_score + (b.1.access_count as f32 * 0.01);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 移除最低重要性的10%
        let remove_count = (contexts.len() / 10).max(1);
        for i in 0..remove_count.min(contexts_vec.len()) {
            let id = &contexts_vec[i].0;
            contexts.remove(id);
        }

        Ok(())
    }

    /// 清理指定上下文的关联
    fn cleanup_correlations_for_context(&self, context_id: &str) -> CoreResult<()> {
        let mut correlations = self
            .correlations
            .write()
            .map_err(|_| CoreError::InvalidInput("获取关联写锁失败".to_string()))?;

        correlations.retain(|_, correlation| {
            correlation.source_context_id != context_id
                && correlation.target_context_id != context_id
        });

        Ok(())
    }

    /// 检测环境变化
    fn detect_environment_changes(&self, _context_id: &str) -> CoreResult<()> {
        // 这里可以实现环境变化检测逻辑
        // 比较当前上下文与历史上下文，检测显著变化
        Ok(())
    }

    /// 检测状态变化
    fn detect_state_changes(&self, _old_state: &ContextState, _context_id: &str) -> CoreResult<()> {
        // 这里可以实现状态变化检测逻辑
        // 比较新旧状态，记录变化事件
        Ok(())
    }

    /// 更新统计信息
    fn update_stats(&self) -> CoreResult<()> {
        let contexts = self
            .contexts
            .read()
            .map_err(|_| CoreError::InvalidInput("获取上下文读锁失败".to_string()))?;

        let correlations = self
            .correlations
            .read()
            .map_err(|_| CoreError::InvalidInput("获取关联读锁失败".to_string()))?;

        let change_events = self
            .change_events
            .read()
            .map_err(|_| CoreError::InvalidInput("获取变化事件读锁失败".to_string()))?;

        let mut stats = self
            .stats
            .write()
            .map_err(|_| CoreError::InvalidInput("获取统计写锁失败".to_string()))?;

        let now = Utc::now();
        let mut contexts_by_environment = HashMap::new();
        let mut contexts_by_time_of_day = HashMap::new();
        let mut active_contexts = 0;
        let mut expired_contexts = 0;
        let mut total_importance = 0.0;
        let mut total_access_count = 0;

        for context in contexts.values() {
            // 按环境类型统计
            *contexts_by_environment
                .entry(context.environment_type)
                .or_insert(0) += 1;

            // 按时间段统计
            *contexts_by_time_of_day
                .entry(context.temporal_info.time_of_day)
                .or_insert(0) += 1;

            // 活跃/过期统计
            if let Some(expires_at) = context.expires_at {
                if now > expires_at {
                    expired_contexts += 1;
                } else {
                    active_contexts += 1;
                }
            } else {
                active_contexts += 1;
            }

            total_importance += context.importance_score as f64;
            total_access_count += context.access_count;
        }

        stats.total_contexts = contexts.len();
        stats.contexts_by_environment = contexts_by_environment;
        stats.contexts_by_time_of_day = contexts_by_time_of_day;
        stats.active_contexts = active_contexts;
        stats.expired_contexts = expired_contexts;
        stats.total_correlations = correlations.len();
        stats.change_events_count = change_events.len();
        stats.average_importance = if !contexts.is_empty() {
            total_importance / contexts.len() as f64
        } else {
            0.0
        };
        stats.average_access_count = if !contexts.is_empty() {
            total_access_count as f64 / contexts.len() as f64
        } else {
            0.0
        };
        stats.last_updated = now;

        Ok(())
    }

    /// 更新访问统计
    fn update_access_stats(&self) -> CoreResult<()> {
        // 这里可以实现更细粒度的访问统计更新
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_manager() -> ContextualMemoryManager {
        let config = ContextualMemoryConfig {
            max_contexts: 100,
            context_expiry_seconds: 3600, // 1小时
            change_detection_threshold: 0.3,
            auto_cleanup_enabled: true,
            correlation_analysis_enabled: true,
            adaptive_learning_enabled: true,
        };
        ContextualMemoryManager::new(config)
    }

    fn create_test_user_state() -> UserState {
        UserState {
            activity_state: ActivityState::Active,
            attention_level: 0.8,
            mood_state: Some("focused".to_string()),
            cognitive_load: 0.6,
            fatigue_level: 0.3,
            stress_level: 0.4,
        }
    }

    fn create_test_location() -> LocationInfo {
        LocationInfo {
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            address: Some("San Francisco, CA".to_string()),
            place_name: Some("Office".to_string()),
            place_type: Some("workplace".to_string()),
            accuracy: Some(10.0),
        }
    }

    #[test]
    fn test_environment_type_all_types() {
        let types = EnvironmentType::all_types();
        assert_eq!(types.len(), 7);
        assert!(types.contains(&EnvironmentType::Physical));
        assert!(types.contains(&EnvironmentType::Digital));
        assert!(types.contains(&EnvironmentType::Social));
    }

    #[test]
    fn test_environment_type_description() {
        assert!(EnvironmentType::Physical.description().contains("物理环境"));
        assert!(EnvironmentType::Digital.description().contains("数字环境"));
        assert!(EnvironmentType::Work.description().contains("工作环境"));
    }

    #[test]
    fn test_create_contextual_memory_manager() {
        let manager = create_test_manager();
        let stats = manager.get_stats().unwrap();

        assert_eq!(stats.total_contexts, 0);
        assert_eq!(stats.active_contexts, 0);
        assert_eq!(stats.total_correlations, 0);
    }

    #[test]
    fn test_create_context_state() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();
        let location = create_test_location();
        let mut env_params = HashMap::new();
        env_params.insert(
            "temperature".to_string(),
            serde_json::Value::Number(serde_json::Number::from(22)),
        );
        env_params.insert(
            "lighting".to_string(),
            serde_json::Value::String("bright".to_string()),
        );

        let context_id = manager
            .create_context_state(
                EnvironmentType::Work,
                Some(location),
                env_params,
                user_state,
                None,
                None,
            )
            .unwrap();

        assert!(!context_id.is_empty());

        // 验证上下文已创建
        let context = manager.get_context_state(&context_id).unwrap();
        assert_eq!(context.environment_type, EnvironmentType::Work);
        assert!(context.location.is_some());
        assert_eq!(context.access_count, 1); // 获取时会增加访问次数
    }

    #[test]
    fn test_get_context_state() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();
        let env_params = HashMap::new();

        let context_id = manager
            .create_context_state(
                EnvironmentType::Digital,
                None,
                env_params,
                user_state,
                None,
                None,
            )
            .unwrap();

        let context = manager.get_context_state(&context_id).unwrap();
        assert_eq!(context.id, context_id);
        assert_eq!(context.environment_type, EnvironmentType::Digital);
        assert_eq!(context.access_count, 1);

        // 再次获取，访问次数应该增加
        let context2 = manager.get_context_state(&context_id).unwrap();
        assert_eq!(context2.access_count, 2);
    }

    #[test]
    fn test_update_context_state() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();
        let env_params = HashMap::new();

        let context_id = manager
            .create_context_state(
                EnvironmentType::Social,
                None,
                env_params,
                user_state,
                None,
                None,
            )
            .unwrap();

        // 更新用户状态
        let mut new_user_state = create_test_user_state();
        new_user_state.activity_state = ActivityState::Focused;
        new_user_state.attention_level = 0.9;

        let mut new_env_params = HashMap::new();
        new_env_params.insert(
            "noise_level".to_string(),
            serde_json::Value::String("low".to_string()),
        );

        manager
            .update_context_state(
                &context_id,
                Some(new_env_params),
                Some(new_user_state),
                None,
                None,
            )
            .unwrap();

        let updated_context = manager.get_context_state(&context_id).unwrap();
        assert_eq!(
            updated_context.user_state.activity_state,
            ActivityState::Focused
        );
        assert_eq!(updated_context.user_state.attention_level, 0.9);
        assert!(updated_context
            .environment_params
            .contains_key("noise_level"));
    }

    #[test]
    fn test_set_and_get_current_context() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();
        let env_params = HashMap::new();

        // 创建上下文
        let context_id = manager
            .create_context_state(
                EnvironmentType::Learning,
                None,
                env_params,
                user_state,
                None,
                None,
            )
            .unwrap();

        // 验证当前上下文已自动设置
        let current = manager.get_current_context().unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().id, context_id);

        // 创建另一个上下文
        let context_id2 = manager
            .create_context_state(
                EnvironmentType::Entertainment,
                None,
                HashMap::new(),
                create_test_user_state(),
                None,
                None,
            )
            .unwrap();

        // 手动设置当前上下文
        manager.set_current_context(&context_id2).unwrap();
        let current2 = manager.get_current_context().unwrap();
        assert!(current2.is_some());
        assert_eq!(current2.unwrap().id, context_id2);
    }

    #[test]
    fn test_search_contexts() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();
        let location = create_test_location();

        // 创建不同类型的上下文
        let _work_context = manager
            .create_context_state(
                EnvironmentType::Work,
                Some(location.clone()),
                HashMap::new(),
                user_state.clone(),
                None,
                None,
            )
            .unwrap();

        let _social_context = manager
            .create_context_state(
                EnvironmentType::Social,
                None,
                HashMap::new(),
                user_state.clone(),
                None,
                None,
            )
            .unwrap();

        // 搜索工作环境
        let work_results = manager
            .search_contexts(Some(EnvironmentType::Work), None, None, None, None, None)
            .unwrap();
        assert_eq!(work_results.len(), 1);
        assert_eq!(work_results[0].environment_type, EnvironmentType::Work);

        // 搜索所有上下文
        let all_results = manager
            .search_contexts(None, None, None, None, None, None)
            .unwrap();
        assert_eq!(all_results.len(), 2);
    }

    #[test]
    fn test_create_correlation() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();

        // 创建两个上下文
        let context1 = manager
            .create_context_state(
                EnvironmentType::Work,
                None,
                HashMap::new(),
                user_state.clone(),
                None,
                None,
            )
            .unwrap();

        let context2 = manager
            .create_context_state(
                EnvironmentType::Learning,
                None,
                HashMap::new(),
                user_state,
                None,
                None,
            )
            .unwrap();

        // 创建关联
        let correlation_id = manager
            .create_correlation(&context1, &context2, CorrelationType::Sequential, 0.8, 0.9)
            .unwrap();

        assert!(!correlation_id.is_empty());

        // 获取关联
        let correlations = manager.get_correlations_for_context(&context1).unwrap();
        assert_eq!(correlations.len(), 1);
        assert_eq!(
            correlations[0].correlation_type,
            CorrelationType::Sequential
        );
        assert_eq!(correlations[0].strength, 0.8);
        assert_eq!(correlations[0].confidence, 0.9);
    }

    #[test]
    fn test_delete_context_state() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();

        let context_id = manager
            .create_context_state(
                EnvironmentType::Other,
                None,
                HashMap::new(),
                user_state,
                None,
                None,
            )
            .unwrap();

        // 验证上下文存在
        assert!(manager.get_context_state(&context_id).is_ok());

        // 删除上下文
        manager.delete_context_state(&context_id).unwrap();

        // 验证上下文已删除
        assert!(manager.get_context_state(&context_id).is_err());
    }

    #[test]
    fn test_cleanup_expired_contexts() {
        let mut config = ContextualMemoryConfig::default();
        config.context_expiry_seconds = 1; // 1秒过期
        let manager = ContextualMemoryManager::new(config);
        let user_state = create_test_user_state();

        // 创建上下文
        let _context_id = manager
            .create_context_state(
                EnvironmentType::Physical,
                None,
                HashMap::new(),
                user_state,
                None,
                None,
            )
            .unwrap();

        // 等待过期
        std::thread::sleep(std::time::Duration::from_secs(2));

        // 清理过期上下文
        let removed_count = manager.cleanup_expired_contexts().unwrap();
        assert_eq!(removed_count, 1);

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_contexts, 0);
    }

    #[test]
    fn test_importance_score_calculation() {
        let manager = create_test_manager();

        // 测试不同环境类型的重要性评分
        let work_user_state = UserState {
            activity_state: ActivityState::Focused,
            attention_level: 0.9,
            mood_state: None,
            cognitive_load: 0.8,
            fatigue_level: 0.2,
            stress_level: 0.3,
        };

        let mut env_params = HashMap::new();
        env_params.insert(
            "priority".to_string(),
            serde_json::Value::String("high".to_string()),
        );

        let work_score = manager.calculate_importance_score(
            &EnvironmentType::Work,
            &work_user_state,
            &env_params,
        );

        let entertainment_score = manager.calculate_importance_score(
            &EnvironmentType::Entertainment,
            &work_user_state,
            &env_params,
        );

        // 工作环境应该比娱乐环境重要性更高
        assert!(work_score > entertainment_score);
        assert!((0.0..=1.0).contains(&work_score));
        assert!((0.0..=1.0).contains(&entertainment_score));
    }

    #[test]
    fn test_distance_calculation() {
        let manager = create_test_manager();

        // 测试距离计算（旧金山到洛杉矶大约559公里）
        let sf_lat = 37.7749;
        let sf_lon = -122.4194;
        let la_lat = 34.0522;
        let la_lon = -118.2437;

        let distance = manager.calculate_distance(sf_lat, sf_lon, la_lat, la_lon);

        // 允许一定误差范围
        assert!(distance > 500.0 && distance < 600.0);
    }

    #[test]
    fn test_temporal_info_generation() {
        let manager = create_test_manager();

        // 测试不同时间的时间段分类
        let morning_time = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(9, 0, 0)
            .unwrap()
            .and_utc();
        let afternoon_time = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(15, 0, 0)
            .unwrap()
            .and_utc();
        let evening_time = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(20, 0, 0)
            .unwrap()
            .and_utc();
        let night_time = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_utc();

        let morning_info = manager.generate_temporal_info(morning_time);
        let afternoon_info = manager.generate_temporal_info(afternoon_time);
        let evening_info = manager.generate_temporal_info(evening_time);
        let night_info = manager.generate_temporal_info(night_time);

        assert_eq!(morning_info.time_of_day, TimeOfDay::Morning);
        assert_eq!(afternoon_info.time_of_day, TimeOfDay::Afternoon);
        assert_eq!(evening_info.time_of_day, TimeOfDay::Evening);
        assert_eq!(night_info.time_of_day, TimeOfDay::Night);
    }

    #[test]
    fn test_stats_update() {
        let manager = create_test_manager();
        let user_state = create_test_user_state();

        // 创建不同类型的上下文
        let _work_context = manager
            .create_context_state(
                EnvironmentType::Work,
                None,
                HashMap::new(),
                user_state.clone(),
                None,
                None,
            )
            .unwrap();

        let _social_context = manager
            .create_context_state(
                EnvironmentType::Social,
                None,
                HashMap::new(),
                user_state,
                None,
                None,
            )
            .unwrap();

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_contexts, 2);
        assert_eq!(stats.active_contexts, 2);
        assert_eq!(stats.expired_contexts, 0);
        assert!(stats
            .contexts_by_environment
            .contains_key(&EnvironmentType::Work));
        assert!(stats
            .contexts_by_environment
            .contains_key(&EnvironmentType::Social));
        assert_eq!(stats.contexts_by_environment[&EnvironmentType::Work], 1);
        assert_eq!(stats.contexts_by_environment[&EnvironmentType::Social], 1);
    }
}
