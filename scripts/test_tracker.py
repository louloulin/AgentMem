#!/usr/bin/env python3
"""
AgentMem Test Tracker
参考 MIRIX 的 TestTracker 实现

功能:
1. 跟踪测试执行状态
2. 记录测试结果
3. 生成测试报告
4. 支持子测试
5. 性能统计

使用示例:
    tracker = TestTracker()
    tracker.start_test("Memory Engine Tests", "测试记忆引擎核心功能")
    
    # 运行子测试
    idx = tracker.start_subtest("测试记忆存储")
    try:
        # 测试代码
        tracker.pass_subtest(idx, "成功存储 100 条记忆")
    except Exception as e:
        tracker.fail_subtest(e, idx)
    
    tracker.pass_test("所有测试通过")
    tracker.print_summary()
"""

import time
import json
from datetime import datetime
from typing import List, Dict, Optional, Any
from dataclasses import dataclass, field, asdict
from enum import Enum


class TestStatus(Enum):
    """测试状态枚举"""
    NOT_STARTED = "not_started"
    RUNNING = "running"
    PASSED = "passed"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class SubtestResult:
    """子测试结果"""
    name: str
    status: TestStatus = TestStatus.RUNNING
    error: Optional[str] = None
    message: Optional[str] = None
    duration: float = 0.0
    start_time: float = field(default_factory=time.time)
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典"""
        return {
            'name': self.name,
            'status': self.status.value,
            'error': self.error,
            'message': self.message,
            'duration': self.duration,
        }


@dataclass
class TestResult:
    """测试结果"""
    name: str
    description: str = ""
    status: TestStatus = TestStatus.RUNNING
    error: Optional[str] = None
    message: Optional[str] = None
    subtests: List[SubtestResult] = field(default_factory=list)
    duration: float = 0.0
    start_time: float = field(default_factory=time.time)
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典"""
        return {
            'name': self.name,
            'description': self.description,
            'status': self.status.value,
            'error': self.error,
            'message': self.message,
            'subtests': [st.to_dict() for st in self.subtests],
            'duration': self.duration,
        }


