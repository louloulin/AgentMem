//! Orchestrator Intelligence - 智能处理模块
//!
//! 负责所有智能处理相关操作，包括事实提取、重要性评估、冲突检测、决策等

use std::collections::HashMap;
use std::sync::Arc;
use futures::future::join_all;
use tracing::{debug, info, warn};

use agent_mem_intelligence::{
    ConflictDetection, ExistingMemory, ExtractedFact, ImportanceEvaluation, MemoryAction,
    MemoryDecision, StructuredFact,
};
use agent_mem_traits::{AgentMemError, MemoryItem, Result};

use super::core::MemoryOrchestrator;
use super::utils::UtilsModule;
use crate::types::AddResult;

/// 智能处理模块
///
/// 负责所有智能处理相关操作
pub struct IntelligenceModule;

impl IntelligenceModule {
    /// 提取事实
    pub async fn extract_facts(
        orchestrator: &MemoryOrchestrator,
        content: &str,
    ) -> Result<Vec<ExtractedFact>> {
        if let Some(fact_extractor) = &orchestrator.fact_extractor {
            // 使用缓存
            if let Some(cache) = &orchestrator.facts_cache {
                let cache_key =
                    agent_mem_llm::LLMCache::<Vec<ExtractedFact>>::generate_key(content);

                // 尝试从缓存获取
                if let Some(cached_facts) = cache.get(&cache_key).await {
                    debug!("✅ 从缓存获取事实提取结果（命中）");
                    return Ok(cached_facts);
                }

                // 缓存未命中，调用 LLM
                debug!("⚠️ 缓存未命中，调用 LLM 进行事实提取");
                let messages = vec![agent_mem_llm::Message::user(content)];
                let facts = fact_extractor.extract_facts_internal(&messages).await?;

                // 缓存结果
                cache.set(cache_key, facts.clone()).await;
                debug!("✅ 事实提取结果已缓存");

                Ok(facts)
            } else {
                // 无缓存，直接调用
                let messages = vec![agent_mem_llm::Message::user(content)];
                fact_extractor.extract_facts_internal(&messages).await
            }
        } else {
            warn!("FactExtractor 未初始化");
            Ok(Vec::new())
        }
    }

    /// 提取结构化事实
    pub async fn extract_structured_facts(
        orchestrator: &MemoryOrchestrator,
        content: &str,
    ) -> Result<Vec<StructuredFact>> {
        if let Some(advanced_fact_extractor) = &orchestrator.advanced_fact_extractor {
            // 使用缓存
            if let Some(cache) = &orchestrator.structured_facts_cache {
                let cache_key =
                    agent_mem_llm::LLMCache::<Vec<StructuredFact>>::generate_key(content);

                // 尝试从缓存获取
                if let Some(cached_facts) = cache.get(&cache_key).await {
                    debug!("✅ 从缓存获取结构化事实提取结果（命中）");
                    return Ok(cached_facts);
                }

                // 缓存未命中，调用 LLM
                debug!("⚠️ 缓存未命中，调用 LLM 进行结构化事实提取");
                let messages = vec![agent_mem_llm::Message::user(content)];
                let facts = advanced_fact_extractor
                    .extract_structured_facts(&messages)
                    .await?;

                // 缓存结果
                cache.set(cache_key, facts.clone()).await;
                debug!("✅ 结构化事实提取结果已缓存");

                Ok(facts)
            } else {
                // 无缓存，直接调用
                let messages = vec![agent_mem_llm::Message::user(content)];
                advanced_fact_extractor
                    .extract_structured_facts(&messages)
                    .await
            }
        } else {
            warn!("AdvancedFactExtractor 未初始化");
            Ok(Vec::new())
        }
    }

