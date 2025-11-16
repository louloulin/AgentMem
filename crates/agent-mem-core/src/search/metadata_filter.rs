//! 元数据过滤系统
//!
//! 实现Mem0级别的高级元数据过滤，支持逻辑操作符和比较操作符

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 过滤操作符
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterOperator {
    /// 等于
    #[serde(rename = "eq")]
    Eq,
    /// 不等于
    #[serde(rename = "ne")]
    Ne,
    /// 大于
    #[serde(rename = "gt")]
    Gt,
    /// 大于等于
    #[serde(rename = "gte")]
    Gte,
    /// 小于
    #[serde(rename = "lt")]
    Lt,
    /// 小于等于
    #[serde(rename = "lte")]
    Lte,
    /// 在列表中
    #[serde(rename = "in")]
    In,
    /// 不在列表中
    #[serde(rename = "nin")]
    Nin,
    /// 包含文本
    #[serde(rename = "contains")]
    Contains,
    /// 不区分大小写包含
    #[serde(rename = "icontains")]
    IContains,
}

/// 过滤值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    List(Vec<FilterValue>),
    Null,
}

/// 元数据过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFilter {
    /// 字段名
    pub field: String,
    /// 操作符
    pub operator: FilterOperator,
    /// 值
    pub value: FilterValue,
}

/// 逻辑操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogicalOperator {
    /// 逻辑与
    And(Vec<MetadataFilter>),
    /// 逻辑或
    Or(Vec<MetadataFilter>),
    /// 逻辑非
    Not(Box<MetadataFilter>),
    /// 单个过滤条件
    Single(MetadataFilter),
}

/// 元数据过滤系统
pub struct MetadataFilterSystem;

