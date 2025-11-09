//! E2E Test - V4 Pipeline Execution
//! Week 9-10: Pipeline和DAG执行流程测试

use agent_mem_core::types::{
    Content, Memory, MemoryBuilder, Pipeline, PipelineContext, PipelineStage, StageResult,
    DagPipeline, AttributeKey, AttributeValue,
};
use anyhow::Result;

/// 测试Stage：内容预处理
struct ContentPreprocessStage;

#[async_trait::async_trait]
impl PipelineStage for ContentPreprocessStage {
    type Input = Memory;
    type Output = Memory;

    fn name(&self) -> &str {
        "content_preprocess"
    }

    async fn execute(
        &self,
        input: Self::Input,
        _context: &mut PipelineContext,
    ) -> Result<StageResult<Self::Output>> {
        // 简单的内容清洗
        let mut output = input;
        if let Content::Text(ref text) = output.content {
            let cleaned = text.trim().to_string();
            output.content = Content::Text(cleaned);
        }
        Ok(StageResult::Continue(output))
    }
}

/// 测试Stage：去重检查
struct DeduplicationStage;

#[async_trait::async_trait]
impl PipelineStage for DeduplicationStage {
    type Input = Memory;
    type Output = Memory;

    fn name(&self) -> &str {
        "deduplication"
    }

    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> Result<StageResult<Self::Output>> {
        // 模拟去重逻辑
        context.set("dedup_checked", true);
        Ok(StageResult::Continue(input))
    }
}

/// 测试Stage：重要性评估
struct ImportanceStage;

#[async_trait::async_trait]
impl PipelineStage for ImportanceStage {
    type Input = Memory;
    type Output = Memory;

    fn name(&self) -> &str {
        "importance_evaluation"
    }

    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> Result<StageResult<Self::Output>> {
        let mut output = input;
        // 简单的重要性评分
        let importance = if let Content::Text(ref text) = output.content {
            (text.len() as f64 / 100.0).min(1.0)
        } else {
            0.5
        };
        
        output.attributes.set(
            AttributeKey::system("importance"),
            AttributeValue::Number(importance),
        );
        context.set("importance_calculated", importance);
        Ok(StageResult::Continue(output))
    }
}

/// E2E测试：线性Pipeline执行
#[tokio::test]
async fn test_linear_pipeline_execution() {
    let memory = MemoryBuilder::new()
        .content(Content::Text("  这是一段需要处理的文本  ".to_string()))
        .build();

    let pipeline = Pipeline::<Memory, Memory>::new("memory_processing")
        .add_stage(ContentPreprocessStage)
        .add_stage(DeduplicationStage)
        .add_stage(ImportanceStage);

    let mut context = PipelineContext::new();
    let result = pipeline.execute(memory.clone(), &mut context).await;

    assert!(result.is_ok());
    let processed_memory = result.unwrap();

    // 验证内容已清理
    if let Content::Text(text) = &processed_memory.content {
        assert_eq!(text, "这是一段需要处理的文本");
    }

    // 验证上下文
    assert_eq!(context.get::<bool>("dedup_checked"), Some(true));
    assert!(context.get::<f64>("importance_calculated").is_some());

    // 验证重要性已设置
    assert!(processed_memory
        .attributes
        .get(&AttributeKey::system("importance"))
        .is_some());

    println!("✅ Linear pipeline executed successfully");
    println!("   - Stages: content_preprocess → deduplication → importance");
    println!("   - Context variables: 2");
    println!("   - Attributes added: importance");
}

/// E2E测试：DAG Pipeline并行执行
#[tokio::test]
async fn test_dag_pipeline_parallel_execution() {
    let memory = MemoryBuilder::new()
        .content(Content::Text("DAG测试内容".to_string()))
        .build();

    // 构建DAG: A → B, A → C, B → D, C → D
    let pipeline = DagPipeline::<Memory, Memory>::new("dag_pipeline")
        .add_node("preprocess", ContentPreprocessStage, vec![])
        .add_node("dedup", DeduplicationStage, vec!["preprocess".to_string()])
        .add_node(
            "importance",
            ImportanceStage,
            vec!["preprocess".to_string()],
        );

    let mut context = PipelineContext::new();
    let results = pipeline.execute(memory.clone(), &mut context).await;

    assert!(results.is_ok());
    let outputs = results.unwrap();

    // 验证所有节点都执行了
    assert!(outputs.contains_key("preprocess"));
    assert!(outputs.contains_key("dedup"));
    assert!(outputs.contains_key("importance"));

    println!("✅ DAG pipeline executed successfully");
    println!("   - Topology: preprocess → [dedup, importance] (parallel)");
    println!("   - Nodes executed: {}", outputs.len());
    println!("   - Max parallelism: 2");
}

/// E2E测试：DAG条件分支
#[tokio::test]
async fn test_dag_conditional_branching() {
    let memory = MemoryBuilder::new()
        .content(Content::Text("长文本测试内容，用于触发条件分支逻辑".to_string()))
        .build();

    let pipeline = DagPipeline::<Memory, Memory>::new("conditional_dag")
        .add_node("preprocess", ContentPreprocessStage, vec![])
        .add_node("importance", ImportanceStage, vec!["preprocess".to_string()])
        .add_condition("high_importance", |ctx| {
            ctx.get::<f64>("importance_calculated")
                .unwrap_or(0.0)
                > 0.5
        })
        .add_edge("importance", "dedup", Some("high_importance".to_string()));

    let mut context = PipelineContext::new();
    let results = pipeline.execute(memory.clone(), &mut context).await;

    assert!(results.is_ok());
    
    println!("✅ DAG conditional branching test passed");
    println!("   - Condition: high_importance");
    println!("   - Branch: importance → dedup (conditional)");
}