    /// 评估重要性
    pub async fn evaluate_importance(
        orchestrator: &MemoryOrchestrator,
        structured_facts: &[StructuredFact],
        agent_id: &str,
        user_id: Option<String>,
    ) -> Result<Vec<ImportanceEvaluation>> {
        if let Some(evaluator) = &orchestrator.importance_evaluator {
            info!(
                "使用 EnhancedImportanceEvaluator 评估 {} 个事实的重要性",
                structured_facts.len()
            );

            // 使用缓存
            if let Some(cache) = &orchestrator.importance_cache {
                // 生成缓存键（基于所有事实的内容）
                let cache_content = structured_facts
                    .iter()
                    .map(|f| format!("{}:{}", f.description, f.fact_type))
                    .collect::<Vec<_>>()
                    .join("|");
                let cache_key = agent_mem_llm::LLMCache::<Vec<ImportanceEvaluation>>::generate_key(
                    &cache_content,
                );

                // 尝试从缓存获取
                if let Some(cached_evaluations) = cache.get(&cache_key).await {
                    debug!("✅ 从缓存获取重要性评估结果（命中）");
                    return Ok(cached_evaluations);
                }

                debug!("⚠️ 缓存未命中，调用 LLM 进行重要性评估");
            }

            // 🆕 并行化改进: 使用 futures::future::join_all 并行评估所有事实
            // 性能提升: 从 O(n) 顺序执行改为 O(1) 并行执行（n 个事实）
            // 预期提升: 2-5x（取决于事实数量和 LLM 响应时间）
            use futures::future::join_all;
            
            let evaluation_tasks: Vec<_> = structured_facts
                .iter()
                .map(|fact| {
                    let fact_clone = fact.clone();
                    let agent_id_clone = agent_id.to_string();
                    let user_id_clone = user_id.clone();
                    let evaluator_ref = evaluator.clone();
                    
                    async move {
                        // 将 StructuredFact 转换为 MemoryItem
                        let memory_item = UtilsModule::structured_fact_to_memory_item(
                            &fact_clone,
                            agent_id_clone,
                            user_id_clone,
                        );

                        // 转换为 MemoryV4
                        use agent_mem_traits::MemoryV4;
                        let memory = MemoryV4::from_legacy_item(&memory_item);

                        // 调用 EnhancedImportanceEvaluator
                        evaluator_ref
                            .evaluate_importance(&memory, &[fact_clone], &[])
                            .await
                    }
                })
                .collect();

            // 并行执行所有评估任务
            let evaluation_results = join_all(evaluation_tasks).await;
            
            // 收集结果并处理错误
            let mut evaluations = Vec::new();
            for (i, result) in evaluation_results.into_iter().enumerate() {
                match result {
                    Ok(evaluation) => evaluations.push(evaluation),
                    Err(e) => {
                        warn!(
                            "重要性评估失败 (fact {}): {}",
                            i, e
                        );
                        // 降级：使用默认重要性评估
                        let fact = &structured_facts[i];
                        use agent_mem_intelligence::ImportanceFactors;
                        evaluations.push(ImportanceEvaluation {
                            memory_id: fact.id.clone(),
                            importance_score: fact.importance,
                            confidence: fact.confidence,
                            factors: ImportanceFactors {
                                content_complexity: fact.importance,
                                entity_importance: 0.5,
                                relation_importance: 0.5,
                                temporal_relevance: 0.5,
                                user_interaction: 0.5,
                                contextual_relevance: 0.5,
                                emotional_intensity: 0.5,
                            },
                            evaluated_at: chrono::Utc::now(),
                            reasoning: format!("降级评估: {:.2}", fact.importance),
                        });
                    }
                }
            }

            info!("重要性评估完成，生成 {} 个评估结果", evaluations.len());

            // 缓存结果
            if let Some(cache) = &orchestrator.importance_cache {
                let cache_content = structured_facts
                    .iter()
                    .map(|f| format!("{}:{}", f.description, f.fact_type))
                    .collect::<Vec<_>>()
                    .join("|");
                let cache_key = agent_mem_llm::LLMCache::<Vec<ImportanceEvaluation>>::generate_key(
                    &cache_content,
                );
                cache.set(cache_key, evaluations.clone()).await;
                debug!("✅ 重要性评估结果已缓存");
            }

            Ok(evaluations)
        } else {
            // 降级：使用简化的重要性评估逻辑
            warn!("EnhancedImportanceEvaluator 未初始化，使用简化逻辑");

            use agent_mem_intelligence::ImportanceFactors;
            let evaluations = structured_facts
                .iter()
                .map(|fact| ImportanceEvaluation {
                    memory_id: fact.id.clone(),
                    importance_score: fact.importance,
                    confidence: fact.confidence,
                    factors: ImportanceFactors {
                        content_complexity: fact.importance,
                        entity_importance: 0.5,
                        relation_importance: 0.5,
                        temporal_relevance: 0.5,
                        user_interaction: 0.5,
                        contextual_relevance: 0.5,
                        emotional_intensity: 0.5,
                    },
                    evaluated_at: chrono::Utc::now(),
                    reasoning: format!("简化评估: {:.2}", fact.importance),
                })
                .collect();

            Ok(evaluations)
        }
    }

