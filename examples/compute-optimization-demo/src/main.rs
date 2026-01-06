//! 计算优化系统演示程序
//!
//! 展示 SIMD 优化、GPU 计算、模型量化、批处理和预计算等功能

use agent_mem_compat::compute_optimization::{
    BatchConfig, ComputeOptimizationConfig, ComputeOptimizationManager, GPUConfig, GPUPlatform,
    InstructionSet, PrecomputeConfig, PrecomputeStrategy, QuantizationConfig,
    QuantizationPrecision, QuantizationStrategy, SIMDConfig,
};
use agent_mem_traits::Result;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🚀 启动计算优化系统演示");

    // 创建计算优化配置
    let config = create_demo_config();

    // 创建计算优化管理器
    let manager = ComputeOptimizationManager::new(config).await?;
    info!("✅ 计算优化管理器创建成功");

    // 演示各种计算优化功能
    demo_simd_optimization(&manager).await?;
    demo_gpu_computation(&manager).await?;
    demo_model_quantization(&manager).await?;
    demo_batch_processing(&manager).await?;
    demo_precomputation(&manager).await?;

    // 启动计算优化系统
    info!("🔄 启动计算优化系统");
    manager.start().await?;

    // 获取统计信息
    info!("📊 获取计算优化统计信息");
    let stats = manager.get_statistics().await?;

    // 显示性能报告
    info!("📈 计算优化统计信息:");
    println!("{}", stats.generate_performance_report());

    // 运行一段时间
    info!("⏱️  运行计算优化系统 30 秒...");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // 停止系统
    info!("⏹️  停止计算优化系统");
    manager.stop().await?;

    info!("🎉 计算优化系统演示完成！");
    Ok(())
}

/// 创建演示配置
fn create_demo_config() -> ComputeOptimizationConfig {
    ComputeOptimizationConfig {
        simd_config: SIMDConfig {
            enabled: true,
            vector_length_threshold: 64,
            instruction_sets: vec![InstructionSet::AVX2, InstructionSet::SSE4],
            parallelism: 4,
        },
        gpu_config: GPUConfig {
            enabled: true, // 启用 GPU 演示
            device_id: 0,
            platform: GPUPlatform::CUDA,
            memory_limit_mb: 4096,
            batch_size: 32,
        },
        quantization_config: QuantizationConfig {
            enabled: true,
            precision: QuantizationPrecision::FP16,
            calibration_size: 500,
            strategy: QuantizationStrategy::Dynamic,
        },
        batch_config: BatchConfig {
            max_batch_size: 64,
            batch_timeout_ms: 50,
            pipeline_depth: 4,
            prefetch_size: 16,
        },
        precompute_config: PrecomputeConfig {
            enabled: true,
            cache_size: 5000,
            ttl_seconds: 1800,
            strategies: vec![
                PrecomputeStrategy::HotQueries,
                PrecomputeStrategy::UserPreferences,
                PrecomputeStrategy::TimePatterns,
            ],
        },
    }
}

/// 演示 SIMD 优化功能
async fn demo_simd_optimization(manager: &ComputeOptimizationManager) -> Result<()> {
    info!("🔧 演示 SIMD 优化功能");

    // 创建测试向量
    let test_vectors = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0],
        vec![0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5],
    ];

    info!("测试 SIMD 向量计算...");
    let start_time = std::time::Instant::now();
    let optimized_vectors = manager.optimize_vector_computation(&test_vectors).await?;
    let processing_time = start_time.elapsed();

    info!(
        "✅ SIMD 优化完成: {} 个向量，处理时间: {:.2}ms",
        optimized_vectors.len(),
        processing_time.as_millis()
    );

    for (i, vector) in optimized_vectors.iter().enumerate() {
        info!(
            "   向量 {}: [{:.2}, {:.2}, {:.2}, ...]",
            i + 1,
            vector[0],
            vector[1],
            vector[2]
        );
    }

    Ok(())
}

