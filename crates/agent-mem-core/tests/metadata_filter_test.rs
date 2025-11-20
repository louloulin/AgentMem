//! 元数据过滤系统测试

use agent_mem_core::search::metadata_filter::{
    FilterOperator, FilterValue, LogicalOperator, MetadataFilter, MetadataFilterSystem,
};
use std::collections::HashMap;

#[test]
fn test_has_advanced_operators() {
    // 测试简单过滤（无高级操作符）
    let mut simple_filters = HashMap::new();
    simple_filters.insert("category".to_string(), serde_json::json!("food"));
    assert!(!MetadataFilterSystem::has_advanced_operators(
        &simple_filters
    ));

    // 测试AND操作符
    let mut and_filters = HashMap::new();
    and_filters.insert("AND".to_string(), serde_json::json!([]));
    assert!(MetadataFilterSystem::has_advanced_operators(&and_filters));

    // 测试OR操作符
    let mut or_filters = HashMap::new();
    or_filters.insert("OR".to_string(), serde_json::json!([]));
    assert!(MetadataFilterSystem::has_advanced_operators(&or_filters));

    // 测试比较操作符
    let mut op_filters = HashMap::new();
    op_filters.insert("price".to_string(), serde_json::json!({"gt": 100}));
    assert!(MetadataFilterSystem::has_advanced_operators(&op_filters));
}

#[test]
fn test_process_metadata_filters() {
    // 测试简单条件
    let mut filters = HashMap::new();
    filters.insert("category".to_string(), serde_json::json!("food"));

    let result = MetadataFilterSystem::process_metadata_filters(&filters);
    assert!(result.is_ok());
    let processed = result.unwrap();
    match processed {
        LogicalOperator::Single(filter) => {
            assert_eq!(filter.field, "category");
            assert_eq!(filter.operator, FilterOperator::Eq);
            match &filter.value {
                FilterValue::String(s) => assert_eq!(s, "food"),
                _ => panic!("Expected String value"),
            }
        }
        _ => panic!("Expected Single filter"),
    }

    // 测试比较操作符
    let mut filters = HashMap::new();
    filters.insert("price".to_string(), serde_json::json!({"gt": 100}));

    let result = MetadataFilterSystem::process_metadata_filters(&filters);
    assert!(result.is_ok());
    let processed = result.unwrap();
    match processed {
        LogicalOperator::Single(filter) => {
            assert_eq!(filter.field, "price");
            assert_eq!(filter.operator, FilterOperator::Gt);
            match filter.value {
                FilterValue::Number(n) => assert_eq!(n, 100.0),
                FilterValue::Integer(i) => assert_eq!(i, 100),
                _ => panic!("Expected Number or Integer value"),
            }
        }
        _ => panic!("Expected Single filter"),
    }
}

#[test]
fn test_matches_filter() {
    // 测试等于操作符
    let filter = MetadataFilter {
        field: "category".to_string(),
        operator: FilterOperator::Eq,
        value: FilterValue::String("food".to_string()),
    };
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("food"));
    let logical = LogicalOperator::Single(filter);
    assert!(MetadataFilterSystem::matches(&logical, &metadata));

    // 测试不等于操作符
    let filter = MetadataFilter {
        field: "category".to_string(),
        operator: FilterOperator::Ne,
        value: FilterValue::String("drink".to_string()),
    };
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("food"));
    let logical = LogicalOperator::Single(filter);
    assert!(MetadataFilterSystem::matches(&logical, &metadata));

    // 测试大于操作符
    let filter = MetadataFilter {
        field: "price".to_string(),
        operator: FilterOperator::Gt,
        value: FilterValue::Number(100.0),
    };
    let mut metadata = HashMap::new();
    metadata.insert("price".to_string(), serde_json::json!(150));
    let logical = LogicalOperator::Single(filter);
    assert!(MetadataFilterSystem::matches(&logical, &metadata));

    // 测试小于操作符（应该失败）
    let filter = MetadataFilter {
        field: "price".to_string(),
        operator: FilterOperator::Lt,
        value: FilterValue::Number(100.0),
    };
    let mut metadata = HashMap::new();
    metadata.insert("price".to_string(), serde_json::json!(150));
    let logical = LogicalOperator::Single(filter);
    assert!(!MetadataFilterSystem::matches(&logical, &metadata));
}

#[test]
fn test_logical_operators() {
    // 测试AND操作符
    let and_filter = LogicalOperator::And(vec![
        MetadataFilter {
            field: "category".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("food".to_string()),
        },
        MetadataFilter {
            field: "price".to_string(),
            operator: FilterOperator::Gt,
            value: FilterValue::Number(100.0),
        },
    ]);
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("food"));
    metadata.insert("price".to_string(), serde_json::json!(150));
    assert!(MetadataFilterSystem::matches(&and_filter, &metadata));

    // 测试OR操作符
    let or_filter = LogicalOperator::Or(vec![
        MetadataFilter {
            field: "category".to_string(),
            operator: FilterOperator::Eq,
            value: FilterValue::String("drink".to_string()),
        },
        MetadataFilter {
            field: "price".to_string(),
            operator: FilterOperator::Gt,
            value: FilterValue::Number(100.0),
        },
    ]);
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("food"));
    metadata.insert("price".to_string(), serde_json::json!(150));
    assert!(MetadataFilterSystem::matches(&or_filter, &metadata));

    // 测试NOT操作符
    let not_filter = LogicalOperator::Not(Box::new(MetadataFilter {
        field: "category".to_string(),
        operator: FilterOperator::Eq,
        value: FilterValue::String("drink".to_string()),
    }));
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("food"));
    assert!(MetadataFilterSystem::matches(&not_filter, &metadata));
}