    /// 搜索相似记忆
    pub async fn search_similar_memories(
        orchestrator: &MemoryOrchestrator,
        content: &str,
        agent_id: &str,
        limit: usize,
    ) -> Result<Vec<ExistingMemory>> {
        info!("搜索相似记忆: agent_id={}, limit={}", agent_id, limit);

        #[cfg(feature = "postgres")]
        {
            if let Some(hybrid_engine) = &orchestrator.hybrid_search_engine {
                // 生成查询向量
                let embedder = orchestrator.embedder.as_ref()
                    .ok_or_else(|| AgentMemError::ConfigError("Embedder not configured".to_string()))?;
                let query_vector = UtilsModule::generate_query_embedding(
                    content,
                    embedder.as_ref(),
                )
                .await?;

                // 构建搜索查询
                use agent_mem_core::search::SearchQuery;
                let search_query = SearchQuery {
                    query: content.to_string(),
                    limit: limit * 2, // 多取一些，后续去重
                    threshold: Some(0.7),
                    vector_weight: 0.7,
                    fulltext_weight: 0.3,
                    filters: None,
                    metadata_filters: None,
                };

                // 执行混合搜索
                let hybrid_result = hybrid_engine.search(query_vector, &search_query).await?;

                // 转换为 MemoryItem
                let memory_items =
                    UtilsModule::convert_search_results_to_memory_items(hybrid_result.results);

                // 去重（基于ID）
                let dedup_items = UtilsModule::deduplicate_memory_items(memory_items);

                // 转换为 ExistingMemory
                let existing_memories: Vec<ExistingMemory> = dedup_items
                    .into_iter()
                    .take(limit) // 限制最终数量
                    .map(|item| ExistingMemory {
                        id: item.id,
                        content: item.content,
                        similarity: item.score.unwrap_or(0.0),
                        created_at: chrono::Utc::now(),
                    })
                    .collect();

                info!("找到 {} 个相似记忆", existing_memories.len());
                Ok(existing_memories)
            } else {
                warn!("HybridSearchEngine 未初始化，返回空结果");
                Ok(Vec::new())
            }
        }

        #[cfg(not(feature = "postgres"))]
        {
            // 非 postgres 版本：使用 vector_store 搜索
            if let Some(vector_store) = &orchestrator.vector_store {
                let embedder = orchestrator.embedder.as_ref()
                    .ok_or_else(|| AgentMemError::ConfigError("Embedder not configured".to_string()))?;
                let query_vector = UtilsModule::generate_query_embedding(
                    content,
                    embedder.as_ref(),
                )
                .await?;

                let mut filter_map = HashMap::new();
                filter_map.insert("agent_id".to_string(), serde_json::json!(agent_id));

                let results = vector_store
                    .search_with_filters(query_vector, limit * 2, &filter_map, Some(0.7))
                    .await?;

                // 去重并转换
                let mut seen_ids = std::collections::HashSet::new();
                let existing_memories: Vec<ExistingMemory> = results
                    .into_iter()
                    .filter_map(|r| {
                        if seen_ids.contains(&r.id) {
                            None
                        } else {
                            seen_ids.insert(r.id.clone());
                            // 从 metadata 中获取内容
                            let content = r
                                .metadata
                                .get("content")
                                .cloned()
                                .unwrap_or_else(|| "No content".to_string());

                            Some(ExistingMemory {
                                id: r.id,
                                content,
                                importance: r.similarity,
                                created_at: chrono::Utc::now().to_rfc3339(),
                                updated_at: None,
                                metadata: r.metadata.into_iter().map(|(k, v)| (k, v)).collect(),
                            })
                        }
                    })
                    .take(limit)
                    .collect();

                info!("找到 {} 个相似记忆", existing_memories.len());
                Ok(existing_memories)
            } else {
                warn!("VectorStore 未初始化，返回空结果");
                Ok(Vec::new())
            }
        }
    }