/// 演示 GPU 计算功能
async fn demo_gpu_computation(manager: &ComputeOptimizationManager) -> Result<()> {
    info!("🎮 演示 GPU 计算功能");

    // 创建大批量向量用于 GPU 计算
    let mut large_vectors = Vec::new();
    for i in 0..100 {
        let vector: Vec<f32> = (0..256).map(|j| (i * 256 + j) as f32 / 1000.0).collect();
        large_vectors.push(vector);
    }

    info!("测试 GPU 并行计算...");
    let start_time = std::time::Instant::now();
    let gpu_results = manager.optimize_vector_computation(&large_vectors).await?;
    let processing_time = start_time.elapsed();

    info!(
        "✅ GPU 计算完成: {} 个向量，处理时间: {:.2}ms",
        gpu_results.len(),
        processing_time.as_millis()
    );
    info!("   平均向量维度: {}", gpu_results[0].len());
    info!("   GPU 加速比: ~{:.1}x (模拟)", 3.5);

    Ok(())
}

/// 演示模型量化功能
async fn demo_model_quantization(manager: &ComputeOptimizationManager) -> Result<()> {
    info!("🗜️ 演示模型量化功能");

    // 模拟模型数据
    let model_data = vec![0u8; 100 * 1024 * 1024]; // 100MB 模型
    let original_size = model_data.len();

    info!("测试模型量化...");
    info!(
        "原始模型大小: {:.1}MB",
        original_size as f32 / 1024.0 / 1024.0
    );

    let start_time = std::time::Instant::now();
    let quantized_data = manager.quantize_model(&model_data).await?;
    let processing_time = start_time.elapsed();

    let quantized_size = quantized_data.len();
    let compression_ratio = original_size as f32 / quantized_size as f32;
    let size_reduction = (original_size - quantized_size) as f32 / 1024.0 / 1024.0;

    info!("✅ 模型量化完成:");
    info!(
        "   量化后大小: {:.1}MB",
        quantized_size as f32 / 1024.0 / 1024.0
    );
    info!("   压缩比: {:.1}x", compression_ratio);
    info!("   大小减少: {:.1}MB", size_reduction);
    info!("   量化时间: {:.2}ms", processing_time.as_millis());

    Ok(())
}

/// 演示批处理功能
async fn demo_batch_processing(manager: &ComputeOptimizationManager) -> Result<()> {
    info!("📦 演示批处理功能");

    // 创建测试请求
    let requests: Vec<String> = (0..150).map(|i| format!("request_{i}")).collect();

    info!("测试批处理...");
    info!("总请求数: {}", requests.len());

    let start_time = std::time::Instant::now();
    let results = manager
        .batch_process(requests, |batch| {
            // 模拟处理逻辑
            let processed: Vec<String> = batch
                .into_iter()
                .map(|req| format!("processed_{req}"))
                .collect();
            Ok(processed)
        })
        .await?;
    let processing_time = start_time.elapsed();

    info!("✅ 批处理完成:");
    info!("   处理结果数: {}", results.len());
    info!("   处理时间: {:.2}ms", processing_time.as_millis());
    info!(
        "   吞吐量: {:.1} req/s",
        results.len() as f32 / processing_time.as_secs_f32()
    );

    Ok(())
}

/// 演示预计算功能
async fn demo_precomputation(manager: &ComputeOptimizationManager) -> Result<()> {
    info!("💾 演示预计算功能");

    // 测试预计算查询
    let test_queries = vec![
        "machine learning",
        "artificial intelligence",
        "deep learning",
        "neural networks",
        "data science",
    ];

    info!("测试预计算查询...");

    let mut cache_hits = 0;
    let mut cache_misses = 0;

    for query in &test_queries {
        match manager.precompute_query(query).await? {
            Some(result) => {
                cache_hits += 1;
                info!("✅ 缓存命中: '{}' -> 向量维度: {}", query, result.len());
            }
            None => {
                cache_misses += 1;
                info!("❌ 缓存未命中: '{}'", query);
            }
        }
    }

    let hit_rate = cache_hits as f32 / test_queries.len() as f32 * 100.0;
    info!("📊 预计算统计:");
    info!("   缓存命中: {}", cache_hits);
    info!("   缓存未命中: {}", cache_misses);
    info!("   命中率: {:.1}%", hit_rate);

    Ok(())
}
