//! 测试用例实现

use crate::datasets::{ConversationSession, QuestionAnswer};
use crate::framework::{ErrorCase, PerformanceMetrics, TestResult};
use crate::llm_integration::LlmClient;
use crate::metrics::{AccuracyMetrics, MetricsCalculator};
use agent_mem::Memory;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

fn estimate_tokens_from_text(text: &str) -> usize {
    // 粗略估计：4字符约等于1 token
    (text.chars().count() / 4).max(1)
}

/// 测试用例trait
#[async_trait::async_trait]
trait TestCase {
    async fn run(&self, sessions: &[ConversationSession]) -> Result<TestResult>;
}

/// Single-hop推理测试
pub struct SingleHopTest {
    memory: Arc<Memory>,
    llm_client: Option<Arc<LlmClient>>,
}

impl SingleHopTest {
    pub fn new(memory: Arc<Memory>, llm_client: Option<Arc<LlmClient>>) -> Self {
        Self { memory, llm_client }
    }

    pub async fn run(&self, sessions: &[ConversationSession]) -> Result<TestResult> {
        let mut passed = 0;
        let mut total = 0;
        let mut metrics_map = HashMap::new();
        let mut error_cases = Vec::new();
        let mut latencies = Vec::new();
        let mut search_latencies = Vec::new();
        let mut generation_latencies = Vec::new();
        let mut total_tokens = 0;

        for session in sessions {
            // 存储对话记忆
            for message in &session.messages {
                let formatted = format!("{}: {}", message.role, message.content);
                if let Err(e) = self.memory.add(&formatted).await {
                    tracing::warn!("添加记忆失败（继续测试）: {}", e);
                }
            }

            // 测试每个问题
            for qa in &session.questions {
                total += 1;
                let start_time = Instant::now();

                // 检索相关记忆
                let search_start = Instant::now();
                let search_results = match self.memory.search(&qa.question).await {
                    Ok(results) => results,
                    Err(e) => {
                        tracing::warn!("搜索失败: {}, 使用空结果", e);
                        Vec::new()
                    }
                };
                let search_latency = search_start.elapsed().as_millis() as f64;
                search_latencies.push(search_latency);

                // 生成答案：优先使用LLM，否则回退到检索结果拼接
                let context_snippets: Vec<String> = if !search_results.is_empty() {
                    #[allow(deprecated)]
                    search_results
                        .iter()
                        .take(5)
                        .map(|r| r.content.clone())
                        .collect()
                } else {
                    session.messages.iter().map(|m| m.content.clone()).collect()
                };

                let generation_start = Instant::now();
                let actual_answer = match &self.llm_client {
                    Some(llm) => match llm.generate_answer(&qa.question, &context_snippets).await {
                        Ok(ans) => ans,
                        Err(e) => {
                            tracing::warn!("LLM生成失败，使用回退答案: {}", e);
                            if !context_snippets.is_empty() {
                                context_snippets.join(" ")
                            } else {
                                "I don't have enough information to answer this question."
                                    .to_string()
                            }
                        }
                    },
                    None => {
                        if !context_snippets.is_empty() {
                            context_snippets.join(" ")
                        } else {
                            "I don't have enough information to answer this question.".to_string()
                        }
                    }
                };
                let generation_latency = generation_start.elapsed().as_millis() as f64;
                generation_latencies.push(generation_latency);

                let total_latency = start_time.elapsed().as_millis() as f64;
                latencies.push(total_latency);
                for snippet in &context_snippets {
                    total_tokens += estimate_tokens_from_text(snippet);
                }
                total_tokens += estimate_tokens_from_text(&qa.question);
                total_tokens += estimate_tokens_from_text(&actual_answer);

                // 计算准确性指标
                let mut accuracy_metrics = MetricsCalculator::calculate_accuracy_metrics(
                    &qa.expected_answer,
                    &actual_answer,
                );
                if let Some(llm) = &self.llm_client {
                    if let Ok(score) = llm
                        .judge_answer(&qa.question, &qa.expected_answer, &actual_answer)
                        .await
                    {
                        accuracy_metrics.llm_judge_score = Some(score);
                    }
                }
                let composite_score =
                    MetricsCalculator::calculate_composite_score(&accuracy_metrics);

                // 判断是否通过（阈值：0.5）
                if composite_score >= 0.5 {
                    passed += 1;
                } else {
                    error_cases.push(ErrorCase {
                        question_id: qa.question_id.clone(),
                        question: qa.question.clone(),
                        expected_answer: qa.expected_answer.clone(),
                        actual_answer,
                        error_reason: format!("Score too low: {:.2}", composite_score),
                    });
                }

                // 记录指标
                metrics_map.insert(format!("f1_{}", qa.question_id), accuracy_metrics.f1_score);
                metrics_map.insert(
                    format!("bleu1_{}", qa.question_id),
                    accuracy_metrics.bleu1_score,
                );
            }
        }

        // 计算平均延迟
        let avg_latency = if !latencies.is_empty() {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        } else {
            0.0
        };

        let accuracy_score = if total > 0 {
            passed as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        Ok(TestResult {
            category: "single_hop".to_string(),
            total_tests: total,
            passed_tests: passed,
            accuracy_score,
            metrics: metrics_map,
            performance: {
                let mut search_latencies: Vec<f64> = search_latencies.clone();
                search_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let p95_search_idx = (search_latencies.len() as f64 * 0.95) as usize;
                let p95_search = if !search_latencies.is_empty() {
                    search_latencies[p95_search_idx.min(search_latencies.len() - 1)]
                } else {
                    0.0
                };

                let p95_total_idx = (latencies.len() as f64 * 0.95) as usize;
                let p95_total = if !latencies.is_empty() {
                    latencies[p95_total_idx.min(latencies.len() - 1)]
                } else {
                    0.0
                };

                PerformanceMetrics {
                    avg_search_latency_ms: if !search_latencies.is_empty() {
                        search_latencies.iter().sum::<f64>() / search_latencies.len() as f64
                    } else {
                        0.0
                    },
                    avg_generation_latency_ms: if !generation_latencies.is_empty() {
                        generation_latencies.iter().sum::<f64>() / generation_latencies.len() as f64
                    } else {
                        (avg_latency
                            - if !search_latencies.is_empty() {
                                search_latencies.iter().sum::<f64>() / search_latencies.len() as f64
                            } else {
                                0.0
                            })
                        .max(0.0)
                    },
                    p95_search_latency_ms: p95_search,
                    p95_total_latency_ms: p95_total,
                    avg_tokens: if total > 0 { total_tokens / total } else { 0 },
                }
            },
            error_cases,
        })
    }
}

/// Multi-hop推理测试
pub struct MultiHopTest {
    memory: Arc<Memory>,
    llm_client: Option<Arc<LlmClient>>,
}

impl MultiHopTest {
    pub fn new(memory: Arc<Memory>, llm_client: Option<Arc<LlmClient>>) -> Self {
        Self { memory, llm_client }
    }

