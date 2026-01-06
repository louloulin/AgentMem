#!/usr/bin/env python3
"""
Mem0 简化性能基准测试
真实运行 Mem0 的性能测试
"""

import os
import time
import sys

# 配置代理（如果需要）
if "https_proxy" in os.environ:
    print(f"使用代理: {os.environ['https_proxy']}")

from mem0 import Memory

def benchmark_mem0_simple(num_items=50, infer=False):
    """测试 Mem0 的简单添加性能"""
    print(f"\n{'='*60}")
    print(f"Mem0 性能测试 (infer={infer})")
    print(f"{'='*60}")
    print(f"测试项数: {num_items}")
    
    # 初始化 Memory（默认优先本地 fastembed，无需 API Key）
    print("\n初始化 Memory...")
    try:
        if not os.getenv("OPENAI_API_KEY"):
            print("⚠️  未设置 OPENAI_API_KEY，使用本地 fastembed 配置（禁用 LLM）...")
            mem = Memory.from_config({
                "vector_store": {
                    "provider": "qdrant",
                    "config": {
                        "collection_name": "mem0",
                        "path": "./qdrant_db"
                    }
                },
                "embedder": {
                    "provider": "fastembed",
                    "config": {
                        "model_name": "BAAI/bge-small-en-v1.5"
                    }
                }
            })
        else:
            mem = Memory()
        print("✅ Memory 初始化成功")
    except Exception as e:
        print(f"❌ Memory 初始化失败: {e}")
        print("\n提示: 请确保已安装 mem0 及 fastembed:")
        print("  pip install mem0ai fastembed")
        return 0, 0
    
    # 准备测试数据
    test_contents = [
        f"Test memory item {i}: This is a test memory for performance benchmarking."
        for i in range(num_items)
    ]
    
    # 测试添加性能
    print(f"\n开始添加 {num_items} 项...")
    start_time = time.time()
    results = []
    
    for i, content in enumerate(test_contents):
        try:
            # Mem0 的 add 方法接受消息列表
            result = mem.add(
                [{"role": "user", "content": content}],
                user_id="test-user",
                agent_id="test-agent",
                infer=infer
            )
            results.append(result)
            if (i + 1) % 10 == 0:
                print(f"  已添加: {i + 1}/{num_items}")
        except Exception as e:
            print(f"  添加失败 (item {i}): {e}")
            # 继续执行，不中断测试
    
    elapsed_time = time.time() - start_time
    success_count = len(results)
    ops_per_sec = success_count / elapsed_time if elapsed_time > 0 else 0
    
    print(f"\n{'='*60}")
    print("测试结果")
    print(f"{'='*60}")
    print(f"成功: {success_count}/{num_items}")
    print(f"耗时: {elapsed_time:.3f} 秒")
    print(f"吞吐量: {ops_per_sec:.2f} ops/s")
    if success_count > 0:
        print(f"平均延迟: {elapsed_time / success_count * 1000:.2f} ms")
    
    return ops_per_sec, elapsed_time

def main():
    """主函数"""
    print("=" * 60)
    print("Mem0 性能基准测试")
    print("=" * 60)
    print("\n注意: 此测试需要安装 mem0")
    print("安装命令: pip install mem0")
    print()
    
    # 检查是否安装了 mem0
    try:
        import mem0
        print("✅ mem0 已安装")
    except ImportError:
        print("❌ mem0 未安装")
        print("\n请先安装 mem0:")
        print("  pip install mem0")
        sys.exit(1)
    
    # 测试 1: 单个添加 (infer=False)
    print("\n" + "=" * 60)
    print("测试 1: 单个添加 (infer=False)")
    print("=" * 60)
    ops1, time1 = benchmark_mem0_simple(num_items=50, infer=False)
    
    # 测试 2: 批量添加 (infer=False)
    print("\n" + "=" * 60)
    print("测试 2: 批量添加 (infer=False)")
    print("=" * 60)
    ops2, time2 = benchmark_mem0_simple(num_items=100, infer=False)
    
    # 总结
    print("\n" + "=" * 60)
    print("性能总结")
    print("=" * 60)
    print(f"单个添加 (50项): {ops1:.2f} ops/s")
    print(f"批量添加 (100项): {ops2:.2f} ops/s")
    print(f"\n注意: Mem0 目标性能为 10,000 ops/s (infer=False)")
    print(f"当前测试结果可能受环境因素影响")

if __name__ == "__main__":
    main()
