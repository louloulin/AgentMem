//! LOCOMO测试框架核心

use crate::datasets::{ConversationSession, DatasetLoader};
use crate::llm_integration::{LlmClient, LlmConfig};
pub use crate::metrics::PerformanceMetrics;
use crate::test_cases::{
    AdversarialTest, MultiHopTest, OpenDomainTest, SingleHopTest, TemporalTest,
};
use agent_mem::{DeleteAllOptions, Memory};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// 测试数据集路径
    pub dataset_path: String,
    /// 是否启用详细输出
    pub verbose: bool,
    /// LLM配置
    pub llm_config: Option<LlmConfig>,
    /// 每个类别最多使用多少个会话（None 表示使用全部）
    pub max_sessions_per_category: Option<usize>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            dataset_path: "data".to_string(),
            verbose: true,
            llm_config: None,
            max_sessions_per_category: None,
        }
    }
}

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// 测试类别
    pub category: String,
    /// 总测试数
    pub total_tests: usize,
    /// 通过测试数
    pub passed_tests: usize,
    /// 准确性得分
    pub accuracy_score: f64,
    /// 详细指标
    pub metrics: HashMap<String, f64>,
    /// 性能指标
    pub performance: PerformanceMetrics,
    /// 错误案例
    pub error_cases: Vec<ErrorCase>,
}

/// 错误案例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCase {
    /// 问题ID
    pub question_id: String,
    /// 问题
    pub question: String,
    /// 期望答案
    pub expected_answer: String,
    /// 实际答案
    pub actual_answer: String,
    /// 错误原因
    pub error_reason: String,
}

/// 总体测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallTestResults {
    /// 各分类测试结果
    pub category_results: HashMap<String, TestResult>,
    /// 总体得分
    pub overall_score: f64,
    /// 总体性能指标
    pub overall_performance: PerformanceMetrics,
    /// 测试时间
    pub test_duration_secs: f64,
}

/// LOCOMO测试框架
pub struct LocomoTestFramework {
    config: TestConfig,
    memory: Arc<Memory>,
    llm_client: Option<Arc<LlmClient>>,
}

impl LocomoTestFramework {
    /// 创建新的测试框架（异步版本）
    pub async fn new() -> Result<Self> {
        let config = TestConfig::default();
        Self::with_config_async(config).await
    }

    /// 使用自定义配置创建（异步版本）
    pub async fn with_config_async(config: TestConfig) -> Result<Self> {
        // 使用 mem0_mode 确保所有组件正确初始化
        // 这会自动配置 FastEmbed、LibSQL 和 LanceDB
        let memory = Memory::mem0_mode().await?;
        let llm_client = config
            .llm_config
            .clone()
            .map(|cfg| Arc::new(LlmClient::new(cfg)));

        Ok(Self {
            config,
            memory: Arc::new(memory),
            llm_client,
        })
    }

    /// 使用自定义配置创建（同步版本，用于非tokio环境）
    pub fn with_config(config: TestConfig) -> Result<Self> {
        let rt = tokio::runtime::Runtime::new()?;
        let memory = rt.block_on(async {
            // 使用 mem0_mode 确保所有组件正确初始化
            Memory::mem0_mode().await
        })?;
        let llm_client = config
            .llm_config
            .clone()
            .map(|cfg| Arc::new(LlmClient::new(cfg)));

        Ok(Self {
            config,
            memory: Arc::new(memory),
            llm_client,
        })
    }

    /// 运行所有测试
    pub async fn run_all_tests(&self) -> Result<OverallTestResults> {
        use std::time::Instant;
        let start_time = Instant::now();

        println!("📋 加载测试数据集...");
        let dataset_loader = DatasetLoader::new(&self.config.dataset_path);
        let datasets = dataset_loader.load_all().await?;

        // 如果设置了每类最大会话数，则在内存中做一次裁剪，避免一次性加载过多样本导致 OOM
        let limit = self.config.max_sessions_per_category;
        let limit_sessions = |sessions: &Vec<ConversationSession>| -> Vec<ConversationSession> {
            if let Some(max) = limit {
                sessions.iter().cloned().take(max).collect()
            } else {
                sessions.clone()
            }
        };

        let single_hop_data = limit_sessions(&datasets.single_hop);
        let multi_hop_data = limit_sessions(&datasets.multi_hop);
        let temporal_data = limit_sessions(&datasets.temporal);
        let open_domain_data = limit_sessions(&datasets.open_domain);
        let adversarial_data = limit_sessions(&datasets.adversarial);

        let mut category_results = HashMap::new();

        // 1. Single-hop推理测试
        println!("\n🔍 运行Single-hop推理测试...");
        self.reset_memory().await;
        let single_hop_result = self.run_single_hop_test(&single_hop_data).await?;
        category_results.insert("single_hop".to_string(), single_hop_result);

        // 2. Multi-hop推理测试
        println!("\n🔗 运行Multi-hop推理测试...");
        self.reset_memory().await;
        let multi_hop_result = self.run_multi_hop_test(&multi_hop_data).await?;
        category_results.insert("multi_hop".to_string(), multi_hop_result);

        // 3. Temporal推理测试
        println!("\n⏰ 运行Temporal推理测试...");
        self.reset_memory().await;
        let temporal_result = self.run_temporal_test(&temporal_data).await?;
        category_results.insert("temporal".to_string(), temporal_result);

        // 4. Open-domain知识测试
        println!("\n🌐 运行Open-domain知识测试...");
        self.reset_memory().await;
        let open_domain_result = self.run_open_domain_test(&open_domain_data).await?;
        category_results.insert("open_domain".to_string(), open_domain_result);

        // 5. Adversarial问题测试
        println!("\n🛡️ 运行Adversarial问题测试...");
        self.reset_memory().await;
        let adversarial_result = self.run_adversarial_test(&adversarial_data).await?;
        category_results.insert("adversarial".to_string(), adversarial_result);

        // 计算总体得分
        let overall_score = self.calculate_overall_score(&category_results);
        let overall_performance = self.calculate_overall_performance(&category_results);
        let test_duration_secs = start_time.elapsed().as_secs_f64();

        Ok(OverallTestResults {
            category_results,
            overall_score,
            overall_performance,
            test_duration_secs,
        })
    }

