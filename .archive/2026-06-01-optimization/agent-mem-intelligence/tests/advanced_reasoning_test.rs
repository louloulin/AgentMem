//! 高级推理功能测试

use agent_mem_intelligence::reasoning::{AdvancedReasoner, CausalRelationType, MemoryData};
use chrono::{Duration, Utc};

/// 创建测试记忆
fn create_test_memory(id: &str, content: &str, hours_ago: i64) -> MemoryData {
    MemoryData {
        id: id.to_string(),
        content: content.to_string(),
        created_at: Utc::now() - Duration::hours(hours_ago),
        embedding: Some(vec![0.5, 0.5, 0.5]),
    }
}

/// 创建带嵌入的测试记忆
fn create_memory_with_embedding(
    id: &str,
    content: &str,
    hours_ago: i64,
    embedding: Vec<f32>,
) -> MemoryData {
    MemoryData {
        id: id.to_string(),
        content: content.to_string(),
        created_at: Utc::now() - Duration::hours(hours_ago),
        embedding: Some(embedding),
    }
}

#[test]
fn test_advanced_reasoner_creation() {
    let _reasoner = AdvancedReasoner::default();
    // 推理器创建成功
}

#[test]
fn test_advanced_reasoner_custom_config() {
    let _reasoner = AdvancedReasoner::new(3, 0.7, 5);
    // 推理器创建成功
}

#[test]
fn test_multi_hop_causal_reasoning() {
    let reasoner = AdvancedReasoner::default();

    // 创建因果链: A -> B -> C
    let memory_a = create_memory_with_embedding(
        "A",
        "started learning machine learning",
        10,
        vec![1.0, 0.0, 0.0],
    );
    let memory_b = create_memory_with_embedding(
        "B",
        "completed machine learning course",
        5,
        vec![0.9, 0.1, 0.0],
    );
    let memory_c = create_memory_with_embedding(
        "C",
        "got machine learning job offer",
        1,
        vec![0.8, 0.2, 0.0],
    );

    let all_memories = vec![memory_a.clone(), memory_b.clone(), memory_c.clone()];

    let results = reasoner
        .multi_hop_causal_reasoning(&memory_a, &memory_c, &all_memories)
        .unwrap();

    // 应该找到至少一条因果链
    assert!(!results.is_empty());

    for result in &results {
        // 检查推理链
        assert!(!result.reasoning_chain.is_empty());
        assert!(result.depth > 0);
        assert!(result.overall_confidence > 0.0);
        assert!(result.overall_confidence <= 1.0);
        assert!(!result.explanation.is_empty());

        // 检查因果步骤
        for step in &result.reasoning_chain {
            assert!(!step.cause_id.is_empty());
            assert!(!step.effect_id.is_empty());
            assert!(step.confidence > 0.0);
            assert!(step.confidence <= 1.0);
            assert!(!step.evidence.is_empty());
        }
    }
}

#[test]
fn test_multi_hop_causal_reasoning_no_path() {
    let reasoner = AdvancedReasoner::default();

    // 创建不相关的记忆
    let memory_a = create_memory_with_embedding("A", "learning cooking", 10, vec![1.0, 0.0, 0.0]);
    let memory_b = create_memory_with_embedding("B", "studying physics", 5, vec![0.0, 1.0, 0.0]);

    let all_memories = vec![memory_a.clone(), memory_b.clone()];

    let results = reasoner
        .multi_hop_causal_reasoning(&memory_a, &memory_b, &all_memories)
        .unwrap();

    // 可能找不到高置信度的因果链
    if !results.is_empty() {
        for result in &results {
            // 如果找到了，置信度应该较低
            assert!(result.overall_confidence < 0.8);
        }
    }
}

#[test]
fn test_counterfactual_reasoning() {
    let reasoner = AdvancedReasoner::default();

    let target_memory = create_memory_with_embedding(
        "target",
        "decided to study computer science",
        10,
        vec![1.0, 0.0, 0.0],
    );

    let dependent_memory = create_memory_with_embedding(
        "dependent",
        "learned programming and got software job",
        5,
        vec![0.9, 0.1, 0.0],
    );

    let all_memories = vec![target_memory.clone(), dependent_memory.clone()];

    let result = reasoner
        .counterfactual_reasoning(
            &target_memory,
            "decided to study medicine instead",
            &all_memories,
        )
        .unwrap();

    // 检查反事实推理结果
    assert_eq!(
        result.original_scenario,
        "decided to study computer science"
    );
    assert_eq!(
        result.counterfactual_hypothesis,
        "decided to study medicine instead"
    );
    assert!(!result.predicted_outcome.is_empty());
    assert!(result.confidence > 0.0);
    assert!(result.confidence <= 1.0);
    assert!(!result.explanation.is_empty());
}