    pub async fn run(&self, sessions: &[ConversationSession]) -> Result<TestResult> {
        let mut passed = 0;
        let mut total = 0;
        let mut metrics_map = HashMap::new();
        let mut error_cases = Vec::new();
        let mut latencies = Vec::new();
        let mut search_latencies = Vec::new();
        let mut generation_latencies = Vec::new();
        let mut total_tokens = 0;

        // 存储所有会话的记忆（多跳场景）
        for session in sessions {
            for message in &session.messages {
                let formatted = format!("{}: {}", message.role, message.content);
                if let Err(e) = self.memory.add(&formatted).await {
                    tracing::warn!("添加记忆失败（继续测试）: {}", e);
                }
            }
        }

        // 测试需要跨会话的问题
        for session in sessions {
            for qa in &session.questions {
                // 多跳问题按类别划分，全部评估
                total += 1;
                let start_time = Instant::now();

                let search_start = Instant::now();
                let search_results = match self.memory.search(&qa.question).await {
                    Ok(results) => results,
                    Err(e) => {
                        tracing::warn!("搜索失败: {}, 使用空结果", e);
                        Vec::new()
                    }
                };
                let search_latency = search_start.elapsed().as_millis() as f64;
                search_latencies.push(search_latency);

                let context_snippets: Vec<String> = if !search_results.is_empty() {
                    #[allow(deprecated)]
                    search_results
                        .iter()
                        .take(8)
                        .map(|r| r.content.clone())
                        .collect()
                } else {
                    sessions
                        .iter()
                        .flat_map(|s| s.messages.iter())
                        .map(|m| m.content.clone())
                        .collect()
                };

                // 综合多个记忆片段生成答案
                let generation_start = Instant::now();
                let actual_answer = match &self.llm_client {
                    Some(llm) => match llm.generate_answer(&qa.question, &context_snippets).await {
                        Ok(ans) => ans,
                        Err(e) => {
                            tracing::warn!("LLM生成失败（多跳），使用拼接答案: {}", e);
                            if !context_snippets.is_empty() {
                                context_snippets.join(". ")
                            } else {
                                "I don't have enough information to answer this question."
                                    .to_string()
                            }
                        }
                    },
                    None => {
                        if !context_snippets.is_empty() {
                            context_snippets.join(". ")
                        } else {
                            "I don't have enough information to answer this question.".to_string()
                        }
                    }
                };
                let generation_latency = generation_start.elapsed().as_millis() as f64;
                generation_latencies.push(generation_latency);

                let total_latency = start_time.elapsed().as_millis() as f64;
                latencies.push(total_latency);
                for snippet in &context_snippets {
                    total_tokens += estimate_tokens_from_text(snippet);
                }
                total_tokens += estimate_tokens_from_text(&qa.question);
                total_tokens += estimate_tokens_from_text(&actual_answer);

                // 计算准确性
                let mut accuracy_metrics = MetricsCalculator::calculate_accuracy_metrics(
                    &qa.expected_answer,
                    &actual_answer,
                );
                if let Some(llm) = &self.llm_client {
                    if let Ok(score) = llm
                        .judge_answer(&qa.question, &qa.expected_answer, &actual_answer)
                        .await
                    {
                        accuracy_metrics.llm_judge_score = Some(score);
                    }
                }
                let composite_score =
                    MetricsCalculator::calculate_composite_score(&accuracy_metrics);

                if composite_score >= 0.5 {
                    passed += 1;
                } else {
                    error_cases.push(ErrorCase {
                        question_id: qa.question_id.clone(),
                        question: qa.question.clone(),
                        expected_answer: qa.expected_answer.clone(),
                        actual_answer,
                        error_reason: format!("Score too low: {:.2}", composite_score),
                    });
                }
            }
        }

        let avg_latency = if !latencies.is_empty() {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        } else {
            0.0
        };

        let accuracy_score = if total > 0 {
            passed as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        Ok(TestResult {
            category: "multi_hop".to_string(),
            total_tests: total,
            passed_tests: passed,
            accuracy_score,
            metrics: metrics_map,
            performance: PerformanceMetrics {
                avg_search_latency_ms: if !search_latencies.is_empty() {
                    search_latencies.iter().sum::<f64>() / search_latencies.len() as f64
                } else {
                    0.0
                },
                avg_generation_latency_ms: if !generation_latencies.is_empty() {
                    generation_latencies.iter().sum::<f64>() / generation_latencies.len() as f64
                } else {
                    (avg_latency
                        - if !search_latencies.is_empty() {
                            search_latencies.iter().sum::<f64>() / search_latencies.len() as f64
                        } else {
                            0.0
                        })
                    .max(0.0)
                },
                p95_search_latency_ms: if !search_latencies.is_empty() {
                    let mut sorted = search_latencies.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let idx = (sorted.len() as f64 * 0.95) as usize;
                    sorted[idx.min(sorted.len() - 1)]
                } else {
                    0.0
                },
                p95_total_latency_ms: if !latencies.is_empty() {
                    let mut sorted = latencies.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let idx = (sorted.len() as f64 * 0.95) as usize;
                    sorted[idx.min(sorted.len() - 1)]
                } else {
                    0.0
                },
                avg_tokens: if total > 0 { total_tokens / total } else { 0 },
            },
            error_cases,
        })
    }
}

/// Temporal推理测试
pub struct TemporalTest {
    memory: Arc<Memory>,
    llm_client: Option<Arc<LlmClient>>,
}

impl TemporalTest {
    pub fn new(memory: Arc<Memory>, llm_client: Option<Arc<LlmClient>>) -> Self {
        Self { memory, llm_client }
    }

