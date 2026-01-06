//! 报告生成模块

use crate::framework::{ErrorCase, OverallTestResults, PerformanceMetrics, TestResult};
use anyhow::Result;
use chrono::Utc;
use serde_json;
use std::fs;
use std::path::Path;

/// 报告生成器
pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        Self
    }

    /// 生成测试报告
    pub async fn generate_report(&self, results: &OverallTestResults) -> Result<()> {
        // 生成Markdown报告
        let markdown_report = self.generate_markdown_report(results)?;

        // 生成JSON报告
        let json_report = self.generate_json_report(results)?;

        // 保存报告
        let reports_dir = Path::new("results/reports");
        if !reports_dir.exists() {
            fs::create_dir_all(reports_dir)?;
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let md_path = reports_dir.join(format!("locomo_report_{}.md", timestamp));
        let json_path = reports_dir.join(format!("locomo_report_{}.json", timestamp));

        fs::write(&md_path, markdown_report)?;
        fs::write(&json_path, json_report)?;

        println!("✅ 报告已保存:");
        println!("   - Markdown: {:?}", md_path);
        println!("   - JSON: {:?}", json_path);

        Ok(())
    }

    /// 生成Markdown报告
    fn generate_markdown_report(&self, results: &OverallTestResults) -> Result<String> {
        let mut report = String::new();

        // 标题
        report.push_str("# AgentMem LOCOMO基准测试报告\n\n");
        report.push_str(&format!(
            "**测试日期**: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
        report.push_str(&format!("**测试版本**: AgentMem v0.x.x\n"));
        report.push_str(&format!("**总体得分**: {:.2}%\n\n", results.overall_score));
        report.push_str("---\n\n");

        // 执行摘要
        report.push_str("## 📊 执行摘要\n\n");
        report.push_str(&format!("- **总体得分**: {:.2}%\n", results.overall_score));
        report.push_str(&format!(
            "- **测试耗时**: {:.2}秒\n",
            results.test_duration_secs
        ));
        report.push_str(&format!(
            "- **平均搜索延迟**: {:.2}ms\n",
            results.overall_performance.avg_search_latency_ms
        ));
        report.push_str(&format!(
            "- **平均总延迟**: {:.2}ms\n",
            results.overall_performance.avg_search_latency_ms
                + results.overall_performance.avg_generation_latency_ms
        ));
        report.push_str(&format!(
            "- **平均Token消耗**: {}\n\n",
            results.overall_performance.avg_tokens
        ));

        // 分类得分
        report.push_str("## 📈 分类得分\n\n");
        report.push_str("| 类别 | 得分 | 通过/总数 | 状态 |\n");
        report.push_str("|------|------|----------|------|\n");

        for (category, result) in &results.category_results {
            let status = if result.accuracy_score >= 70.0 {
                "✅ 优秀"
            } else if result.accuracy_score >= 50.0 {
                "⚠️ 中等"
            } else {
                "❌ 需改进"
            };

            report.push_str(&format!(
                "| {} | {:.2}% | {}/{} | {} |\n",
                category, result.accuracy_score, result.passed_tests, result.total_tests, status
            ));
        }

        report.push_str("\n");

        // 详细分类结果
        for (category, result) in &results.category_results {
            report.push_str(&format!("### {}\n\n", self.format_category_name(category)));
            report.push_str(&format!("- **得分**: {:.2}%\n", result.accuracy_score));
            report.push_str(&format!(
                "- **测试用例**: {}/{}\n",
                result.passed_tests, result.total_tests
            ));
            report.push_str(&format!(
                "- **平均搜索延迟**: {:.2}ms\n",
                result.performance.avg_search_latency_ms
            ));
            report.push_str(&format!(
                "- **平均生成延迟**: {:.2}ms\n",
                result.performance.avg_generation_latency_ms
            ));
            report.push_str(&format!(
                "- **平均Token消耗**: {}\n\n",
                result.performance.avg_tokens
            ));

            // 错误案例
            if !result.error_cases.is_empty() {
                report.push_str("#### 错误案例分析\n\n");
                report.push_str("| 问题ID | 问题 | 期望答案 | 实际答案 | 错误原因 |\n");
                report.push_str("|--------|------|----------|----------|----------|\n");

                for error_case in &result.error_cases {
                    report.push_str(&format!(
                        "| {} | {} | {} | {} | {} |\n",
                        error_case.question_id,
                        self.truncate(&error_case.question, 50),
                        self.truncate(&error_case.expected_answer, 50),
                        self.truncate(&error_case.actual_answer, 50),
                        error_case.error_reason
                    ));
                }
                report.push_str("\n");
            }
        }

        // 性能指标
        report.push_str("## ⚡ 性能指标\n\n");
        report.push_str(&format!(
            "- **平均搜索延迟**: {:.2}ms\n",
            results.overall_performance.avg_search_latency_ms
        ));
        report.push_str(&format!(
            "- **P95搜索延迟**: {:.2}ms\n",
            results.overall_performance.p95_search_latency_ms
        ));
        report.push_str(&format!(
            "- **平均总响应时间**: {:.2}ms\n",
            results.overall_performance.avg_search_latency_ms
                + results.overall_performance.avg_generation_latency_ms
        ));
        report.push_str(&format!(
            "- **P95总响应时间**: {:.2}ms\n",
            results.overall_performance.p95_total_latency_ms
        ));
        report.push_str(&format!(
            "- **平均Token消耗**: {}\n\n",
            results.overall_performance.avg_tokens
        ));

        // 平台对比
        report.push_str("## 🔄 平台对比\n\n");
        report.push_str("| 平台 | Single-Hop | Multi-Hop | Open-Domain | Temporal | Overall |\n");
        report.push_str("|------|-----------|-----------|-------------|----------|---------|\n");
        report.push_str("| **AgentMem** | ");

        let single_hop = results
            .category_results
            .get("single_hop")
            .map(|r| r.accuracy_score)
            .unwrap_or(0.0);
        let multi_hop = results
            .category_results
            .get("multi_hop")
            .map(|r| r.accuracy_score)
            .unwrap_or(0.0);
        let open_domain = results
            .category_results
            .get("open_domain")
            .map(|r| r.accuracy_score)
            .unwrap_or(0.0);
        let temporal = results
            .category_results
            .get("temporal")
            .map(|r| r.accuracy_score)
            .unwrap_or(0.0);

        report.push_str(&format!(
            "{:.2}% | {:.2}% | {:.2}% | {:.2}% | {:.2}% |\n",
            single_hop, multi_hop, open_domain, temporal, results.overall_score
        ));
        report.push_str("| Mem0 | 67.13% | 51.15% | 72.93% | 55.51% | 66.88% |\n");
        report.push_str("| MemOS | 78.44% | 64.30% | 55.21% | 73.21% | 73.31% |\n");
        report.push_str("| LangMem | 62.23% | 47.92% | 71.12% | 23.43% | 58.10% |\n");
        report.push_str("| OpenAI | 63.79% | 42.92% | 62.29% | 21.71% | 52.90% |\n\n");

        // 详细分析
        report.push_str("## 📝 详细分析\n\n");
        report.push_str("### 优势\n");
        report.push_str("- TODO: 分析AgentMem的优势\n\n");
        report.push_str("### 劣势\n");
        report.push_str("- TODO: 分析AgentMem的劣势\n\n");
        report.push_str("### 改进建议\n");
        report.push_str("- TODO: 提供改进建议\n\n");

        Ok(report)
    }

    /// 生成JSON报告
    fn generate_json_report(&self, results: &OverallTestResults) -> Result<String> {
        Ok(serde_json::to_string_pretty(results)?)
    }

    /// 格式化类别名称
    fn format_category_name(&self, category: &str) -> String {
        match category {
            "single_hop" => "Single-hop推理",
            "multi_hop" => "Multi-hop推理",
            "temporal" => "Temporal推理",
            "open_domain" => "Open-domain知识",
            "adversarial" => "Adversarial问题",
            _ => category,
        }
        .to_string()
    }

    /// 截断字符串
    fn truncate(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}