impl MetadataFilterSystem {
    /// 检测是否包含高级操作符
    ///
    /// 检查过滤条件中是否包含逻辑操作符（AND, OR, NOT）或比较操作符
    pub fn has_advanced_operators(filters: &HashMap<String, serde_json::Value>) -> bool {
        for (key, _value) in filters {
            // 检查逻辑操作符
            if key == "AND" || key == "OR" || key == "NOT" {
                return true;
            }

            // 检查值是否为对象（可能包含操作符）
            // 例如: {"category": {"eq": "food"}}
            if let Some(obj) = _value.as_object() {
                // 检查是否包含操作符键
                for op_key in obj.keys() {
                    if matches!(
                        op_key.as_str(),
                        "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "in" | "nin" | "contains" | "icontains"
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 处理元数据过滤器
    ///
    /// 将HashMap格式的过滤器转换为LogicalOperator结构
    pub fn process_metadata_filters(
        filters: &HashMap<String, serde_json::Value>,
    ) -> Result<LogicalOperator, String> {
        // 检查是否有逻辑操作符
        if let Some(and_value) = filters.get("AND") {
            if let Some(and_array) = and_value.as_array() {
                let conditions: Result<Vec<MetadataFilter>, String> = and_array
                    .iter()
                    .map(|v| Self::parse_filter_condition(v))
                    .collect();
                return Ok(LogicalOperator::And(conditions?));
            }
        }

        if let Some(or_value) = filters.get("OR") {
            if let Some(or_array) = or_value.as_array() {
                let conditions: Result<Vec<MetadataFilter>, String> = or_array
                    .iter()
                    .map(|v| Self::parse_filter_condition(v))
                    .collect();
                return Ok(LogicalOperator::Or(conditions?));
            }
        }

        if let Some(not_value) = filters.get("NOT") {
            let condition = Self::parse_filter_condition(not_value)?;
            return Ok(LogicalOperator::Not(Box::new(condition)));
        }

        // 处理单个条件或多个条件（默认为AND）
        let mut conditions = Vec::new();
        for (key, value) in filters {
            if key != "AND" && key != "OR" && key != "NOT" {
                conditions.push(Self::parse_field_filter(key, value)?);
            }
        }

        if conditions.len() == 1 {
            Ok(LogicalOperator::Single(conditions.remove(0)))
        } else if conditions.is_empty() {
            Err("No filter conditions found".to_string())
        } else {
            Ok(LogicalOperator::And(conditions))
        }
    }

    /// 解析过滤条件
    fn parse_filter_condition(value: &serde_json::Value) -> Result<MetadataFilter, String> {
        if let Some(obj) = value.as_object() {
            // 格式: {"field": {"operator": "value"}}
            if obj.len() == 1 {
                let (field, filter_obj) = obj.iter().next().unwrap();
                if let Some(filter_map) = filter_obj.as_object() {
                    return Self::parse_field_filter(field, &serde_json::Value::Object(
                        filter_map.clone()
                    ));
                }
            }
        }
        Err("Invalid filter condition format".to_string())
    }

    /// 解析字段过滤器
    fn parse_field_filter(
        field: &str,
        value: &serde_json::Value,
    ) -> Result<MetadataFilter, String> {
        // 如果值是对象，包含操作符
        if let Some(obj) = value.as_object() {
            // 查找操作符
            for (op_str, val) in obj {
                let operator = match op_str.as_str() {
                    "eq" => FilterOperator::Eq,
                    "ne" => FilterOperator::Ne,
                    "gt" => FilterOperator::Gt,
                    "gte" => FilterOperator::Gte,
                    "lt" => FilterOperator::Lt,
                    "lte" => FilterOperator::Lte,
                    "in" => FilterOperator::In,
                    "nin" => FilterOperator::Nin,
                    "contains" => FilterOperator::Contains,
                    "icontains" => FilterOperator::IContains,
                    _ => return Err(format!("Unknown operator: {}", op_str)),
                };

                let filter_value = Self::parse_filter_value(val)?;
                return Ok(MetadataFilter {
                    field: field.to_string(),
                    operator,
                    value: filter_value,
                });
            }
        }

        // 如果值是简单值，默认为eq操作符
        let filter_value = Self::parse_filter_value(value)?;
        Ok(MetadataFilter {
            field: field.to_string(),
            operator: FilterOperator::Eq,
            value: filter_value,
        })
    }

    /// 解析过滤值
    fn parse_filter_value(value: &serde_json::Value) -> Result<FilterValue, String> {
        match value {
            serde_json::Value::String(s) => Ok(FilterValue::String(s.clone())),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(FilterValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(FilterValue::Number(f))
                } else {
                    Err("Invalid number format".to_string())
                }
            }
            serde_json::Value::Bool(b) => Ok(FilterValue::Boolean(*b)),
            serde_json::Value::Array(arr) => {
                let list: Result<Vec<FilterValue>, String> =
                    arr.iter().map(|v| Self::parse_filter_value(v)).collect();
                Ok(FilterValue::List(list?))
            }
            serde_json::Value::Null => Ok(FilterValue::Null),
            _ => Err("Unsupported filter value type".to_string()),
        }
    }

    /// 检查记忆是否匹配过滤条件
    ///
    /// 根据元数据过滤条件检查记忆是否匹配
    pub fn matches(
        filter: &LogicalOperator,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> bool {
        match filter {
            LogicalOperator::And(conditions) => {
                conditions.iter().all(|c| Self::matches_single(c, metadata))
            }
            LogicalOperator::Or(conditions) => {
                conditions.iter().any(|c| Self::matches_single(c, metadata))
            }
            LogicalOperator::Not(condition) => !Self::matches_single(condition, metadata),
            LogicalOperator::Single(condition) => Self::matches_single(condition, metadata),
        }
    }

    /// 检查单个过滤条件是否匹配
    fn matches_single(
        filter: &MetadataFilter,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> bool {
        let field_value = metadata.get(&filter.field);

        match &filter.operator {
            FilterOperator::Eq => {
                Self::compare_values(field_value, &filter.value, |a, b| a == b)
            }
            FilterOperator::Ne => {
                !Self::compare_values(field_value, &filter.value, |a, b| a == b)
            }
            FilterOperator::Gt => {
                Self::compare_numeric(field_value, &filter.value, |a, b| a > b)
            }
            FilterOperator::Gte => {
                Self::compare_numeric(field_value, &filter.value, |a, b| a >= b)
            }
            FilterOperator::Lt => {
                Self::compare_numeric(field_value, &filter.value, |a, b| a < b)
            }
            FilterOperator::Lte => {
                Self::compare_numeric(field_value, &filter.value, |a, b| a <= b)
            }
            FilterOperator::In => {
                if let FilterValue::List(list) = &filter.value {
                    list.iter().any(|v| Self::compare_values(field_value, v, |a, b| a == b))
                } else {
                    false
                }
            }
            FilterOperator::Nin => {
                if let FilterValue::List(list) = &filter.value {
                    !list.iter().any(|v| Self::compare_values(field_value, v, |a, b| a == b))
                } else {
                    true
                }
            }
            FilterOperator::Contains => {
                Self::compare_string(field_value, &filter.value, |a, b| a.contains(b))
            }
            FilterOperator::IContains => {
                Self::compare_string(field_value, &filter.value, |a, b| {
                    a.to_lowercase().contains(&b.to_lowercase())
                })
            }
        }
    }

    /// 比较值
    fn compare_values<F>(field_value: Option<&serde_json::Value>, filter_value: &FilterValue, cmp: F) -> bool
    where
        F: Fn(&serde_json::Value, &FilterValue) -> bool,
    {
        if let Some(fv) = field_value {
            cmp(fv, filter_value)
        } else {
            false
        }
    }

    /// 比较数值
    fn compare_numeric<F>(field_value: Option<&serde_json::Value>, filter_value: &FilterValue, cmp: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let field_num = field_value.and_then(|v| v.as_f64());
        let filter_num = match filter_value {
            FilterValue::Number(n) => Some(*n),
            FilterValue::Integer(i) => Some(*i as f64),
            _ => None,
        };

        if let (Some(fn_val), Some(ft_val)) = (field_num, filter_num) {
            cmp(fn_val, ft_val)
        } else {
            false
        }
    }

    /// 比较字符串
    fn compare_string<F>(field_value: Option<&serde_json::Value>, filter_value: &FilterValue, cmp: F) -> bool
    where
        F: Fn(&str, &str) -> bool,
    {
        let field_str = field_value.and_then(|v| v.as_str());
        let filter_str = match filter_value {
            FilterValue::String(s) => Some(s.as_str()),
            _ => None,
        };

        if let (Some(fs_val), Some(ft_val)) = (field_str, filter_str) {
            cmp(fs_val, ft_val)
        } else {
            false
        }
    }
}

// 为serde_json::Value实现比较
impl PartialEq<FilterValue> for serde_json::Value {
    fn eq(&self, other: &FilterValue) -> bool {
        match (self, other) {
            (serde_json::Value::String(s1), FilterValue::String(s2)) => s1 == s2,
            (serde_json::Value::Number(n1), FilterValue::Number(n2)) => {
                n1.as_f64().map(|f| f == *n2).unwrap_or(false)
            }
            (serde_json::Value::Number(n1), FilterValue::Integer(i2)) => {
                n1.as_i64().map(|i| i == *i2).unwrap_or(false)
            }
            (serde_json::Value::Bool(b1), FilterValue::Boolean(b2)) => b1 == b2,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_advanced_operators() {
        let mut filters = HashMap::new();
        filters.insert("category".to_string(), serde_json::json!("food"));
        assert!(!MetadataFilterSystem::has_advanced_operators(&filters));

        filters.insert("AND".to_string(), serde_json::json!([]));
        assert!(MetadataFilterSystem::has_advanced_operators(&filters));

        filters.clear();
        filters.insert("category".to_string(), serde_json::json!({"eq": "food"}));
        assert!(MetadataFilterSystem::has_advanced_operators(&filters));
    }

    #[test]
    fn test_process_metadata_filters() {
        let mut filters = HashMap::new();
        filters.insert("category".to_string(), serde_json::json!("food"));
        
        let result = MetadataFilterSystem::process_metadata_filters(&filters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_matches() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), serde_json::json!("food"));
        metadata.insert("importance".to_string(), serde_json::json!("high"));

        let filter = MetadataFilter {
            field: "category".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("food".to_string()),
        };

        let logical = LogicalOperator::Single(filter);
        assert!(MetadataFilterSystem::matches(&logical, &metadata));
    }
}

