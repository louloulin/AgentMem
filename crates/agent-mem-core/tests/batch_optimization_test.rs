//! Batch optimization integration tests

use agent_mem_core::embeddings_batch::{
    BatchPerformanceComparison, EmbeddingBatchConfig, EmbeddingBatchProcessor, EmbeddingBatchStats,
};

#[tokio::test]
async fn test_embedding_batch_processor_basic() {
    let config = EmbeddingBatchConfig {
        max_batch_size: 10,
        min_batch_size: 5,
        max_wait_ms: 50,
        enable_auto_batching: true,
    };

    let processor = EmbeddingBatchProcessor::new(config);

    // Test with 25 texts
    let texts: Vec<String> = (0..25).map(|i| format!("Test text number {}", i)).collect();

    // Mock embedding function that simulates API call
    let embed_fn = |batch: Vec<String>| async move {
        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Return mock embeddings
        let embeddings: Vec<Vec<f32>> = batch
            .iter()
            .enumerate()
            .map(|(i, _)| vec![i as f32 * 0.1, i as f32 * 0.2, i as f32 * 0.3])
            .collect();

        Ok(embeddings)
    };

    let result = processor.batch_embed(texts.clone(), embed_fn).await;
    assert!(result.is_ok());

    let embeddings = result.unwrap();
    assert_eq!(embeddings.len(), 25);

    // Check embeddings are correct
    assert_eq!(embeddings[0], vec![0.0, 0.0, 0.0]);
    assert_eq!(embeddings[1], vec![0.1, 0.2, 0.3]);

    // Check statistics
    let stats = processor.get_stats().await;
    assert_eq!(stats.total_texts, 25);
    assert!(stats.total_batches >= 3); // 25 / 10 = 3 batches
    assert!(stats.throughput_texts_per_second > 0.0);
}

#[tokio::test]
async fn test_embedding_batch_empty_texts() {
    let config = EmbeddingBatchConfig::default();
    let processor = EmbeddingBatchProcessor::new(config);

    let texts: Vec<String> = Vec::new();
    let embed_fn = |batch: Vec<String>| async move { Ok(vec![vec![0.1, 0.2, 0.3]; batch.len()]) };

    let result = processor.batch_embed(texts, embed_fn).await.unwrap();
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_embedding_batch_single_text() {
    let config = EmbeddingBatchConfig::default();
    let processor = EmbeddingBatchProcessor::new(config);

    let texts = vec!["Single text".to_string()];
    let embed_fn = |batch: Vec<String>| async move {
        assert_eq!(batch.len(), 1);
        Ok(vec![vec![1.0, 2.0, 3.0]])
    };

    let result = processor.batch_embed(texts, embed_fn).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], vec![1.0, 2.0, 3.0]);
}

#[tokio::test]
async fn test_embedding_batch_large_batch() {
    let config = EmbeddingBatchConfig {
        max_batch_size: 50,
        ..Default::default()
    };
    let processor = EmbeddingBatchProcessor::new(config);

    // Test with 200 texts
    let texts: Vec<String> = (0..200).map(|i| format!("Text {}", i)).collect();

    let embed_fn = |batch: Vec<String>| async move {
        let embeddings: Vec<Vec<f32>> = batch.iter().map(|_| vec![0.5, 0.6, 0.7]).collect();
        Ok(embeddings)
    };

    let result = processor.batch_embed(texts, embed_fn).await.unwrap();
    assert_eq!(result.len(), 200);

    let stats = processor.get_stats().await;
    assert_eq!(stats.total_texts, 200);
    assert_eq!(stats.total_batches, 4); // 200 / 50 = 4 batches
}

#[tokio::test]
async fn test_embedding_batch_stats() {
    let config = EmbeddingBatchConfig::default();
    let processor = EmbeddingBatchProcessor::new(config);

    let texts: Vec<String> = (0..30).map(|i| format!("Text {}", i)).collect();

    let embed_fn = |batch: Vec<String>| async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(vec![vec![1.0, 2.0, 3.0]; batch.len()])
    };

    let _ = processor.batch_embed(texts, embed_fn).await.unwrap();

    let stats = processor.get_stats().await;

    // Validate statistics
    assert_eq!(stats.total_texts, 30);
    assert!(stats.total_batches > 0);
    assert!(stats.average_batch_size > 0.0);
    assert!(stats.throughput_texts_per_second > 0.0);
    assert!(stats.total_processing_time_ms > 0);

    // Test stats formatting
    let report = stats.format_report();
    assert!(report.contains("Total batches"));
    assert!(report.contains("Total texts processed: 30"));
    assert!(report.contains("Throughput"));
}

