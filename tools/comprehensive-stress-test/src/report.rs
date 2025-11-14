//! 压测报告生成模块

use anyhow::Result;
use serde_json;
use std::fs;
use std::path::Path;
use tracing::info;

use crate::stats::StressTestStats;

pub struct ReportGenerator {
    output_dir: String,
}

impl ReportGenerator {
    pub fn new(output_dir: &str) -> Self {
        Self {
            output_dir: output_dir.to_string(),
        }
    }

    /// 保存单个场景的统计数据
    pub async fn save_scenario_stats(&self, scenario_name: &str, stats: &StressTestStats) -> Result<()> {
        let file_path = format!("{}/{}.json", self.output_dir, scenario_name);
        let json = serde_json::to_string_pretty(stats)?;
        fs::write(&file_path, json)?;
        info!("场景统计已保存: {}", file_path);
        Ok(())
    }

    /// 生成综合报告
    pub async fn generate_comprehensive_report(&self, all_stats: &[(&str, StressTestStats)]) -> Result<()> {
        let report_path = format!("{}/comprehensive-report.md", self.output_dir);
        let mut report = String::new();

        report.push_str("# AgentMem 综合压测报告\n\n");
        report.push_str(&format!("**生成时间**: {}\n\n", chrono::Utc::now().to_rfc3339()));
        report.push_str("---\n\n");

        // 总体摘要
        report.push_str("## 📊 总体摘要\n\n");
        report.push_str("| 场景 | 总操作数 | 成功率 | 吞吐量 (ops/s) | P95延迟 (ms) | P99延迟 (ms) |\n");
        report.push_str("|------|----------|--------|----------------|--------------|-------------|\n");

        for (name, stats) in all_stats {
            let success_rate = if stats.total_operations > 0 {
                (stats.successful_operations as f64 / stats.total_operations as f64) * 100.0
            } else {
                0.0
            };

            report.push_str(&format!(
                "| {} | {} | {:.2}% | {:.2} | {:.2} | {:.2} |\n",
                name, stats.total_operations, success_rate, stats.throughput, 
                stats.latency_p95, stats.latency_p99
            ));
        }

        report.push_str("\n---\n\n");

        // 各场景详细报告
        for (name, stats) in all_stats {
            report.push_str(&self.generate_scenario_section(name, stats));
        }

        // 瓶颈分析
        report.push_str(&self.generate_bottleneck_analysis(all_stats));

        // 优化建议
        report.push_str(&self.generate_recommendations(all_stats));

        fs::write(&report_path, report)?;
        info!("综合报告已生成: {}", report_path);

        Ok(())
    }

    fn generate_scenario_section(&self, name: &str, stats: &StressTestStats) -> String {
        let mut section = String::new();

        section.push_str(&format!("## 场景: {}\n\n", name));

        section.push_str("### 基本统计\n\n");
        section.push_str(&format!("- **总操作数**: {}\n", stats.total_operations));
        section.push_str(&format!("- **成功操作**: {}\n", stats.successful_operations));
        section.push_str(&format!("- **失败操作**: {}\n", stats.failed_operations));
        section.push_str(&format!("- **运行时间**: {:.2}秒\n", stats.duration_seconds));
        section.push_str(&format!("- **吞吐量**: {:.2} ops/s\n\n", stats.throughput));

        section.push_str("### 延迟分布\n\n");
        section.push_str(&format!("- **最小延迟**: {:.2} ms\n", stats.latency_min));
        section.push_str(&format!("- **平均延迟**: {:.2} ms\n", stats.latency_mean));
        section.push_str(&format!("- **P50 延迟**: {:.2} ms\n", stats.latency_p50));
        section.push_str(&format!("- **P90 延迟**: {:.2} ms\n", stats.latency_p90));
        section.push_str(&format!("- **P95 延迟**: {:.2} ms\n", stats.latency_p95));
        section.push_str(&format!("- **P99 延迟**: {:.2} ms\n", stats.latency_p99));
        section.push_str(&format!("- **最大延迟**: {:.2} ms\n\n", stats.latency_max));

        section.push_str("### 资源使用\n\n");
        section.push_str(&format!("- **平均 CPU**: {:.2}%\n", stats.avg_cpu_usage));
        section.push_str(&format!("- **峰值 CPU**: {:.2}%\n", stats.peak_cpu_usage));
        section.push_str(&format!("- **平均内存**: {:.2} MB\n", stats.avg_memory_mb));
        section.push_str(&format!("- **峰值内存**: {:.2} MB\n\n", stats.peak_memory_mb));

        section.push_str("---\n\n");

        section
    }

