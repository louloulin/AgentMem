//! Mem5 智能处理演示程序
//!
//! 展示 AgentMem Mem5 计划 Phase 4 的智能处理集成功能：
//! - 高级事实提取器 (AdvancedFactExtractor)
//! - 智能决策引擎 (EnhancedDecisionEngine)
//! - 冲突解决系统 (ConflictResolver)
//! - 重要性评估器 (ImportanceEvaluator)
//! - 完整智能处理流水线 (EnhancedIntelligentProcessor)

use agent_mem_core::Memory;
use agent_mem_intelligence::{
    AdvancedFactExtractor, EnhancedDecisionEngine, ConflictResolver, 
    EnhancedImportanceEvaluator, EnhancedIntelligentProcessor,
    ImportanceEvaluatorConfig, ConflictResolverConfig, ProcessorConfig
};
use agent_mem_llm::providers::deepseek::DeepSeekProvider;
use agent_mem_traits::{Message, MessageRole};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 启动 Mem5 智能处理演示程序");
    
    // 创建模拟的 LLM 提供者
    let llm = Arc::new(DeepSeekProvider::new("demo-key".to_string())?);
    
    // 演示各个组件
    demo_fact_extraction(&llm).await?;
    demo_importance_evaluation(&llm).await?;
    demo_conflict_resolution(&llm).await?;
    demo_enhanced_decision_engine(&llm).await?;
    demo_complete_pipeline(&llm).await?;
    
    info!("✅ Mem5 智能处理演示完成");
    Ok(())
}

/// 演示高级事实提取功能
async fn demo_fact_extraction(llm: &Arc<DeepSeekProvider>) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n📊 === 高级事实提取演示 ===");
    
    let fact_extractor = AdvancedFactExtractor::new(llm.clone());
    
    // 创建测试消息
    let messages = vec![
        Message {
            id: "msg1".to_string(),
            role: MessageRole::User,
            content: "我叫张三，今年30岁，在北京工作，喜欢编程和阅读。".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        },
        Message {
            id: "msg2".to_string(),
            role: MessageRole::User,
            content: "我的公司是科技创新有限公司，我们主要做人工智能产品。".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        },
    ];
    
    let start_time = Instant::now();
    
    // 提取结构化事实
    match fact_extractor.extract_structured_facts(&messages).await {
        Ok(facts) => {
            let duration = start_time.elapsed();
            info!("✅ 事实提取成功，耗时: {:?}", duration);
            info!("📋 提取到 {} 个结构化事实:", facts.len());
            
            for (i, fact) in facts.iter().enumerate() {
                info!("  {}. {} (置信度: {:.2})", i + 1, fact.description, fact.confidence);
                info!("     实体数量: {}, 关系数量: {}", fact.entities.len(), fact.relations.len());
            }
        }
        Err(e) => {
            warn!("⚠️ 事实提取失败: {}", e);
        }
    }
    
    Ok(())
}