#[tokio::test]
async fn test_embedding_batch_stats_reset() {
    let config = EmbeddingBatchConfig::default();
    let processor = EmbeddingBatchProcessor::new(config);

    let texts: Vec<String> = vec!["Test".to_string()];
    let embed_fn = |batch: Vec<String>| async move { Ok(vec![vec![1.0]; batch.len()]) };

    // First batch
    let _ = processor
        .batch_embed(texts.clone(), |b| embed_fn(b))
        .await
        .unwrap();

    let stats_before = processor.get_stats().await;
    assert_eq!(stats_before.total_texts, 1);

    // Reset
    processor.reset_stats().await;

    let stats_after = processor.get_stats().await;
    assert_eq!(stats_after.total_texts, 0);
    assert_eq!(stats_after.total_batches, 0);
}

#[tokio::test]
async fn test_embedding_batch_different_sizes() {
    let config = EmbeddingBatchConfig {
        max_batch_size: 15,
        ..Default::default()
    };
    let processor = EmbeddingBatchProcessor::new(config);

    // Test with various sizes
    for size in [1, 5, 10, 15, 20, 50, 100] {
        processor.reset_stats().await;

        let texts: Vec<String> = (0..size).map(|i| format!("Text {}", i)).collect();

        let embed_fn = |batch: Vec<String>| async move { Ok(vec![vec![1.0, 2.0]; batch.len()]) };

        let result = processor
            .batch_embed(texts.clone(), embed_fn)
            .await
            .unwrap();
        assert_eq!(result.len(), size);

        let stats = processor.get_stats().await;
        assert_eq!(stats.total_texts, size);
    }
}

#[test]
fn test_expected_speedup_calculations() {
    use agent_mem_core::embeddings_batch::EmbeddingBatchProcessor;

    // Test expected speedup for different batch sizes
    assert_eq!(EmbeddingBatchProcessor::expected_speedup(1), 1.0);
    assert!(EmbeddingBatchProcessor::expected_speedup(10) > 2.0);
    assert!(EmbeddingBatchProcessor::expected_speedup(50) > 3.5);
    assert!(EmbeddingBatchProcessor::expected_speedup(100) > 4.0);
    assert!(EmbeddingBatchProcessor::expected_speedup(200) >= 5.0);
}

#[test]
fn test_performance_comparison_formatting() {
    let comparison = BatchPerformanceComparison {
        text_count: 100,
        single_method_ms: 5000,
        batch_method_ms: 1000,
        speedup: 5.0,
    };

    let report = comparison.format_report();

    assert!(report.contains("Text count: 100"));
    assert!(report.contains("Single method: 5000ms"));
    assert!(report.contains("Batch method: 1000ms"));
    assert!(report.contains("Speedup: 5.00x"));
    assert!(report.contains("Time saved: 4000ms"));
}

#[tokio::test]
async fn test_concurrent_batch_processing() {
    let config = EmbeddingBatchConfig::default();
    let processor = std::sync::Arc::new(EmbeddingBatchProcessor::new(config));

    // Simulate multiple concurrent batch requests
    let mut handles = Vec::new();

    for batch_id in 0..5 {
        let processor_clone = processor.clone();
        let handle = tokio::spawn(async move {
            let texts: Vec<String> = (0..10)
                .map(|i| format!("Batch {} Text {}", batch_id, i))
                .collect();

            let embed_fn = |batch: Vec<String>| async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Ok(vec![vec![1.0, 2.0]; batch.len()])
            };

            processor_clone.batch_embed(texts, embed_fn).await
        });

        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 10);
    }

    // Check total statistics
    let stats = processor.get_stats().await;
    assert_eq!(stats.total_texts, 50); // 5 batches * 10 texts
}

#[tokio::test]
async fn test_batch_error_handling() {
    let config = EmbeddingBatchConfig::default();
    let processor = EmbeddingBatchProcessor::new(config);

    let texts: Vec<String> = vec!["Test".to_string()];

    // Mock embedding function that returns error
    let embed_fn = |_batch: Vec<String>| async move {
        Err(agent_mem_traits::AgentMemError::internal_error(
            "Simulated API error",
        ))
    };

    let result = processor.batch_embed(texts, embed_fn).await;
    assert!(result.is_err());
}