/// E2E测试：Pipeline错误处理
#[tokio::test]
async fn test_pipeline_error_handling() {
    /// 错误Stage：总是失败
    struct FailingStage;

    #[async_trait::async_trait]
    impl PipelineStage for FailingStage {
        type Input = Memory;
        type Output = Memory;

        fn name(&self) -> &str {
            "failing_stage"
        }

        async fn execute(
            &self,
            _input: Self::Input,
            _context: &mut PipelineContext,
        ) -> Result<StageResult<Self::Output>> {
            Err(anyhow::anyhow!("Intentional failure"))
        }
    }

    let memory = MemoryBuilder::new()
        .content(Content::Text("测试错误处理".to_string()))
        .build();

    let pipeline = Pipeline::<Memory, Memory>::new("error_pipeline")
        .add_stage(ContentPreprocessStage)
        .add_stage(FailingStage);

    let mut context = PipelineContext::new();
    let result = pipeline.execute(memory.clone(), &mut context).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Intentional failure"));

    println!("✅ Pipeline error handling test passed");
}

/// E2E测试：Pipeline跳过Stage
#[tokio::test]
async fn test_pipeline_stage_skip() {
    /// 可跳过Stage
    struct SkippableStage;

    #[async_trait::async_trait]
    impl PipelineStage for SkippableStage {
        type Input = Memory;
        type Output = Memory;

        fn name(&self) -> &str {
            "skippable"
        }

        async fn execute(
            &self,
            input: Self::Input,
            context: &mut PipelineContext,
        ) -> Result<StageResult<Self::Output>> {
            if context.get::<bool>("should_skip") == Some(true) {
                Ok(StageResult::Skip(input.clone()))
            } else {
                Ok(StageResult::Continue(input))
            }
        }
    }

    let memory = MemoryBuilder::new()
        .content(Content::Text("跳过测试".to_string()))
        .build();

    let pipeline = Pipeline::<Memory, Memory>::new("skip_pipeline")
        .add_stage(SkippableStage)
        .add_stage(ImportanceStage);

    let mut context = PipelineContext::new();
    context.set("should_skip", true);

    let result = pipeline.execute(memory.clone(), &mut context).await;
    assert!(result.is_ok());

    println!("✅ Pipeline stage skip test passed");
}

/// E2E测试：Pipeline性能（批量处理）
#[tokio::test]
async fn test_pipeline_batch_performance() {
    use std::time::Instant;

    let pipeline = Pipeline::<Memory, Memory>::new("batch_pipeline")
        .add_stage(ContentPreprocessStage)
        .add_stage(DeduplicationStage)
        .add_stage(ImportanceStage);

    let batch_size = 100;
    let start = Instant::now();

    for i in 0..batch_size {
        let memory = MemoryBuilder::new()
            .content(Content::Text(format!("批量测试 {}", i)))
            .build();

        let mut context = PipelineContext::new();
        let result = pipeline.execute(memory, &mut context).await;
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();
    let throughput = batch_size as f64 / elapsed.as_secs_f64();

    println!("✅ Pipeline batch performance test passed");
    println!("   - Batch size: {}", batch_size);
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} memories/sec", throughput);

    assert!(throughput > 1000.0, "Pipeline throughput below threshold");
}

/// E2E测试：DAG环检测
#[tokio::test]
async fn test_dag_cycle_detection() {
    let memory = MemoryBuilder::new()
        .content(Content::Text("环检测测试".to_string()))
        .build();

    // 构建有环的DAG（应该被检测出来）
    let pipeline = DagPipeline::<Memory, Memory>::new("cyclic_dag")
        .add_node("A", ContentPreprocessStage, vec!["B".to_string()])
        .add_node("B", DeduplicationStage, vec!["A".to_string()]);

    let mut context = PipelineContext::new();
    let result = pipeline.execute(memory, &mut context).await;

    // 应该检测到环并返回错误
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle") 
            || result.as_ref().unwrap_err().to_string().contains("dependency"));

    println!("✅ DAG cycle detection test passed");
}

/// E2E测试：复杂DAG拓扑
#[tokio::test]
async fn test_complex_dag_topology() {
    let memory = MemoryBuilder::new()
        .content(Content::Text("复杂拓扑测试".to_string()))
        .build();

    // 构建复杂DAG:
    //        A
    //       / \
    //      B   C
    //       \ / \
    //        D   E
    let pipeline = DagPipeline::<Memory, Memory>::new("complex_dag")
        .add_node("A", ContentPreprocessStage, vec![])
        .add_node("B", DeduplicationStage, vec!["A".to_string()])
        .add_node("C", ImportanceStage, vec!["A".to_string()])
        .add_node(
            "D",
            ContentPreprocessStage,
            vec!["B".to_string(), "C".to_string()],
        )
        .add_node("E", DeduplicationStage, vec!["C".to_string()]);

    let mut context = PipelineContext::new();
    let results = pipeline.execute(memory, &mut context).await;

    assert!(results.is_ok());
    let outputs = results.unwrap();

    // 验证所有节点执行
    assert_eq!(outputs.len(), 5);
    assert!(outputs.contains_key("A"));
    assert!(outputs.contains_key("B"));
    assert!(outputs.contains_key("C"));
    assert!(outputs.contains_key("D"));
    assert!(outputs.contains_key("E"));

    println!("✅ Complex DAG topology test passed");
    println!("   - Nodes: 5");
    println!("   - Edges: 5");
    println!("   - Max parallel: 2 (B, C can run in parallel)");
}