    fn generate_bottleneck_analysis(&self, all_stats: &[(&str, StressTestStats)]) -> String {
        let mut analysis = String::new();

        analysis.push_str("## 🔍 瓶颈分析\n\n");

        // CPU 瓶颈分析
        analysis.push_str("### CPU 瓶颈\n\n");
        for (name, stats) in all_stats {
            if stats.peak_cpu_usage > 80.0 {
                analysis.push_str(&format!(
                    "- ⚠️ **{}**: 峰值 CPU 使用率 {:.2}%，可能存在 CPU 瓶颈\n",
                    name, stats.peak_cpu_usage
                ));
            }
        }
        analysis.push_str("\n");

        // 延迟瓶颈分析
        analysis.push_str("### 延迟瓶颈\n\n");
        for (name, stats) in all_stats {
            if stats.latency_p95 > 50.0 {
                analysis.push_str(&format!(
                    "- ⚠️ **{}**: P95 延迟 {:.2}ms，超过目标值 30ms\n",
                    name, stats.latency_p95
                ));
            }
        }
        analysis.push_str("\n");

        // 吞吐量瓶颈分析
        analysis.push_str("### 吞吐量瓶颈\n\n");
        for (name, stats) in all_stats {
            if stats.throughput < 1000.0 && !name.contains("intelligence") {
                analysis.push_str(&format!(
                    "- ⚠️ **{}**: 吞吐量 {:.2} ops/s，低于预期\n",
                    name, stats.throughput
                ));
            }
        }
        analysis.push_str("\n");

        analysis.push_str("---\n\n");

        analysis
    }

    fn generate_recommendations(&self, all_stats: &[(&str, StressTestStats)]) -> String {
        let mut recommendations = String::new();

        recommendations.push_str("## 💡 优化建议\n\n");

        for (name, stats) in all_stats {
            recommendations.push_str(&format!("### {}\n\n", name));

            if stats.peak_cpu_usage > 80.0 {
                recommendations.push_str("- 🔧 **CPU 优化**: 考虑使用更高效的算法或并行处理\n");
            }

            if stats.latency_p95 > 50.0 {
                recommendations.push_str("- 🔧 **延迟优化**: 增加缓存、优化数据库查询、使用连接池\n");
            }

            if stats.peak_memory_mb > 1000.0 {
                recommendations.push_str("- 🔧 **内存优化**: 实现内存池、减少大对象分配、优化数据结构\n");
            }

            if stats.error_rate > 0.01 {
                recommendations.push_str(&format!(
                    "- 🔧 **错误率优化**: 当前错误率 {:.2}%，需要改进错误处理和重试机制\n",
                    stats.error_rate * 100.0
                ));
            }

            recommendations.push_str("\n");
        }

        recommendations.push_str("---\n\n");
        recommendations.push_str("## 📝 总结\n\n");
        recommendations.push_str("压测完成，请根据以上分析和建议进行系统优化。\n\n");

        recommendations
    }

    /// 从目录加载结果并生成报告
    pub async fn generate_from_directory(&self, results_dir: &str) -> Result<()> {
        let mut all_stats = Vec::new();

        for entry in fs::read_dir(results_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let stats: StressTestStats = serde_json::from_str(&content)?;
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                all_stats.push((name.to_string(), stats));
            }
        }

        // 转换为引用
        let stats_refs: Vec<(&str, StressTestStats)> = all_stats.iter()
            .map(|(name, stats)| (name.as_str(), stats.clone()))
            .collect();

        self.generate_comprehensive_report(&stats_refs).await?;

        Ok(())
    }
}

