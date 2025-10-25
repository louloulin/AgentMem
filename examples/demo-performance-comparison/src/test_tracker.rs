//! TestTracker - 测试跟踪和报告工具
//!
//! 对标MIRIX的TestTracker类，提供：
//! 1. 测试和子测试跟踪
//! 2. 成功/失败状态管理
//! 3. 详细测试报告
//! 4. 统计摘要
//!
//! 真实实现，对标MIRIX的test_memory.py中的TestTracker

use colored::*;
use serde::{Deserialize, Serialize};

/// 测试状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Running,
    Passed,
    Failed,
}

/// 子测试
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTest {
    pub name: String,
    pub status: TestStatus,
    pub error: Option<String>,
}

/// 测试
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Test {
    pub name: String,
    pub description: String,
    pub status: TestStatus,
    pub error: Option<String>,
    pub subtests: Vec<SubTest>,
}

/// 测试统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub total_subtests: usize,
    pub passed_subtests: usize,
    pub failed_subtests: usize,
}

/// 测试跟踪器
pub struct TestTracker {
    tests: Vec<Test>,
    current_test: Option<Test>,
}

impl TestTracker {
    /// 创建新的测试跟踪器
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            current_test: None,
        }
    }

    /// 开始新测试
    pub fn start_test(&mut self, test_name: impl Into<String>, description: impl Into<String>) {
        self.current_test = Some(Test {
            name: test_name.into(),
            description: description.into(),
            status: TestStatus::Running,
            error: None,
            subtests: Vec::new(),
        });

        if let Some(test) = &self.current_test {
            println!("\n{}", format!("🚀 Starting: {}", test.name).cyan().bold());
            if !test.description.is_empty() {
                println!("   Description: {}", test.description.bright_black());
            }
        }
    }

    /// 开始子测试
    pub fn start_subtest(&mut self, subtest_name: impl Into<String>) -> Option<usize> {
        if let Some(test) = &mut self.current_test {
            let subtest = SubTest {
                name: subtest_name.into(),
                status: TestStatus::Running,
                error: None,
            };

            println!("  ▶️  {}", subtest.name);
            test.subtests.push(subtest);
            Some(test.subtests.len() - 1)
        } else {
            println!("{}", "Warning: No current test to add subtest to".yellow());
            None
        }
    }

    /// 标记子测试通过
    pub fn pass_subtest(&mut self, subtest_index: Option<usize>, message: &str) {
        if let Some(test) = &mut self.current_test {
            let index = subtest_index.unwrap_or(test.subtests.len().saturating_sub(1));

            if let Some(subtest) = test.subtests.get_mut(index) {
                subtest.status = TestStatus::Passed;
                let msg = if message.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", message)
                };
                println!("  {} {}{}", "✅".green(), subtest.name, msg.bright_black());
            }
        }
    }

    /// 标记子测试失败
    pub fn fail_subtest(&mut self, error: &str, subtest_index: Option<usize>) {
        if let Some(test) = &mut self.current_test {
            let index = subtest_index.unwrap_or(test.subtests.len().saturating_sub(1));

            if let Some(subtest) = test.subtests.get_mut(index) {
                subtest.status = TestStatus::Failed;
                subtest.error = Some(error.to_string());
                println!(
                    "  {} {} - ERROR: {}",
                    "❌".red(),
                    subtest.name,
                    error.red()
                );
            }
        }
    }

    /// 标记测试通过
    pub fn pass_test(&mut self, message: &str) {
        if let Some(test) = &mut self.current_test {
            test.status = TestStatus::Passed;
            let msg = if message.is_empty() {
                String::new()
            } else {
                format!(" - {}", message)
            };
            println!("{} PASSED: {}{}", "✅".green(), test.name.green().bold(), msg);

            self.tests.push(test.clone());
            self.current_test = None;
        }
    }

    /// 标记测试失败
    pub fn fail_test(&mut self, error: &str) {
        if let Some(test) = &mut self.current_test {
            test.status = TestStatus::Failed;
            test.error = Some(error.to_string());
            println!(
                "{} FAILED: {} - ERROR: {}",
                "❌".red(),
                test.name.red().bold(),
                error.red()
            );

            self.tests.push(test.clone());
            self.current_test = None;
        }
    }

    /// 获取测试摘要
    pub fn get_summary(&self) -> TestSummary {
        let total_tests = self.tests.len();
        let passed_tests = self
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Passed)
            .count();
        let failed_tests = self
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed)
            .count();

        let total_subtests = self.tests.iter().map(|t| t.subtests.len()).sum();
        let passed_subtests = self
            .tests
            .iter()
            .flat_map(|t| &t.subtests)
            .filter(|s| s.status == TestStatus::Passed)
            .count();
        let failed_subtests = self
            .tests
            .iter()
            .flat_map(|t| &t.subtests)
            .filter(|s| s.status == TestStatus::Failed)
            .count();

        TestSummary {
            total_tests,
            passed_tests,
            failed_tests,
            total_subtests,
            passed_subtests,
            failed_subtests,
        }
    }

    /// 打印测试摘要
    pub fn print_summary(&self) {
        println!("\n{}", "═".repeat(70).blue());
        println!(
            "{}",
            "                        📊 测试摘要                         ".blue().bold()
        );
        println!("{}", "═".repeat(70).blue());

        let summary = self.get_summary();

        println!("\n{}", "主测试统计:".yellow().bold());
        println!("  - 总测试数: {}", summary.total_tests);
        println!(
            "  - {} 通过: {}",
            "✅".green(),
            summary.passed_tests.to_string().green().bold()
        );
        println!(
            "  - {} 失败: {}",
            "❌".red(),
            summary.failed_tests.to_string().red().bold()
        );

        if summary.total_tests > 0 {
            let pass_rate = (summary.passed_tests as f64 / summary.total_tests as f64) * 100.0;
            println!(
                "  - 通过率: {:.1}%",
                if pass_rate >= 90.0 {
                    format!("{:.1}%", pass_rate).green().bold()
                } else if pass_rate >= 70.0 {
                    format!("{:.1}%", pass_rate).yellow().bold()
                } else {
                    format!("{:.1}%", pass_rate).red().bold()
                }
            );
        }

        println!("\n{}", "子测试统计:".yellow().bold());
        println!("  - 总子测试数: {}", summary.total_subtests);
        println!(
            "  - {} 通过: {}",
            "✅".green(),
            summary.passed_subtests.to_string().green().bold()
        );
        println!(
            "  - {} 失败: {}",
            "❌".red(),
            summary.failed_subtests.to_string().red().bold()
        );

        // 打印失败测试详情
        let failed_tests: Vec<&Test> = self
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed)
            .collect();

        if !failed_tests.is_empty() {
            println!("\n{}", "失败测试详情:".red().bold());
            for test in failed_tests {
                println!("  {} {}", "❌".red(), test.name.red().bold());
                if let Some(error) = &test.error {
                    println!("     Error: {}", error.red());
                }

                for subtest in test.subtests.iter().filter(|s| s.status == TestStatus::Failed) {
                    println!("    {} {}", "❌".red(), subtest.name);
                    if let Some(error) = &subtest.error {
                        println!("       Error: {}", error.red());
                    }
                }
            }
        }

        println!("\n{}", "═".repeat(70).blue());

        // 最终状态
        if summary.failed_tests == 0 && summary.failed_subtests == 0 {
            println!(
                "{}",
                "            ✅ 所有测试通过！✅            ".green().bold()
            );
        } else {
            println!(
                "{}",
                "            ⚠️  存在测试失败 ⚠️           ".yellow().bold()
            );
        }

        println!("{}", "═".repeat(70).blue());
    }

    /// 获取所有测试
    pub fn get_tests(&self) -> &[Test] {
        &self.tests
    }
}

impl Default for TestTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_basic() {
        let mut tracker = TestTracker::new();

        tracker.start_test("Test 1", "First test");
        tracker.start_subtest("Subtest 1");
        tracker.pass_subtest(None, "Success");
        tracker.pass_test("Test completed");

        let summary = tracker.get_summary();
        assert_eq!(summary.total_tests, 1);
        assert_eq!(summary.passed_tests, 1);
        assert_eq!(summary.failed_tests, 0);
    }

    #[test]
    fn test_tracker_failures() {
        let mut tracker = TestTracker::new();

        tracker.start_test("Test 2", "Second test");
        tracker.start_subtest("Subtest 1");
        tracker.fail_subtest("Test error", None);
        tracker.fail_test("Test failed");

        let summary = tracker.get_summary();
        assert_eq!(summary.total_tests, 1);
        assert_eq!(summary.passed_tests, 0);
        assert_eq!(summary.failed_tests, 1);
    }
}

