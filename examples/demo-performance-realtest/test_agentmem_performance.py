#!/usr/bin/env python3
"""
AgentMem 真实性能测试

对标目标: 验证 AgentMem vs Mem0 的实际性能差异
测试项目:
- 添加记忆延迟
- 搜索记忆延迟
- 批量操作性能
- 并发吞吐量
- 内存占用
- 启动时间

运行方式:
  python3 test_agentmem_performance.py
"""

import os
import sys
import time
import psutil
import asyncio
from pathlib import Path
from typing import List, Dict
from datetime import datetime

# Add project root to path
project_root = Path(__file__).parent.parent.parent.absolute()
sys.path.insert(0, str(project_root))

try:
    from agent_mem_python import AgentMem
except ImportError:
    print("❌ Error: agent_mem_python not found")
    print("Please build the Python bindings first:")
    print("  cd crates/agent-mem-python && maturin develop --release")
    sys.exit(1)


class PerformanceTest:
    """性能测试类"""
    
    def __init__(self):
        self.results = {}
        self.process = psutil.Process()
    
    def test_startup_time(self) -> float:
        """测试启动时间"""
        print("\n" + "="*70)
        print("📊 测试 1: 启动时间")
        print("="*70)
        
        start_time = time.time()
        
        memory = AgentMem(
            embedder_provider="fastembed",
            embedder_model="bge-small-en-v1.5",
            disable_intelligent_features=True
        )
        
        startup_time = (time.time() - start_time) * 1000  # ms
        
        print(f"✅ AgentMem 启动时间: {startup_time:.2f}ms")
        
        self.results['startup_time_ms'] = startup_time
        return startup_time
    
    def test_add_memory_latency(self, memory: AgentMem, iterations: int = 100) -> Dict:
        """测试添加记忆延迟"""
        print("\n" + "="*70)
        print(f"📊 测试 2: 添加记忆延迟 ({iterations}次)")
        print("="*70)
        
        latencies = []
        
        for i in range(iterations):
            content = f"Test memory item {i}: This is a sample memory for performance testing."
            
            start_time = time.time()
            memory.add(content, user_id="perf_test")
            latency = (time.time() - start_time) * 1000  # ms
            
            latencies.append(latency)
            
            if (i + 1) % 20 == 0:
                avg = sum(latencies) / len(latencies)
                print(f"  进度: {i+1}/{iterations} - 平均延迟: {avg:.2f}ms")
        
        avg_latency = sum(latencies) / len(latencies)
        min_latency = min(latencies)
        max_latency = max(latencies)
        p95_latency = sorted(latencies)[int(len(latencies) * 0.95)]
        
        print(f"\n✅ 添加记忆性能:")
        print(f"  平均延迟: {avg_latency:.2f}ms")
        print(f"  最小延迟: {min_latency:.2f}ms")
        print(f"  最大延迟: {max_latency:.2f}ms")
        print(f"  P95延迟: {p95_latency:.2f}ms")
        
        result = {
            'avg_ms': avg_latency,
            'min_ms': min_latency,
            'max_ms': max_latency,
            'p95_ms': p95_latency,
            'iterations': iterations
        }
        
        self.results['add_memory'] = result
        return result
    
    def test_search_memory_latency(self, memory: AgentMem, iterations: int = 50) -> Dict:
        """测试搜索记忆延迟"""
        print("\n" + "="*70)
        print(f"📊 测试 3: 搜索记忆延迟 ({iterations}次)")
        print("="*70)
        
        # 先添加一些测试数据
        print("  准备测试数据...")
        for i in range(50):
            memory.add(
                f"Test data {i}: Sample content about topic {i % 10}",
                user_id="perf_test"
            )
        
        latencies = []
        queries = [
            "sample content",
            "topic 5",
            "test data",
            "performance",
            "memory item"
        ]
        
        print("  开始搜索测试...")
        for i in range(iterations):
            query = queries[i % len(queries)]
            
            start_time = time.time()
            results = memory.search(query, user_id="perf_test")
            latency = (time.time() - start_time) * 1000  # ms
            
            latencies.append(latency)
            
            if (i + 1) % 10 == 0:
                avg = sum(latencies) / len(latencies)
                print(f"  进度: {i+1}/{iterations} - 平均延迟: {avg:.2f}ms")
        
        avg_latency = sum(latencies) / len(latencies)
        min_latency = min(latencies)
        max_latency = max(latencies)
        p95_latency = sorted(latencies)[int(len(latencies) * 0.95)]
        
        print(f"\n✅ 搜索记忆性能:")
        print(f"  平均延迟: {avg_latency:.2f}ms")
        print(f"  最小延迟: {min_latency:.2f}ms")
        print(f"  最大延迟: {max_latency:.2f}ms")
        print(f"  P95延迟: {p95_latency:.2f}ms")
        
        result = {
            'avg_ms': avg_latency,
            'min_ms': min_latency,
            'max_ms': max_latency,
            'p95_ms': p95_latency,
            'iterations': iterations
        }
        
        self.results['search_memory'] = result
        return result
    
    def test_batch_operations(self, memory: AgentMem, batch_size: int = 50) -> Dict:
        """测试批量操作性能"""
        print("\n" + "="*70)
        print(f"📊 测试 4: 批量操作 ({batch_size}条)")
        print("="*70)
        
        # 批量添加
        print("  测试批量添加...")
        start_time = time.time()
        
        for i in range(batch_size):
            memory.add(
                f"Batch item {i}: Content for batch testing",
                user_id="batch_test"
            )
        
        batch_add_time = (time.time() - start_time) * 1000  # ms
        throughput = batch_size / (batch_add_time / 1000)  # ops/s
        
        print(f"  批量添加: {batch_add_time:.2f}ms ({throughput:.2f} ops/s)")
        
        # 批量搜索
        print("  测试批量搜索...")
        start_time = time.time()
        
        for i in range(10):
            memory.search(f"batch item {i}", user_id="batch_test")
        
        batch_search_time = (time.time() - start_time) * 1000  # ms
        search_throughput = 10 / (batch_search_time / 1000)  # ops/s
        
        print(f"  批量搜索: {batch_search_time:.2f}ms ({search_throughput:.2f} ops/s)")
        
        result = {
            'batch_add_ms': batch_add_time,
            'batch_add_throughput': throughput,
            'batch_search_ms': batch_search_time,
            'batch_search_throughput': search_throughput,
            'batch_size': batch_size
        }
        
        self.results['batch_operations'] = result
        return result
    
    def test_memory_usage(self, memory: AgentMem) -> Dict:
        """测试内存占用"""
        print("\n" + "="*70)
        print("📊 测试 5: 内存占用")
        print("="*70)
        
        # 初始内存
        initial_memory = self.process.memory_info().rss / 1024 / 1024  # MB
        print(f"  初始内存: {initial_memory:.2f}MB")
        
        # 添加100条记忆后的内存
        for i in range(100):
            memory.add(
                f"Memory test {i}: Testing memory usage with content",
                user_id="memory_test"
            )
        
        after_100_memory = self.process.memory_info().rss / 1024 / 1024  # MB
        memory_increase_100 = after_100_memory - initial_memory
        
        print(f"  添加100条后: {after_100_memory:.2f}MB (+{memory_increase_100:.2f}MB)")
        
        # 添加1000条记忆后的内存
        for i in range(900):
            memory.add(
                f"Memory test {100+i}: Testing memory usage with content",
                user_id="memory_test"
            )
        
        after_1000_memory = self.process.memory_info().rss / 1024 / 1024  # MB
        memory_increase_1000 = after_1000_memory - initial_memory
        
        print(f"  添加1000条后: {after_1000_memory:.2f}MB (+{memory_increase_1000:.2f}MB)")
        print(f"  平均每条: {(memory_increase_1000 / 1000):.4f}MB")
        
        result = {
            'initial_mb': initial_memory,
            'after_100_mb': after_100_memory,
            'after_1000_mb': after_1000_memory,
            'increase_per_item_kb': (memory_increase_1000 / 1000) * 1024
        }
        
        self.results['memory_usage'] = result
        return result
    
    def generate_report(self):
        """生成测试报告"""
        print("\n" + "="*70)
        print("📊 AgentMem 性能测试报告")
        print("="*70)
        print(f"测试时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("="*70)
        
        # 1. 启动时间
        if 'startup_time_ms' in self.results:
            print(f"\n1️⃣  启动时间: {self.results['startup_time_ms']:.2f}ms")
        
        # 2. 添加记忆
        if 'add_memory' in self.results:
            add = self.results['add_memory']
            print(f"\n2️⃣  添加记忆 ({add['iterations']}次):")
            print(f"   平均: {add['avg_ms']:.2f}ms")
            print(f"   P95: {add['p95_ms']:.2f}ms")
            print(f"   范围: {add['min_ms']:.2f} - {add['max_ms']:.2f}ms")
        
        # 3. 搜索记忆
        if 'search_memory' in self.results:
            search = self.results['search_memory']
            print(f"\n3️⃣  搜索记忆 ({search['iterations']}次):")
            print(f"   平均: {search['avg_ms']:.2f}ms")
            print(f"   P95: {search['p95_ms']:.2f}ms")
            print(f"   范围: {search['min_ms']:.2f} - {search['max_ms']:.2f}ms")
        
        # 4. 批量操作
        if 'batch_operations' in self.results:
            batch = self.results['batch_operations']
            print(f"\n4️⃣  批量操作 ({batch['batch_size']}条):")
            print(f"   批量添加: {batch['batch_add_throughput']:.2f} ops/s")
            print(f"   批量搜索: {batch['batch_search_throughput']:.2f} ops/s")
        
        # 5. 内存占用
        if 'memory_usage' in self.results:
            mem = self.results['memory_usage']
            print(f"\n5️⃣  内存占用:")
            print(f"   初始: {mem['initial_mb']:.2f}MB")
            print(f"   1000条后: {mem['after_1000_mb']:.2f}MB")
            print(f"   平均每条: {mem['increase_per_item_kb']:.2f}KB")
        
        print("\n" + "="*70)
        print("✅ 测试完成！")
        print("="*70)
        
        # 与理论预测对比
        print("\n📈 与理论预测对比:")
        print("="*70)
        
        if 'add_memory' in self.results:
            theoretical_add = 10  # ms
            actual_add = self.results['add_memory']['avg_ms']
            print(f"添加记忆:")
            print(f"  理论: {theoretical_add}ms")
            print(f"  实际: {actual_add:.2f}ms")
            print(f"  偏差: {abs(actual_add - theoretical_add):.2f}ms")
        
        if 'search_memory' in self.results:
            theoretical_search = 10  # ms
            actual_search = self.results['search_memory']['avg_ms']
            print(f"\n搜索记忆:")
            print(f"  理论: {theoretical_search}ms")
            print(f"  实际: {actual_search:.2f}ms")
            print(f"  偏差: {abs(actual_search - theoretical_search):.2f}ms")
        
        print("\n" + "="*70)


def main():
    """主函数"""
    print("╔════════════════════════════════════════════════════════════════╗")
    print("║                                                                ║")
    print("║         🚀 AgentMem 真实性能测试 🚀                          ║")
    print("║                                                                ║")
    print("║         验证理论分析 vs 实际性能                              ║")
    print("║                                                                ║")
    print("╚════════════════════════════════════════════════════════════════╝")
    
    # 创建测试实例
    tester = PerformanceTest()
    
    try:
        # 测试1: 启动时间
        tester.test_startup_time()
        
        # 创建Memory实例用于后续测试
        print("\n正在初始化 AgentMem...")
        memory = AgentMem(
            embedder_provider="fastembed",
            embedder_model="bge-small-en-v1.5",
            disable_intelligent_features=True
        )
        print("✅ 初始化完成")
        
        # 测试2: 添加记忆延迟
        tester.test_add_memory_latency(memory, iterations=100)
        
        # 测试3: 搜索记忆延迟
        tester.test_search_memory_latency(memory, iterations=50)
        
        # 测试4: 批量操作
        tester.test_batch_operations(memory, batch_size=50)
        
        # 测试5: 内存占用
        tester.test_memory_usage(memory)
        
        # 生成报告
        tester.generate_report()
        
    except Exception as e:
        print(f"\n❌ 测试失败: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

