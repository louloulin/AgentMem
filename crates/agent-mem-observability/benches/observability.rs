//! Benchmarks for observability operations

use agent_mem_observability::{MetricsRegistry, PerformanceAnalyzer};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn bench_metrics_recording(c: &mut Criterion) {
    let registry = MetricsRegistry::new();
    let collector = registry.collector();

    c.bench_function("metrics_record_request", |b| {
        b.iter(|| {
            let collector = collector.clone();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                collector
                    .record_request(black_box("GET"), black_box("/api/test"), black_box(200))
                    .await;
            });
        });
    });

    c.bench_function("metrics_record_error", |b| {
        b.iter(|| {
            let collector = collector.clone();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                collector.record_error(black_box("test_error")).await;
            });
        });
    });

    c.bench_function("metrics_record_duration", |b| {
        b.iter(|| {
            let collector = collector.clone();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                collector
                    .record_request_duration(
                        black_box("GET"),
                        black_box("/api/test"),
                        black_box(0.05),
                    )
                    .await;
            });
        });
    });
}

fn bench_performance_tracking(c: &mut Criterion) {
    c.bench_function("performance_record_operation", |b| {
        b.iter(|| {
            let analyzer = PerformanceAnalyzer::new();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                analyzer
                    .record_operation(black_box("test_op"), black_box(Duration::from_millis(10)))
                    .await;
            });
        });
    });

    c.bench_function("performance_start_operation", |b| {
        let analyzer = PerformanceAnalyzer::new();
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _tracker = analyzer.start_operation(black_box("test_op"));
            });
        });
    });
}

fn bench_metrics_gathering(c: &mut Criterion) {
    let registry = MetricsRegistry::new();
    let collector = registry.collector();

    // Pre-populate with some metrics
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        for _i in 0..100 {
            collector.record_request("GET", "/api/test", 200).await;
            collector
                .record_request_duration("GET", "/api/test", 0.05)
                .await;
        }
    });

    c.bench_function("metrics_gather", |b| {
        b.iter(|| {
            let _metrics = registry.gather();
        });
    });
}

fn bench_performance_reporting(c: &mut Criterion) {
    let analyzer = PerformanceAnalyzer::new();

    // Pre-populate with some data
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        for i in 0..100 {
            analyzer
                .record_operation("test_op", Duration::from_millis(10 + i))
                .await;
        }
    });

    c.bench_function("performance_get_report", |b| {
        b.iter(|| {
            let analyzer_clone = &analyzer;
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _report = analyzer_clone.get_report().await;
            });
        });
    });

    c.bench_function("performance_get_stats", |b| {
        b.iter(|| {
            let analyzer_clone = &analyzer;
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _stats = analyzer_clone.get_stats(black_box("test_op")).await;
            });
        });
    });
}

criterion_group!(
    benches,
    bench_metrics_recording,
    bench_performance_tracking,
    bench_metrics_gathering,
    bench_performance_reporting
);
criterion_main!(benches);