    /// 运行Single-hop测试
    async fn run_single_hop_test(&self, test_data: &[ConversationSession]) -> Result<TestResult> {
        let test = SingleHopTest::new(Arc::clone(&self.memory), self.llm_client.clone());
        test.run(test_data).await
    }

    /// 运行Multi-hop测试
    async fn run_multi_hop_test(&self, test_data: &[ConversationSession]) -> Result<TestResult> {
        let test = MultiHopTest::new(Arc::clone(&self.memory), self.llm_client.clone());
        test.run(test_data).await
    }

    /// 运行Temporal测试
    async fn run_temporal_test(&self, test_data: &[ConversationSession]) -> Result<TestResult> {
        let test = TemporalTest::new(Arc::clone(&self.memory), self.llm_client.clone());
        test.run(test_data).await
    }

    /// 运行Open-domain测试
    async fn run_open_domain_test(&self, test_data: &[ConversationSession]) -> Result<TestResult> {
        let test = OpenDomainTest::new(Arc::clone(&self.memory), self.llm_client.clone());
        test.run(test_data).await
    }

    /// 运行Adversarial测试
    async fn run_adversarial_test(&self, test_data: &[ConversationSession]) -> Result<TestResult> {
        let test = AdversarialTest::new(Arc::clone(&self.memory), self.llm_client.clone());
        test.run(test_data).await
    }

    /// 计算总体得分
    fn calculate_overall_score(&self, results: &HashMap<String, TestResult>) -> f64 {
        let scores: Vec<f64> = results.values().map(|r| r.accuracy_score).collect();
        if scores.is_empty() {
            return 0.0;
        }
        scores.iter().sum::<f64>() / scores.len() as f64
    }

    /// 计算总体性能指标
    fn calculate_overall_performance(
        &self,
        results: &HashMap<String, TestResult>,
    ) -> PerformanceMetrics {
        let mut all_search_latencies = Vec::new();
        let mut all_total_latencies = Vec::new();
        let mut total_tokens = 0;
        let mut count = 0;

        for result in results.values() {
            all_search_latencies.push(result.performance.avg_search_latency_ms);
            all_total_latencies.push(
                result.performance.avg_search_latency_ms
                    + result.performance.avg_generation_latency_ms,
            );
            total_tokens += result.performance.avg_tokens;
            count += 1;
        }

        // 计算P95延迟
        all_search_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        all_total_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p95_search = if !all_search_latencies.is_empty() {
            let index = (all_search_latencies.len() as f64 * 0.95) as usize;
            all_search_latencies[index.min(all_search_latencies.len() - 1)]
        } else {
            0.0
        };

        let p95_total = if !all_total_latencies.is_empty() {
            let index = (all_total_latencies.len() as f64 * 0.95) as usize;
            all_total_latencies[index.min(all_total_latencies.len() - 1)]
        } else {
            0.0
        };

        PerformanceMetrics {
            avg_search_latency_ms: if count > 0 {
                all_search_latencies.iter().sum::<f64>() / count as f64
            } else {
                0.0
            },
            avg_generation_latency_ms: if count > 0 {
                results
                    .values()
                    .map(|r| r.performance.avg_generation_latency_ms)
                    .sum::<f64>()
                    / count as f64
            } else {
                0.0
            },
            p95_search_latency_ms: p95_search,
            p95_total_latency_ms: p95_total,
            avg_tokens: if count > 0 { total_tokens / count } else { 0 },
        }
    }

    /// 每个类别开始前清空记忆，避免跨类别污染
    async fn reset_memory(&self) {
        if let Err(e) = self.memory.delete_all(DeleteAllOptions::default()).await {
            tracing::warn!("清空记忆失败（继续执行）: {}", e);
        }
    }
}