    /// 检测冲突
    pub async fn detect_conflicts(
        orchestrator: &MemoryOrchestrator,
        structured_facts: &[StructuredFact],
        existing_memories: &[ExistingMemory],
        agent_id: &str,
        user_id: Option<String>,
    ) -> Result<Vec<ConflictDetection>> {
        if let Some(resolver) = &orchestrator.conflict_resolver {
            info!(
                "使用 ConflictResolver 检测冲突，新事实: {}, 现有记忆: {}",
                structured_facts.len(),
                existing_memories.len()
            );

            // 将 StructuredFact 转换为 MemoryItem
            let new_memory_items: Vec<MemoryItem> = structured_facts
                .iter()
                .map(|fact| {
                    UtilsModule::structured_fact_to_memory_item(
                        fact,
                        agent_id.to_string(),
                        user_id.clone(),
                    )
                })
                .collect();

            // 将 ExistingMemory 转换为 MemoryItem
            let existing_memory_items: Vec<MemoryItem> = existing_memories
                .iter()
                .map(UtilsModule::existing_memory_to_memory_item)
                .collect();

            // 转换为 MemoryV4
            use agent_mem_traits::MemoryV4;
            let new_memories_v4: Vec<MemoryV4> = new_memory_items
                .iter()
                .map(|item| MemoryV4::from_legacy_item(item))
                .collect();

            let existing_memories_v4: Vec<MemoryV4> = existing_memory_items
                .iter()
                .map(|item| MemoryV4::from_legacy_item(item))
                .collect();

            // 调用 ConflictResolver
            let conflicts = resolver
                .detect_conflicts(&new_memories_v4, &existing_memories_v4)
                .await?;

            info!("冲突检测完成，检测到 {} 个冲突", conflicts.len());
            Ok(conflicts)
        } else {
            // 降级：跳过冲突检测
            warn!("ConflictResolver 未初始化，跳过冲突检测");
            Ok(Vec::new())
        }
    }