/// 演示重要性评估功能
async fn demo_importance_evaluation(llm: &Arc<DeepSeekProvider>) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n⭐ === 重要性评估演示 ===");
    
    let config = ImportanceEvaluatorConfig::default();
    let evaluator = EnhancedImportanceEvaluator::new(llm.clone(), config);
    
    // 创建测试记忆
    let memory = Memory {
        id: "mem1".to_string(),
        content: "用户张三是一位30岁的软件工程师，在北京工作".to_string(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let context_memories = vec![
        Memory {
            id: "mem2".to_string(),
            content: "张三喜欢编程和阅读".to_string(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];
    
    let start_time = Instant::now();
    
    // 评估重要性
    match evaluator.evaluate_importance(&memory, &[], &context_memories).await {
        Ok(evaluation) => {
            let duration = start_time.elapsed();
            info!("✅ 重要性评估成功，耗时: {:?}", duration);
            info!("📊 重要性分数: {:.2}", evaluation.importance_score);
            info!("🎯 置信度: {:.2}", evaluation.confidence);
            info!("💭 评估原因: {}", evaluation.reasoning);
            
            let factors = &evaluation.factors;
            info!("📈 评估因子:");
            info!("  - 内容复杂度: {:.2}", factors.content_complexity);
            info!("  - 实体重要性: {:.2}", factors.entity_importance);
            info!("  - 关系重要性: {:.2}", factors.relation_importance);
            info!("  - 时间相关性: {:.2}", factors.temporal_relevance);
            info!("  - 用户交互: {:.2}", factors.user_interaction);
            info!("  - 上下文相关性: {:.2}", factors.contextual_relevance);
            info!("  - 情感强度: {:.2}", factors.emotional_intensity);
        }
        Err(e) => {
            warn!("⚠️ 重要性评估失败: {}", e);
        }
    }
    
    Ok(())
}

/// 演示冲突解决功能
async fn demo_conflict_resolution(llm: &Arc<DeepSeekProvider>) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n⚔️ === 冲突解决演示 ===");
    
    let config = ConflictResolverConfig::default();
    let resolver = ConflictResolver::new(llm.clone(), config);
    
    // 创建冲突的记忆
    let new_memories = vec![
        Memory {
            id: "new1".to_string(),
            content: "张三今年31岁".to_string(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];
    
    let existing_memories = vec![
        Memory {
            id: "old1".to_string(),
            content: "张三今年30岁".to_string(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now() - chrono::Duration::days(1),
            updated_at: chrono::Utc::now() - chrono::Duration::days(1),
        },
    ];
    
    let start_time = Instant::now();
    
    // 检测冲突
    match resolver.detect_conflicts(&new_memories, &existing_memories).await {
        Ok(conflicts) => {
            let duration = start_time.elapsed();
            info!("✅ 冲突检测成功，耗时: {:?}", duration);
            info!("⚔️ 检测到 {} 个冲突:", conflicts.len());
            
            for (i, conflict) in conflicts.iter().enumerate() {
                info!("  {}. 冲突类型: {:?}", i + 1, conflict.conflict_type);
                info!("     涉及记忆: {:?}", conflict.conflicting_memory_ids);
                info!("     严重程度: {:.2}", conflict.severity);
                info!("     推荐策略: {:?}", conflict.resolution_strategy);
                info!("     置信度: {:.2}", conflict.confidence);
            }
            
            // 解决冲突
            if !conflicts.is_empty() {
                let all_memories = [&new_memories[..], &existing_memories[..]].concat();
                match resolver.resolve_memory_conflicts(&conflicts, &all_memories).await {
                    Ok(resolutions) => {
                        info!("🔧 生成了 {} 个解决方案:", resolutions.len());
                        for (i, resolution) in resolutions.iter().enumerate() {
                            info!("  {}. 策略: {:?}", i + 1, resolution.strategy);
                            info!("     置信度: {:.2}", resolution.confidence);
                            info!("     原因: {}", resolution.reasoning);
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ 冲突解决失败: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            warn!("⚠️ 冲突检测失败: {}", e);
        }
    }
    
    Ok(())
}

/// 演示增强决策引擎功能
async fn demo_enhanced_decision_engine(llm: &Arc<DeepSeekProvider>) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n🧠 === 增强决策引擎演示 ===");
    
    let config = agent_mem_intelligence::decision_engine::DecisionEngineConfig::default();
    let decision_engine = EnhancedDecisionEngine::new(llm.clone(), config);
    
    // 创建决策上下文
    let context = agent_mem_intelligence::decision_engine::DecisionContext {
        new_facts: vec![], // 简化演示
        existing_memories: vec![],
        importance_evaluations: vec![],
        conflict_detections: vec![],
        user_preferences: HashMap::new(),
    };
    
    let start_time = Instant::now();
    
    // 制定决策
    match decision_engine.make_decisions(&context).await {
        Ok(decision) => {
            let duration = start_time.elapsed();
            info!("✅ 决策制定成功，耗时: {:?}", duration);
            info!("🎯 决策ID: {}", decision.decision_id);
            info!("📊 置信度: {:.2}", decision.confidence);
            info!("🔄 推荐操作数量: {}", decision.recommended_actions.len());
            info!("💭 决策原因: {}", decision.reasoning);
            info!("⚠️ 需要确认: {}", decision.requires_confirmation);
            
            let impact = &decision.expected_impact;
            info!("📈 预期影响:");
            info!("  - 影响记忆数量: {}", impact.affected_memory_count);
            info!("  - 性能影响: {:.2}", impact.performance_impact);
            info!("  - 存储影响: {:.2}", impact.storage_impact);
            info!("  - 用户体验影响: {:.2}", impact.user_experience_impact);
        }
        Err(e) => {
            warn!("⚠️ 决策制定失败: {}", e);
        }
    }
    
    Ok(())
}

/// 演示完整的智能处理流水线
async fn demo_complete_pipeline(llm: &Arc<DeepSeekProvider>) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n🔄 === 完整智能处理流水线演示 ===");
    
    let config = ProcessorConfig::default();
    let processor = EnhancedIntelligentProcessor::new(llm.clone(), config);
    
    // 创建测试消息
    let messages = vec![
        Message {
            id: "msg1".to_string(),
            role: MessageRole::User,
            content: "我叫李四，今年25岁，是一名数据科学家。".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        },
        Message {
            id: "msg2".to_string(),
            role: MessageRole::User,
            content: "我在上海工作，专注于机器学习和深度学习研究。".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        },
    ];
    
    let existing_memories = vec![
        Memory {
            id: "existing1".to_string(),
            content: "李四是一位年轻的研究员".to_string(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now() - chrono::Duration::hours(1),
            updated_at: chrono::Utc::now() - chrono::Duration::hours(1),
        },
    ];
    
    let start_time = Instant::now();
    
    // 执行完整的智能处理流水线
    match processor.process_memory_addition(&messages, &existing_memories).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            info!("✅ 智能处理流水线执行成功，耗时: {:?}", duration);
            info!("🆔 处理ID: {}", result.processing_id);
            info!("🎯 整体置信度: {:.2}", result.overall_confidence);
            
            let stats = &result.processing_stats;
            info!("📊 处理统计:");
            info!("  - 处理消息数: {}", stats.messages_processed);
            info!("  - 提取事实数: {}", stats.facts_extracted);
            info!("  - 检测冲突数: {}", stats.conflicts_detected);
            info!("  - 生成决策数: {}", stats.decisions_made);
            info!("  - 总处理时间: {}ms", stats.total_processing_time_ms);
            
            let metrics = &stats.performance_metrics;
            info!("⚡ 性能指标:");
            info!("  - 吞吐量: {:.2} 事实/秒", metrics.throughput_facts_per_second);
            info!("  - 平均响应时间: {:.2}ms", metrics.average_response_time_ms);
            
            info!("🔍 阶段耗时:");
            for (stage, time_ms) in &stats.stage_timings {
                info!("  - {}: {}ms", stage, time_ms);
            }
            
            info!("📋 结果摘要:");
            info!("  - 结构化事实: {} 个", result.structured_facts.len());
            info!("  - 重要性评估: {} 个", result.importance_evaluations.len());
            info!("  - 冲突检测: {} 个", result.conflict_detections.len());
            info!("  - 推荐操作: {} 个", result.decision_result.recommended_actions.len());
        }
        Err(e) => {
            error!("❌ 智能处理流水线执行失败: {}", e);
        }
    }
    
    Ok(())
}