class TestTracker:
    """测试跟踪器 - 参考 MIRIX 实现"""
    
    def __init__(self):
        self.tests: List[TestResult] = []
        self.current_test: Optional[TestResult] = None
        self.start_time = time.time()
        
    def start_test(self, test_name: str, description: str = "") -> None:
        """开始一个新测试"""
        self.current_test = TestResult(
            name=test_name,
            description=description,
            status=TestStatus.RUNNING,
            start_time=time.time()
        )
        
        print(f"\n🚀 Starting: {test_name}")
        if description:
            print(f"   Description: {description}")
    
    def start_subtest(self, subtest_name: str) -> int:
        """开始一个子测试"""
        if not self.current_test:
            print("⚠️  Warning: No current test to add subtest to")
            return -1
        
        subtest = SubtestResult(
            name=subtest_name,
            status=TestStatus.RUNNING,
            start_time=time.time()
        )
        self.current_test.subtests.append(subtest)
        
        print(f"  ▶️  {subtest_name}")
        return len(self.current_test.subtests) - 1
    
    def pass_subtest(self, subtest_index: Optional[int] = None, message: str = "") -> None:
        """标记子测试为通过"""
        if not self.current_test:
            return
        
        if subtest_index is None:
            subtest_index = len(self.current_test.subtests) - 1
        
        if 0 <= subtest_index < len(self.current_test.subtests):
            subtest = self.current_test.subtests[subtest_index]
            subtest.status = TestStatus.PASSED
            subtest.duration = time.time() - subtest.start_time
            subtest.message = message
            
            msg_suffix = f" - {message}" if message else ""
            print(f"  ✅ {subtest.name}{msg_suffix}")
    
    def fail_subtest(self, error: Exception, subtest_index: Optional[int] = None) -> None:
        """标记子测试为失败"""
        if not self.current_test:
            return
        
        if subtest_index is None:
            subtest_index = len(self.current_test.subtests) - 1
        
        if 0 <= subtest_index < len(self.current_test.subtests):
            subtest = self.current_test.subtests[subtest_index]
            subtest.status = TestStatus.FAILED
            subtest.duration = time.time() - subtest.start_time
            subtest.error = str(error)
            
            print(f"  ❌ {subtest.name} - ERROR: {error}")
    
    def skip_subtest(self, subtest_index: Optional[int] = None, reason: str = "") -> None:
        """跳过子测试"""
        if not self.current_test:
            return
        
        if subtest_index is None:
            subtest_index = len(self.current_test.subtests) - 1
        
        if 0 <= subtest_index < len(self.current_test.subtests):
            subtest = self.current_test.subtests[subtest_index]
            subtest.status = TestStatus.SKIPPED
            subtest.duration = time.time() - subtest.start_time
            subtest.message = reason
            
            print(f"  ⏭️  {subtest.name} - SKIPPED: {reason}")
    
    def pass_test(self, message: str = "") -> None:
        """标记当前测试为通过"""
        if not self.current_test:
            return
        
        self.current_test.status = TestStatus.PASSED
        self.current_test.duration = time.time() - self.current_test.start_time
        self.current_test.message = message
        
        msg_suffix = f" - {message}" if message else ""
        print(f"✅ PASSED: {self.current_test.name}{msg_suffix}")
        
        self.tests.append(self.current_test)
        self.current_test = None
    
    def fail_test(self, error: Exception) -> None:
        """标记当前测试为失败"""
        if not self.current_test:
            return
        
        self.current_test.status = TestStatus.FAILED
        self.current_test.duration = time.time() - self.current_test.start_time
        self.current_test.error = str(error)
        
        print(f"❌ FAILED: {self.current_test.name} - ERROR: {error}")
        
        self.tests.append(self.current_test)
        self.current_test = None
    
    def skip_test(self, reason: str = "") -> None:
        """跳过当前测试"""
        if not self.current_test:
            return
        
        self.current_test.status = TestStatus.SKIPPED
        self.current_test.duration = time.time() - self.current_test.start_time
        self.current_test.message = reason
        
        print(f"⏭️  SKIPPED: {self.current_test.name} - {reason}")
        
        self.tests.append(self.current_test)
        self.current_test = None
    
    def get_summary(self) -> Dict[str, Any]:
        """获取测试摘要"""
        total_tests = len(self.tests)
        passed_tests = len([t for t in self.tests if t.status == TestStatus.PASSED])
        failed_tests = len([t for t in self.tests if t.status == TestStatus.FAILED])
        skipped_tests = len([t for t in self.tests if t.status == TestStatus.SKIPPED])
        
        total_subtests = sum(len(t.subtests) for t in self.tests)
        passed_subtests = sum(
            len([s for s in t.subtests if s.status == TestStatus.PASSED]) 
            for t in self.tests
        )
        failed_subtests = sum(
            len([s for s in t.subtests if s.status == TestStatus.FAILED]) 
            for t in self.tests
        )
        skipped_subtests = sum(
            len([s for s in t.subtests if s.status == TestStatus.SKIPPED]) 
            for t in self.tests
        )
        
        total_duration = time.time() - self.start_time
        
        return {
            'total_tests': total_tests,
            'passed_tests': passed_tests,
            'failed_tests': failed_tests,
            'skipped_tests': skipped_tests,
            'total_subtests': total_subtests,
            'passed_subtests': passed_subtests,
            'failed_subtests': failed_subtests,
            'skipped_subtests': skipped_subtests,
            'total_duration': total_duration,
            'tests': [t.to_dict() for t in self.tests]
        }
    
    def print_summary(self) -> Dict[str, Any]:
        """打印测试摘要 - 参考 MIRIX 格式"""
        summary = self.get_summary()
        
        print("\n" + "=" * 80)
        print("🏁 TEST EXECUTION SUMMARY")
        print("=" * 80)
        
        print(f"\n📊 OVERALL RESULTS:")
        print(f"   Total Tests: {summary['total_tests']}")
        print(f"   ✅ Passed Tests: {summary['passed_tests']}")
        if summary['failed_tests'] > 0:
            print(f"   ❌ Failed Tests: {summary['failed_tests']}")
        if summary['skipped_tests'] > 0:
            print(f"   ⏭️  Skipped Tests: {summary['skipped_tests']}")
        
        if summary['total_tests'] > 0:
            success_rate = (summary['passed_tests'] / summary['total_tests']) * 100
            print(f"   📈 Success Rate: {success_rate:.1f}%")
        
        if summary['total_subtests'] > 0:
            print(f"\n🔍 SUBTEST DETAILS:")
            print(f"   Total Subtests: {summary['total_subtests']}")
            print(f"   ✅ Passed Subtests: {summary['passed_subtests']}")
            if summary['failed_subtests'] > 0:
                print(f"   ❌ Failed Subtests: {summary['failed_subtests']}")
            if summary['skipped_subtests'] > 0:
                print(f"   ⏭️  Skipped Subtests: {summary['skipped_subtests']}")
            
            subtest_success_rate = (summary['passed_subtests'] / summary['total_subtests']) * 100
            print(f"   📈 Subtest Success Rate: {subtest_success_rate:.1f}%")
        
        # 性能统计
        print(f"\n⏱️  PERFORMANCE:")
        print(f"   Total Duration: {summary['total_duration']:.2f}s")
        if summary['total_tests'] > 0:
            avg_duration = summary['total_duration'] / summary['total_tests']
            print(f"   Average Test Duration: {avg_duration:.2f}s")
        
        # 显示失败的测试详情
        failed_tests = [t for t in self.tests if t.status == TestStatus.FAILED]
        if failed_tests:
            print(f"\n❌ FAILED TESTS DETAILS:")
            for i, test in enumerate(failed_tests, 1):
                print(f"   {i}. {test.name}")
                print(f"      Error: {test.error}")
                
                # 显示失败的子测试
                failed_subtests = [s for s in test.subtests if s.status == TestStatus.FAILED]
                if failed_subtests:
                    print(f"      Failed Subtests:")
                    for subtest in failed_subtests:
                        print(f"        - {subtest.name}: {subtest.error}")
        
        # 显示通过的测试摘要
        passed_tests = [t for t in self.tests if t.status == TestStatus.PASSED]
        if passed_tests:
            print(f"\n✅ PASSED TESTS:")
            for i, test in enumerate(passed_tests, 1):
                subtest_count = len(test.subtests)
                passed_subtest_count = len([s for s in test.subtests if s.status == TestStatus.PASSED])
                print(f"   {i}. {test.name} ({passed_subtest_count}/{subtest_count} subtests passed)")
        
        print("\n" + "=" * 80)
        
        return summary
    
    def save_report(self, filename: str = "test_report.json") -> None:
        """保存测试报告到 JSON 文件"""
        summary = self.get_summary()
        
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(summary, f, indent=2, ensure_ascii=False)
        
        print(f"\n📄 Test report saved to: {filename}")
    
    def save_html_report(self, filename: str = "test_report.html") -> None:
        """保存 HTML 格式的测试报告"""
        summary = self.get_summary()
        
        html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>AgentMem Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background: #2c3e50; color: white; padding: 20px; border-radius: 5px; }}
        .summary {{ background: #ecf0f1; padding: 15px; margin: 20px 0; border-radius: 5px; }}
        .test {{ margin: 10px 0; padding: 10px; border-left: 4px solid #3498db; }}
        .passed {{ border-left-color: #27ae60; }}
        .failed {{ border-left-color: #e74c3c; }}
        .skipped {{ border-left-color: #95a5a6; }}
        .subtest {{ margin-left: 20px; padding: 5px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🏁 AgentMem Test Report</h1>
        <p>Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
    </div>
    
    <div class="summary">
        <h2>📊 Summary</h2>
        <p>Total Tests: {summary['total_tests']}</p>
        <p>✅ Passed: {summary['passed_tests']}</p>
        <p>❌ Failed: {summary['failed_tests']}</p>
        <p>⏭️ Skipped: {summary['skipped_tests']}</p>
        <p>📈 Success Rate: {(summary['passed_tests']/summary['total_tests']*100):.1f}%</p>
        <p>⏱️ Duration: {summary['total_duration']:.2f}s</p>
    </div>
    
    <h2>Test Details</h2>
"""
        
        for test in self.tests:
            status_class = test.status.value
            status_icon = "✅" if test.status == TestStatus.PASSED else "❌" if test.status == TestStatus.FAILED else "⏭️"
            
            html_content += f"""
    <div class="test {status_class}">
        <h3>{status_icon} {test.name}</h3>
        <p>{test.description}</p>
        <p>Duration: {test.duration:.2f}s</p>
"""
            
            if test.subtests:
                html_content += "<div class='subtests'>"
                for subtest in test.subtests:
                    subtest_icon = "✅" if subtest.status == TestStatus.PASSED else "❌" if subtest.status == TestStatus.FAILED else "⏭️"
                    html_content += f"<div class='subtest'>{subtest_icon} {subtest.name}</div>"
                html_content += "</div>"
            
            html_content += "</div>"
        
        html_content += """
</body>
</html>
"""
        
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(html_content)
        
        print(f"\n📄 HTML report saved to: {filename}")


# 示例使用
if __name__ == "__main__":
    # 创建测试跟踪器
    tracker = TestTracker()
    
    # 测试 1: Memory Engine
    tracker.start_test("Memory Engine Tests", "测试记忆引擎核心功能")
    
    idx = tracker.start_subtest("测试记忆存储")
    try:
        # 模拟测试
        time.sleep(0.1)
        tracker.pass_subtest(idx, "成功存储 100 条记忆")
    except Exception as e:
        tracker.fail_subtest(e, idx)
    
    idx = tracker.start_subtest("测试记忆检索")
    try:
        time.sleep(0.1)
        tracker.pass_subtest(idx, "成功检索所有记忆")
    except Exception as e:
        tracker.fail_subtest(e, idx)
    
    tracker.pass_test("所有测试通过")
    
    # 测试 2: Search Methods
    tracker.start_test("Search Methods Tests", "测试不同搜索方法")
    
    for method in ["BM25", "Embedding", "StringMatch"]:
        idx = tracker.start_subtest(f"测试 {method} 搜索")
        try:
            time.sleep(0.05)
            tracker.pass_subtest(idx, f"{method} 搜索正常")
        except Exception as e:
            tracker.fail_subtest(e, idx)
    
    tracker.pass_test("所有搜索方法测试通过")
    
    # 打印摘要
    tracker.print_summary()
    
    # 保存报告
    tracker.save_report("test_report.json")
    tracker.save_html_report("test_report.html")