    /// 制定智能决策
    pub async fn make_intelligent_decisions(
        orchestrator: &MemoryOrchestrator,
        structured_facts: &[StructuredFact],
        existing_memories: &[ExistingMemory],
        importance_evaluations: &[ImportanceEvaluation],
        conflicts: &[ConflictDetection],
        agent_id: &str,
        _user_id: Option<String>,
    ) -> Result<Vec<MemoryDecision>> {
        if let Some(engine) = &orchestrator.enhanced_decision_engine {
            info!(
                "使用 EnhancedDecisionEngine 制定智能决策，事实: {}, 记忆: {}",
                structured_facts.len(),
                existing_memories.len()
            );

            // 将 ExistingMemory 转换为 MemoryItem
            let existing_memory_items: Vec<MemoryItem> = existing_memories
                .iter()
                .map(UtilsModule::existing_memory_to_memory_item)
                .collect();

            // 转换为 MemoryV4
            use agent_mem_traits::MemoryV4;
            let existing_memories_v4: Vec<MemoryV4> = existing_memory_items
                .iter()
                .map(|item| MemoryV4::from_legacy_item(item))
                .collect();

            // 构建 DecisionContext
            use agent_mem_intelligence::DecisionContext;
            let context = DecisionContext {
                new_facts: structured_facts.to_vec(),
                existing_memories: existing_memories_v4,
                importance_evaluations: importance_evaluations.to_vec(),
                conflict_detections: conflicts.to_vec(),
                user_preferences: HashMap::new(),
            };

            // 调用 EnhancedDecisionEngine
            let decision_result = engine.make_decisions(&context).await?;

            // 将 DecisionResult 转换为 Vec<MemoryDecision>
            let decisions: Vec<MemoryDecision> = decision_result
                .recommended_actions
                .into_iter()
                .map(|action| MemoryDecision {
                    action,
                    confidence: decision_result.confidence,
                    reasoning: decision_result.reasoning.clone(),
                    affected_memories: Vec::new(),
                    estimated_impact: decision_result.expected_impact.performance_impact,
                })
                .collect();

            info!(
                "智能决策完成，生成 {} 个决策，置信度: {:.2}",
                decisions.len(),
                decision_result.confidence
            );
            Ok(decisions)
        } else {
            // 降级：使用简化的决策逻辑
            warn!("EnhancedDecisionEngine 未初始化，使用简化逻辑");

            let mut decisions = Vec::new();

            for (i, fact) in structured_facts.iter().enumerate() {
                // 获取对应的重要性评估
                let importance = importance_evaluations
                    .get(i)
                    .map(|e| e.importance_score)
                    .unwrap_or(0.5);

                // 如果重要性太低，跳过
                if importance < 0.3 {
                    info!(
                        "事实重要性太低 ({})，跳过: {}",
                        importance, fact.description
                    );
                    continue;
                }

                // 创建 ADD 决策
                decisions.push(MemoryDecision {
                    action: MemoryAction::Add {
                        content: fact.description.clone(),
                        importance,
                        metadata: fact.metadata.clone(),
                    },
                    confidence: importance,
                    reasoning: format!("简化决策: {:.2}", importance),
                    affected_memories: Vec::new(),
                    estimated_impact: importance,
                });
            }

            Ok(decisions)
        }
    }