#[test]
fn test_counterfactual_reasoning_no_dependencies() {
    let reasoner = AdvancedReasoner::default();

    let target_memory = create_test_memory("target", "had breakfast", 10);
    let unrelated_memory = create_test_memory("unrelated", "went to gym", 5);

    let all_memories = vec![target_memory.clone(), unrelated_memory];

    let result = reasoner
        .counterfactual_reasoning(&target_memory, "skipped breakfast", &all_memories)
        .unwrap();

    // 应该没有受影响的记忆
    assert!(result.affected_memory_ids.is_empty() || result.affected_memory_ids.len() <= 1);
}

#[test]
fn test_analogical_reasoning() {
    let reasoner = AdvancedReasoner::default();

    // 源领域：学习编程
    let source_memories = vec![
        create_test_memory("s1", "learning programming basics", 10),
        create_test_memory("s2", "practicing coding exercises", 8),
        create_test_memory("s3", "building projects", 6),
    ];

    // 目标领域：学习音乐
    let target_memories = vec![
        create_test_memory("t1", "learning music theory basics", 5),
        create_test_memory("t2", "practicing musical scales", 3),
        create_test_memory("t3", "composing songs", 1),
    ];

    let result = reasoner
        .analogical_reasoning(&source_memories, &target_memories)
        .unwrap();

    // 检查类比结果
    assert_eq!(result.source_domain.name, "source");
    assert_eq!(result.target_domain.name, "target");
    assert_eq!(result.source_domain.memory_ids.len(), 3);
    assert_eq!(result.target_domain.memory_ids.len(), 3);
    assert!(!result.mappings.is_empty());
    assert!(result.analogy_strength >= 0.0);
    assert!(result.analogy_strength <= 1.0);
    assert!(!result.conclusion.is_empty());
}

#[test]
fn test_analogical_reasoning_weak_analogy() {
    let reasoner = AdvancedReasoner::default();

    // 源领域：编程
    let source_memories = vec![
        create_test_memory("s1", "writing code", 10),
        create_test_memory("s2", "debugging programs", 8),
    ];

    // 目标领域：烹饪（弱类比）
    let target_memories = vec![
        create_test_memory("t1", "cooking dinner", 5),
        create_test_memory("t2", "baking cake", 3),
    ];

    let result = reasoner
        .analogical_reasoning(&source_memories, &target_memories)
        .unwrap();

    // 类比强度应该较低
    assert!(result.analogy_strength < 0.7);
}

#[test]
fn test_causal_step_structure() {
    // 测试因果步骤的结构
    let reasoner = AdvancedReasoner::default();

    let memory_a = create_memory_with_embedding("A", "started learning", 10, vec![1.0, 0.0, 0.0]);
    let memory_b =
        create_memory_with_embedding("B", "completed learning course", 5, vec![0.9, 0.1, 0.0]);

    let all_memories = vec![memory_a.clone(), memory_b.clone()];

    let results = reasoner
        .multi_hop_causal_reasoning(&memory_a, &memory_b, &all_memories)
        .unwrap();

    if !results.is_empty() {
        let first_result = &results[0];
        for step in &first_result.reasoning_chain {
            // 验证因果关系类型是有效的
            match step.relation_type {
                CausalRelationType::Direct
                | CausalRelationType::Indirect
                | CausalRelationType::Necessary
                | CausalRelationType::Sufficient
                | CausalRelationType::Facilitating
                | CausalRelationType::Inhibiting => {
                    // 有效的类型
                }
            }
        }
    }
}

#[test]
fn test_analogical_domain_structure() {
    let reasoner = AdvancedReasoner::default();

    let source_memories = vec![
        create_test_memory("s1", "learning programming basics", 10),
        create_test_memory("s2", "practicing coding", 8),
    ];

    let target_memories = vec![
        create_test_memory("t1", "learning music theory", 5),
        create_test_memory("t2", "practicing piano", 3),
    ];

    let result = reasoner
        .analogical_reasoning(&source_memories, &target_memories)
        .unwrap();

    // 验证领域结构
    assert!(!result.source_domain.memory_ids.is_empty());
    assert!(!result.target_domain.memory_ids.is_empty());
    assert!(!result.source_domain.features.is_empty());
    assert!(!result.target_domain.features.is_empty());
}

#[test]
fn test_mapping_types() {
    let reasoner = AdvancedReasoner::default();

    let source_memories = vec![
        create_test_memory("s1", "entity with attributes", 10),
        create_test_memory("s2", "related entity", 8),
    ];

    let target_memories = vec![
        create_test_memory("t1", "similar entity with attributes", 5),
        create_test_memory("t2", "related similar entity", 3),
    ];

    let result = reasoner
        .analogical_reasoning(&source_memories, &target_memories)
        .unwrap();

    // 验证映射类型
    for mapping in &result.mappings {
        match mapping.mapping_type {
            agent_mem_intelligence::reasoning::MappingType::Entity
            | agent_mem_intelligence::reasoning::MappingType::Relation
            | agent_mem_intelligence::reasoning::MappingType::Attribute
            | agent_mem_intelligence::reasoning::MappingType::Structural => {
                // 有效的映射类型
            }
        }
    }
}