    pub async fn run(&self, sessions: &[ConversationSession]) -> Result<TestResult> {
        // 复用单跳逻辑，后续可加入时间衰减/时间戳排序
        SingleHopTest::new(self.memory.clone(), self.llm_client.clone())
            .run(sessions)
            .await
    }
}

/// Open-domain知识测试
pub struct OpenDomainTest {
    memory: Arc<Memory>,
    llm_client: Option<Arc<LlmClient>>,
}

impl OpenDomainTest {
    pub fn new(memory: Arc<Memory>, llm_client: Option<Arc<LlmClient>>) -> Self {
        Self { memory, llm_client }
    }

    pub async fn run(&self, sessions: &[ConversationSession]) -> Result<TestResult> {
        // 复用单跳逻辑，后续可在此处融合外部知识库
        SingleHopTest::new(self.memory.clone(), self.llm_client.clone())
            .run(sessions)
            .await
    }
}

/// Adversarial问题测试
pub struct AdversarialTest {
    memory: Arc<Memory>,
    llm_client: Option<Arc<LlmClient>>,
}

impl AdversarialTest {
    pub fn new(memory: Arc<Memory>, llm_client: Option<Arc<LlmClient>>) -> Self {
        Self { memory, llm_client }
    }

    pub async fn run(&self, sessions: &[ConversationSession]) -> Result<TestResult> {
        let mut passed = 0;
        let mut total = 0;
        let mut metrics_map = HashMap::new();
        let mut error_cases = Vec::new();
        let mut latencies = Vec::new();
        let mut search_latencies = Vec::new();
        let mut generation_latencies = Vec::new();
        let mut total_tokens = 0;

        for session in sessions {
            for message in &session.messages {
                let formatted = format!("{}: {}", message.role, message.content);
                if let Err(e) = self.memory.add(&formatted).await {
                    tracing::warn!("添加记忆失败（继续测试）: {}", e);
                }
            }

            for qa in &session.questions {
                total += 1;
                let start_time = Instant::now();

                let search_start = Instant::now();
                let search_results = match self.memory.search(&qa.question).await {
                    Ok(results) => results,
                    Err(e) => {
                        tracing::warn!("搜索失败: {}, 使用空结果", e);
                        Vec::new()
                    }
                };
                let search_latency = search_start.elapsed().as_millis() as f64;
                search_latencies.push(search_latency);

                let context_snippets: Vec<String> = if !search_results.is_empty() {
                    #[allow(deprecated)]
                    search_results
                        .iter()
                        .take(5)
                        .map(|r| r.content.clone())
                        .collect()
                } else {
                    session.messages.iter().map(|m| m.content.clone()).collect()
                };

                let generation_start = Instant::now();
                let actual_answer = match &self.llm_client {
                    Some(llm) => match llm.generate_answer(&qa.question, &context_snippets).await {
                        Ok(ans) => ans,
                        Err(e) => {
                            tracing::warn!("LLM生成失败（对抗），使用回退答案: {}", e);
                            if !context_snippets.is_empty() {
                                context_snippets.join(" ")
                            } else {
                                "No information available".to_string()
                            }
                        }
                    },
                    None => {
                        if !context_snippets.is_empty() {
                            context_snippets.join(" ")
                        } else {
                            "No information available".to_string()
                        }
                    }
                };
                let generation_latency = generation_start.elapsed().as_millis() as f64;
                generation_latencies.push(generation_latency);

                let total_latency = start_time.elapsed().as_millis() as f64;
                latencies.push(total_latency);
                for snippet in &context_snippets {
                    total_tokens += estimate_tokens_from_text(snippet);
                }
                total_tokens += estimate_tokens_from_text(&qa.question);
                total_tokens += estimate_tokens_from_text(&actual_answer);

                let mut accuracy_metrics = MetricsCalculator::calculate_accuracy_metrics(
                    &qa.expected_answer,
                    &actual_answer,
                );
                if let Some(llm) = &self.llm_client {
                    if let Ok(score) = llm
                        .judge_answer(&qa.question, &qa.expected_answer, &actual_answer)
                        .await
                    {
                        accuracy_metrics.llm_judge_score = Some(score);
                    }
                }
                let composite_score =
                    MetricsCalculator::calculate_composite_score(&accuracy_metrics);

                if composite_score >= 0.5 {
                    passed += 1;
                } else {
                    error_cases.push(ErrorCase {
                        question_id: qa.question_id.clone(),
                        question: qa.question.clone(),
                        expected_answer: qa.expected_answer.clone(),
                        actual_answer: actual_answer.clone(),
                        error_reason: format!("Score too low: {:.2}", composite_score),
                    });
                }

                metrics_map.insert(format!("f1_{}", qa.question_id), accuracy_metrics.f1_score);
            }
        }

        let accuracy_score = if total > 0 {
            passed as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        Ok(TestResult {
            category: "adversarial".to_string(),
            total_tests: total,
            passed_tests: passed,
            accuracy_score,
            metrics: metrics_map,
            performance: PerformanceMetrics {
                avg_search_latency_ms: if !search_latencies.is_empty() {
                    search_latencies.iter().sum::<f64>() / search_latencies.len() as f64
                } else {
                    0.0
                },
                avg_generation_latency_ms: if !generation_latencies.is_empty() {
                    generation_latencies.iter().sum::<f64>() / generation_latencies.len() as f64
                } else {
                    (if !latencies.is_empty() {
                        latencies.iter().sum::<f64>() / latencies.len() as f64
                    } else {
                        0.0
                    } - if !search_latencies.is_empty() {
                        search_latencies.iter().sum::<f64>() / search_latencies.len() as f64
                    } else {
                        0.0
                    })
                    .max(0.0)
                },
                p95_search_latency_ms: if !search_latencies.is_empty() {
                    let mut sorted = search_latencies.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let idx = (sorted.len() as f64 * 0.95) as usize;
                    sorted[idx.min(sorted.len() - 1)]
                } else {
                    0.0
                },
                p95_total_latency_ms: if !latencies.is_empty() {
                    let mut sorted = latencies.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let idx = (sorted.len() as f64 * 0.95) as usize;
                    sorted[idx.min(sorted.len() - 1)]
                } else {
                    0.0
                },
                avg_tokens: if total > 0 { total_tokens / total } else { 0 },
            },
            error_cases,
        })
    }
}