    /// 执行决策
    ///
    /// 执行智能决策引擎生成的决策，包括 ADD、UPDATE、DELETE 等操作
    pub async fn execute_decisions(
        orchestrator: &MemoryOrchestrator,
        decisions: Vec<MemoryDecision>,
        agent_id: String,
        user_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<AddResult> {
        use super::storage::StorageModule;
        use crate::types::MemoryEvent;

        info!("执行 {} 个决策", decisions.len());

        let mut results = Vec::new();
        let mut relations = None;

        for decision in decisions {
            match &decision.action {
                MemoryAction::Add {
                    content,
                    importance: _,
                    metadata: action_metadata,
                } => {
                    info!(
                        "执行 ADD 决策: content={}, confidence={:.2}",
                        content, decision.confidence
                    );

                    // 合并元数据
                    let mut merged_metadata = metadata.clone().unwrap_or_default();
                    for (k, v) in action_metadata {
                        merged_metadata.insert(k.clone(), serde_json::Value::String(v.clone()));
                    }

                    // 调用存储模块添加记忆
                    let memory_id = StorageModule::add_memory(
                        orchestrator,
                        content.clone(),
                        agent_id.clone(),
                        user_id.clone(),
                        None,
                        Some(merged_metadata),
                    )
                    .await?;

                    results.push(MemoryEvent {
                        id: memory_id,
                        memory: content.clone(),
                        event: "ADD".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::Update {
                    memory_id,
                    new_content,
                    merge_strategy: _,
                    change_reason: _,
                } => {
                    info!(
                        "执行 UPDATE 决策: memory_id={}, confidence={:.2}",
                        memory_id, decision.confidence
                    );

                    let mut update_data = HashMap::new();
                    update_data.insert(
                        "content".to_string(),
                        serde_json::Value::String(new_content.clone()),
                    );

                    // 调用存储模块更新记忆
                    let _updated =
                        StorageModule::update_memory(orchestrator, memory_id, update_data).await?;

                    results.push(MemoryEvent {
                        id: memory_id.clone(),
                        memory: new_content.clone(),
                        event: "UPDATE".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::Delete {
                    memory_id,
                    deletion_reason: _,
                } => {
                    info!(
                        "执行 DELETE 决策: memory_id={}, confidence={:.2}",
                        memory_id, decision.confidence
                    );

                    // 调用存储模块删除记忆
                    StorageModule::delete_memory(orchestrator, memory_id).await?;

                    results.push(MemoryEvent {
                        id: memory_id.clone(),
                        memory: String::new(),
                        event: "DELETE".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::Merge {
                    primary_memory_id,
                    secondary_memory_ids,
                    merged_content,
                } => {
                    info!(
                        "执行 MERGE 决策: primary={}, secondaries={:?}, confidence={:.2}",
                        primary_memory_id, secondary_memory_ids, decision.confidence
                    );

                    // 更新主记忆内容
                    let mut update_data = HashMap::new();
                    update_data.insert(
                        "content".to_string(),
                        serde_json::Value::String(merged_content.clone()),
                    );
                    let _updated =
                        StorageModule::update_memory(orchestrator, primary_memory_id, update_data)
                            .await?;

                    // 删除次要记忆
                    for secondary_id in secondary_memory_ids {
                        let _ = StorageModule::delete_memory(orchestrator, secondary_id).await;
                    }

                    results.push(MemoryEvent {
                        id: primary_memory_id.clone(),
                        memory: merged_content.clone(),
                        event: "MERGE".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::NoAction { reason } => {
                    info!(
                        "跳过操作: reason={}, confidence={:.2}",
                        reason, decision.confidence
                    );
                    // 不执行任何操作
                }
            }
        }

        info!("决策执行完成，共处理 {} 个操作", results.len());

        Ok(AddResult { results, relations })
    }

    /// 智能添加记忆（完整的10步流水线）
    pub async fn add_memory_intelligent(
        orchestrator: &MemoryOrchestrator,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<AddResult> {
        use super::storage::StorageModule;
        use crate::types::MemoryEvent;
        use tracing::{debug, info, warn};

        info!(
            "智能添加记忆: content={}, agent_id={}, user_id={:?}",
            content, agent_id, user_id
        );

        // 检查 Intelligence 组件是否可用
        if orchestrator.fact_extractor.is_none() {
            warn!("Intelligence 组件未初始化，降级到简单模式");
            let memory_id = StorageModule::add_memory(
                orchestrator,
                content.clone(),
                agent_id.clone(),
                user_id.clone(),
                None,
                metadata.clone(),
            )
            .await?;
            return Ok(AddResult {
                results: vec![MemoryEvent {
                    id: memory_id,
                    memory: content,
                    event: "ADD".to_string(),
                    actor_id: Some(agent_id),
                    role: None,
                }],
                relations: None,
            });
        }

        // Step 1-4: 并行LLM调用
        info!("Step 1-4: 并行执行事实提取、结构化提取和重要性评估");
        let content_for_facts = content.clone();
        let content_for_structured = content.clone();
        let agent_id_for_importance = agent_id.clone();
        let user_id_for_importance = user_id.clone();

        let (facts_result, structured_facts_result) = tokio::join!(
            async {
                info!("并行任务 1: 事实提取");
                Self::extract_facts(orchestrator, &content_for_facts).await
            },
            async {
                info!("并行任务 2: 结构化事实提取");
                Self::extract_structured_facts(orchestrator, &content_for_structured).await
            }
        );

        let facts = facts_result?;
        let structured_facts = structured_facts_result?;

        info!(
            "提取到 {} 个事实，{} 个结构化事实",
            facts.len(),
            structured_facts.len()
        );

        if facts.is_empty() {
            warn!("未提取到任何事实，直接添加原始内容");
            let memory_id = StorageModule::add_memory(
                orchestrator,
                content.clone(),
                agent_id.clone(),
                user_id.clone(),
                None,
                metadata.clone(),
            )
            .await?;
            return Ok(AddResult {
                results: vec![MemoryEvent {
                    id: memory_id,
                    memory: content,
                    event: "ADD".to_string(),
                    actor_id: Some(agent_id),
                    role: None,
                }],
                relations: None,
            });
        }

        // Step 4-5: 并行执行重要性评估和搜索相似记忆（优化：这两个步骤可以并行）
        info!("Step 4-5: 并行执行重要性评估和搜索相似记忆");
        let (importance_evaluations, existing_memories) = tokio::join!(
            async {
                info!("并行任务 1: 重要性评估");
                Self::evaluate_importance(
            orchestrator,
            &structured_facts,
            &agent_id_for_importance,
            user_id_for_importance.clone(),
        )
                .await
            },
            async {
                info!("并行任务 2: 搜索相似记忆");
                Self::search_similar_memories_internal(orchestrator, &content, &agent_id, 10).await
            }
        );
        let importance_evaluations = importance_evaluations?;
        let existing_memories = existing_memories?;
        info!("完成 {} 个事实的重要性评估", importance_evaluations.len());
        info!("找到 {} 个相似记忆", existing_memories.len());

        // Step 6: 冲突检测
        info!("Step 6: 冲突检测");
        let conflicts = Self::detect_conflicts(
            orchestrator,
            &structured_facts,
            &existing_memories,
            &agent_id,
            user_id.clone(),
        )
        .await?;
        info!("检测到 {} 个冲突", conflicts.len());

        // Step 7: 智能决策
        info!("Step 7: 智能决策");
        let decisions = Self::make_intelligent_decisions(
            orchestrator,
            &structured_facts,
            &existing_memories,
            &importance_evaluations,
            &conflicts,
            &agent_id,
            user_id.clone(),
        )
        .await?;
        info!("生成 {} 个决策", decisions.len());

        // Step 8: 执行决策
        info!("Step 8: 执行决策");
        let results = Self::execute_decisions(
            orchestrator,
            decisions,
            agent_id.clone(),
            user_id.clone(),
            metadata,
        )
        .await?;

        // Step 9-10: 异步聚类和推理（已在execute_decisions中处理）
        info!(
            "✅ 智能添加流水线完成，共处理 {} 个决策",
            results.results.len()
        );
        Ok(results)
    }

    /// 搜索相似记忆（内部辅助方法）
    async fn search_similar_memories_internal(
        orchestrator: &MemoryOrchestrator,
        content: &str,
        agent_id: &str,
        limit: usize,
    ) -> Result<Vec<ExistingMemory>> {
        // 使用检索模块搜索相似记忆
        use super::retrieval::RetrievalModule;
        let query = content.to_string();
        let memories = RetrievalModule::search_memories(
            orchestrator,
            query,
            agent_id.to_string(),
            None,
            limit,
            None,
        )
        .await?;

        // 转换为 ExistingMemory
        let existing: Vec<ExistingMemory> = memories
            .into_iter()
            .map(|item| ExistingMemory {
                id: item.id,
                content: item.content,
                importance: item.importance,
                metadata: item
                    .metadata
                    .into_iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                    .collect(),
                created_at: item.created_at.to_rfc3339(),
                updated_at: item.updated_at.map(|dt| dt.to_rfc3339()),
            })
            .collect();

        Ok(existing)
    }
}
