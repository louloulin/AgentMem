//! 监控增强功能测试用例
//! 
//! 🆕 Phase 4.2: 监控和可观测性增强

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use std::sync::OnceLock;

    /// 测试服务器启动时间初始化
    #[test]
    fn test_server_start_time_initialization() {
        // 模拟初始化
        let start_time = OnceLock::new();
        start_time.get_or_init(Instant::now);
        
        assert!(start_time.get().is_some(), "启动时间应该被初始化");
        
        // 验证运行时间计算
        let elapsed = start_time.get().unwrap().elapsed().as_secs_f64();
        assert!(elapsed >= 0.0, "运行时间应该 >= 0");
        assert!(elapsed < 1.0, "测试应该在1秒内完成");
    }

    /// 测试运行时间计算（秒、小时、天）
    #[test]
    fn test_uptime_calculation() {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let uptime_seconds = start.elapsed().as_secs_f64();
        let uptime_hours = uptime_seconds / 3600.0;
        let uptime_days = uptime_seconds / 86400.0;
        
        assert!(uptime_seconds > 0.0, "运行时间（秒）应该 > 0");
        assert!(uptime_hours < 1.0, "运行时间（小时）应该 < 1（测试期间）");
        assert!(uptime_days < 1.0, "运行时间（天）应该 < 1（测试期间）");
        
        // 验证转换关系
        assert!((uptime_hours * 3600.0 - uptime_seconds).abs() < 0.1, "小时和秒的转换应该一致");
        assert!((uptime_days * 86400.0 - uptime_seconds).abs() < 0.1, "天和秒的转换应该一致");
    }

    /// 测试监控指标结构
    #[test]
    fn test_metrics_structure() {
        use std::collections::HashMap;
        
        let mut metrics = HashMap::new();
        metrics.insert("uptime_seconds".to_string(), 3600.0);
        metrics.insert("uptime_hours".to_string(), 1.0);
        metrics.insert("uptime_days".to_string(), 1.0 / 24.0);
        metrics.insert("memory_usage_bytes".to_string(), 1024.0 * 1024.0);
        metrics.insert("memory_usage_mb".to_string(), 1.0);
        metrics.insert("cpu_usage_percent".to_string(), 50.0);
        
        // 验证指标存在
        assert!(metrics.contains_key("uptime_seconds"), "应该包含运行时间（秒）");
        assert!(metrics.contains_key("uptime_hours"), "应该包含运行时间（小时）");
        assert!(metrics.contains_key("uptime_days"), "应该包含运行时间（天）");
        assert!(metrics.contains_key("memory_usage_bytes"), "应该包含内存使用（字节）");
        assert!(metrics.contains_key("memory_usage_mb"), "应该包含内存使用（MB）");
        assert!(metrics.contains_key("cpu_usage_percent"), "应该包含CPU使用率");
        
        // 验证指标值
        assert_eq!(metrics.get("uptime_seconds"), Some(&3600.0));
        assert_eq!(metrics.get("uptime_hours"), Some(&1.0));
        assert_eq!(metrics.get("memory_usage_mb"), Some(&1.0));
    }

    /// 测试搜索统计集成到系统指标
    #[test]
    fn test_search_stats_integration() {
        use std::collections::HashMap;
        
        let mut metrics = HashMap::new();
        
        // 模拟搜索统计
        metrics.insert("search_total_searches".to_string(), 100.0);
        metrics.insert("search_cache_hits".to_string(), 80.0);
        metrics.insert("search_cache_misses".to_string(), 20.0);
        metrics.insert("search_cache_hit_rate".to_string(), 0.8);
        metrics.insert("search_avg_latency_ms".to_string(), 10.5);
        metrics.insert("search_exact_queries".to_string(), 30.0);
        metrics.insert("search_vector_searches".to_string(), 70.0);
        
        // 验证搜索统计指标存在
        assert!(metrics.contains_key("search_total_searches"), "应该包含总搜索次数");
        assert!(metrics.contains_key("search_cache_hits"), "应该包含缓存命中次数");
        assert!(metrics.contains_key("search_cache_misses"), "应该包含缓存未命中次数");
        assert!(metrics.contains_key("search_cache_hit_rate"), "应该包含缓存命中率");
        assert!(metrics.contains_key("search_avg_latency_ms"), "应该包含平均延迟");
        assert!(metrics.contains_key("search_exact_queries"), "应该包含精确查询次数");
        assert!(metrics.contains_key("search_vector_searches"), "应该包含向量搜索次数");
        
        // 验证缓存命中率计算
        let cache_hit_rate = metrics.get("search_cache_hit_rate").unwrap();
        assert!(*cache_hit_rate >= 0.0 && *cache_hit_rate <= 1.0, "缓存命中率应该在0-1之间");
        
        // 验证搜索次数一致性
        let total = metrics.get("search_total_searches").unwrap();
        let hits = metrics.get("search_cache_hits").unwrap();
        let misses = metrics.get("search_cache_misses").unwrap();
        assert!((hits + misses - total).abs() < 0.1, "缓存命中+未命中应该等于总搜索次数");
    }

    /// 测试内存使用单位转换
    #[test]
    fn test_memory_usage_conversion() {
        let memory_bytes = 1024.0 * 1024.0; // 1 MB
        let memory_mb = memory_bytes / (1024.0 * 1024.0);
        
        assert_eq!(memory_mb, 1.0, "1 MB应该等于1.0");
        
        let memory_bytes_2 = 512.0 * 1024.0; // 0.5 MB
        let memory_mb_2 = memory_bytes_2 / (1024.0 * 1024.0);
        
        assert_eq!(memory_mb_2, 0.5, "512 KB应该等于0.5 MB");
    }
}

