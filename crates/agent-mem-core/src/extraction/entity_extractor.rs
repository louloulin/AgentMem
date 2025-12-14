//! 实体提取器实现

use super::types::{Entity, EntityType};
use crate::CoreResult;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// 实体提取器 trait
#[async_trait]
pub trait EntityExtractor: Send + Sync {
    /// 从文本中提取实体
    async fn extract_entities(&self, text: &str) -> CoreResult<Vec<Entity>>;
}

/// 基于规则的实体提取器
///
/// 使用正则表达式和规则匹配来识别实体
pub struct RuleBasedExtractor {
    /// 人名模式
    person_patterns: Vec<Regex>,
    /// 组织模式
    org_patterns: Vec<Regex>,
    /// 地点模式
    location_patterns: Vec<Regex>,
    /// 邮箱模式
    email_pattern: Regex,
    /// 电话模式
    phone_pattern: Regex,
    /// URL 模式
    url_pattern: Regex,
    /// 日期模式
    date_patterns: Vec<Regex>,
    /// 金额模式
    money_patterns: Vec<Regex>,
}

impl RuleBasedExtractor {
    /// 创建新的基于规则的提取器
    pub fn new() -> Self {
        // Helper function to compile regex with fallback
        let compile_regex = |pattern: &str, fallback: &str, name: &str| -> Regex {
            Regex::new(pattern).unwrap_or_else(|e| {
                tracing::error!("Failed to compile {} regex pattern '{}': {e}, using fallback", name, pattern);
                Regex::new(fallback).unwrap_or_else(|_| {
                    tracing::error!("Failed to compile fallback regex pattern '{}', using empty pattern", fallback);
                    // Empty regex pattern "^$" is always valid, but handle error gracefully
                    Regex::new(r"^$").unwrap_or_else(|_| {
                        tracing::error!("Critical: Failed to compile empty regex pattern, this should never happen");
                        // This should never happen, but if it does, return a pattern that matches nothing
                        Regex::new(r"(?!)").unwrap_or_else(|_| {
                            // Last resort: this should never fail, but handle it anyway
                            panic!("Critical regex compilation failure: unable to create any regex pattern");
                        })
                    })
                })
            })
        };

        Self {
            person_patterns: vec![
                compile_regex(r"我叫(\p{Han}{2,4})", r"我叫(.+)", "person pattern 1"),
                compile_regex(r"我是(\p{Han}{2,4})", r"我是(.+)", "person pattern 2"),
                compile_regex(r"名字是(\p{Han}{2,4})", r"名字是(.+)", "person pattern 3"),
                compile_regex(r"My name is ([A-Z][a-z]+ [A-Z][a-z]+)", r"My name is (.+)", "person pattern 4"),
                compile_regex(r"I am ([A-Z][a-z]+ [A-Z][a-z]+)", r"I am (.+)", "person pattern 5"),
            ],
            org_patterns: vec![
                compile_regex(r"(\p{Han}+(?:公司|企业|集团|机构|组织|学校|大学|医院))", r"(.+公司)", "organization pattern 1"),
                compile_regex(r"((?:Google|Microsoft|Apple|Amazon|Facebook|Tesla|Alibaba|Tencent|Baidu)\s*(?:Inc\.|Corp\.|Company)?)", r"(Google|Microsoft|Apple)", "organization pattern 2"),
            ],
            location_patterns: vec![
                compile_regex(r"(\p{Han}+(?:市|省|区|县|镇|村|路|街|道))", r"(.+市)", "location pattern 1"),
                compile_regex(r"(北京|上海|广州|深圳|杭州|成都|重庆|武汉|西安|南京)", r"(北京|上海)", "location pattern 2"),
            ],
            email_pattern: compile_regex(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b", r"@.+", "email pattern"),
            phone_pattern: compile_regex(r"\b(?:\+?86)?1[3-9]\d{9}\b", r"\d{11}", "phone pattern"),
            url_pattern: compile_regex(r"https?://[^\s]+", r"http://.+", "url pattern"),
            date_patterns: vec![
                compile_regex(r"\d{4}年\d{1,2}月\d{1,2}日", r"\d{4}年\d{1,2}月", "date pattern 1"),
                compile_regex(r"\d{4}-\d{1,2}-\d{1,2}", r"\d{4}-\d{2}", "date pattern 2"),
                compile_regex(r"\d{1,2}/\d{1,2}/\d{4}", r"\d{2}/\d{4}", "date pattern 3"),
            ],
            money_patterns: vec![
                compile_regex(r"(\d+(?:\.\d+)?)\s*(?:元|美元|欧元|英镑|日元)", r"(\d+)\s*元", "money pattern 1"),
                compile_regex(r"(?:¥|$|€|£|¥)\s*(\d+(?:,\d{3})*(?:\.\d+)?)", r"¥\s*(\d+)", "money pattern 2"),
            ],
        }
    }

    /// 提取人物实体
    fn extract_persons(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut seen = HashMap::new();

        for pattern in &self.person_patterns {
            for cap in pattern.captures_iter(text) {
                if let Some(name_match) = cap.get(1) {
                    let name = name_match.as_str().to_string();

                    // 去重
                    if seen.contains_key(&name) {
                        continue;
                    }
                    seen.insert(name.clone(), true);

                    let entity = Entity::new(
                        format!("person_{}", Uuid::new_v4()),
                        name,
                        EntityType::Person,
                        0.85,
                    )
                    .with_span(name_match.start(), name_match.end());

                    entities.push(entity);
                }
            }
        }

        entities
    }

    /// 提取组织实体
    fn extract_organizations(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut seen = HashMap::new();

        for pattern in &self.org_patterns {
            for cap in pattern.captures_iter(text) {
                if let Some(org_match) = cap.get(1) {
                    let name = org_match.as_str().to_string();

                    // 去重
                    if seen.contains_key(&name) {
                        continue;
                    }
                    seen.insert(name.clone(), true);

                    let entity = Entity::new(
                        format!("org_{}", Uuid::new_v4()),
                        name,
                        EntityType::Organization,
                        0.80,
                    )
                    .with_span(org_match.start(), org_match.end());

                    entities.push(entity);
                }
            }
        }

        entities
    }

    /// 提取地点实体
    fn extract_locations(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut seen = HashMap::new();

        for pattern in &self.location_patterns {
            for cap in pattern.captures_iter(text) {
                if let Some(loc_match) = cap.get(1) {
                    let name = loc_match.as_str().to_string();

                    // 去重
                    if seen.contains_key(&name) {
                        continue;
                    }
                    seen.insert(name.clone(), true);

                    let entity = Entity::new(
                        format!("loc_{}", Uuid::new_v4()),
                        name,
                        EntityType::Location,
                        0.75,
                    )
                    .with_span(loc_match.start(), loc_match.end());

                    entities.push(entity);
                }
            }
        }

        entities
    }

    /// 提取邮箱实体
    fn extract_emails(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        for email_match in self.email_pattern.find_iter(text) {
            let email = email_match.as_str().to_string();
            let entity = Entity::new(
                format!("email_{}", Uuid::new_v4()),
                email,
                EntityType::Email,
                0.95,
            )
            .with_span(email_match.start(), email_match.end());

            entities.push(entity);
        }

        entities
    }

    /// 提取电话实体
    fn extract_phones(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        for phone_match in self.phone_pattern.find_iter(text) {
            let phone = phone_match.as_str().to_string();
            let entity = Entity::new(
                format!("phone_{}", Uuid::new_v4()),
                phone,
                EntityType::Phone,
                0.90,
            )
            .with_span(phone_match.start(), phone_match.end());

            entities.push(entity);
        }

        entities
    }

    /// 提取 URL 实体
    fn extract_urls(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        for url_match in self.url_pattern.find_iter(text) {
            let url = url_match.as_str().to_string();
            let entity = Entity::new(
                format!("url_{}", Uuid::new_v4()),
                url,
                EntityType::Url,
                0.95,
            )
            .with_span(url_match.start(), url_match.end());

            entities.push(entity);
        }

        entities
    }

    /// 提取日期实体
    fn extract_dates(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut seen = HashMap::new();

        for pattern in &self.date_patterns {
            for date_match in pattern.find_iter(text) {
                let date = date_match.as_str().to_string();

                // 去重
                if seen.contains_key(&date) {
                    continue;
                }
                seen.insert(date.clone(), true);

                let entity = Entity::new(
                    format!("date_{}", Uuid::new_v4()),
                    date,
                    EntityType::Date,
                    0.85,
                )
                .with_span(date_match.start(), date_match.end());

                entities.push(entity);
            }
        }

        entities
    }

    /// 提取金额实体
    fn extract_money(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        for pattern in &self.money_patterns {
            for cap in pattern.captures_iter(text) {
                if let Some(money_match) = cap.get(0) {
                    let money = money_match.as_str().to_string();
                    let entity = Entity::new(
                        format!("money_{}", Uuid::new_v4()),
                        money,
                        EntityType::Money,
                        0.90,
                    )
                    .with_span(money_match.start(), money_match.end());

                    entities.push(entity);
                }
            }
        }

        entities
    }
}

#[async_trait]
impl EntityExtractor for RuleBasedExtractor {
    async fn extract_entities(&self, text: &str) -> CoreResult<Vec<Entity>> {
        info!("开始提取实体，文本长度: {}", text.len());

        let mut all_entities = Vec::new();

        // 提取各类实体
        all_entities.extend(self.extract_persons(text));
        all_entities.extend(self.extract_organizations(text));
        all_entities.extend(self.extract_locations(text));
        all_entities.extend(self.extract_emails(text));
        all_entities.extend(self.extract_phones(text));
        all_entities.extend(self.extract_urls(text));
        all_entities.extend(self.extract_dates(text));
        all_entities.extend(self.extract_money(text));

        debug!("提取到 {} 个实体", all_entities.len());

        Ok(all_entities)
    }
}

impl Default for RuleBasedExtractor {
    fn default() -> Self {
        Self::new()
    }
}
