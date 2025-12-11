#!/usr/bin/env python3
"""
Mem0 性能基准测试
对比 Mem0 和 AgentMem 的性能
"""

import time
import asyncio
from mem0 import Memory
import os

# 设置环境变量（如果需要）
# os.environ["OPENAI_API_KEY"] = "your-key-here"

def benchmark_mem0_add(num_items=100, infer=False):
    """测试 Mem0 的添加性能"""
    print(f"\n=== Mem0 性能测试 (infer={infer}) ===")
    print(f"测试项数: {num_items}")
    
    # 初始化 Memory：默认使用本地 fastembed + 本地 Qdrant，LLM 使用 dummy key（infer=False 时不会调用）
    try:
        os.environ.setdefault("OPENAI_API_KEY", "sk-DUMMY-NO-CALL")

        mem = Memory.from_config({
            "embedder": {
                "provider": "fastembed",
                "config": {
                    "model": "BAAI/bge-small-en-v1.5"
                }
            },
            "vector_store": {
                "provider": "qdrant",
                "config": {
                    "collection_name": "mem0_benchmark_fe",
                    "path": "/tmp/mem0_benchmark_fe",
                    "embedding_model_dims": 384
                }
            },
            "llm": {
                "provider": "openai",
                "config": {
                    "api_key": "sk-DUMMY-NO-CALL"
                }
            }
        })
        print("✅ Memory 初始化成功")
    except Exception as e:
        print(f"❌ Memory 初始化失败: {e}")
        return 0, 0
    
    # 准备测试数据
    test_contents = [f"Test memory item {i}: This is a test memory for performance benchmarking." for i in range(num_items)]
    
    # 测试添加性能
    start_time = time.time()
    results = []
    
    for i, content in enumerate(test_contents):
        try:
            # Mem0 的 add 方法接受消息列表或字符串
            if isinstance(content, str):
                # 如果是字符串，转换为消息格式
                result = mem.add([{"role": "user", "content": content}], user_id="test-user", agent_id="test-agent", infer=infer)
            else:
                result = mem.add(content, user_id="test-user", agent_id="test-agent", infer=infer)
            results.append(result)
            if (i + 1) % 10 == 0:
                print(f"  已添加: {i + 1}/{num_items}")
        except Exception as e:
            print(f"  添加失败 (item {i}): {e}")
            import traceback
            traceback.print_exc()
    
    elapsed_time = time.time() - start_time
    ops_per_sec = num_items / elapsed_time if elapsed_time > 0 else 0
    
    print(f"\n结果:")
    print(f"  成功: {len(results)}/{num_items}")
    print(f"  耗时: {elapsed_time:.3f} 秒")
    print(f"  吞吐量: {ops_per_sec:.2f} ops/s")
    print(f"  平均延迟: {elapsed_time / num_items * 1000:.2f} ms")
    
    return ops_per_sec, elapsed_time

def benchmark_mem0_batch_add(num_items=100, infer=False):
    """测试 Mem0 的批量添加性能"""
    print(f"\n=== Mem0 批量添加性能测试 (infer={infer}) ===")
    print(f"测试项数: {num_items}")
    
    # 初始化 Memory：默认使用本地 fastembed + 本地 Qdrant，LLM 使用 dummy key（infer=False 时不会调用）
    try:
        os.environ.setdefault("OPENAI_API_KEY", "sk-DUMMY-NO-CALL")

        mem = Memory.from_config({
            "embedder": {
                "provider": "fastembed",
                "config": {
                    "model": "BAAI/bge-small-en-v1.5"
                }
            },
            "vector_store": {
                "provider": "qdrant",
                "config": {
                    "collection_name": "mem0_benchmark_batch_fe",
                    "path": "/tmp/mem0_benchmark_batch_fe",
                    "embedding_model_dims": 384
                }
            },
            "llm": {
                "provider": "openai",
                "config": {
                    "api_key": "sk-DUMMY-NO-CALL"
                }
            }
        })
        print("✅ Memory 初始化成功")
    except Exception as e:
        print(f"❌ Memory 初始化失败: {e}")
        return 0, 0
    
    # 准备测试数据
    test_contents = [f"Test memory item {i}: This is a test memory for performance benchmarking." for i in range(num_items)]
    
    # 测试批量添加性能
    start_time = time.time()
    
    try:
        # Mem0 可能没有直接的批量 API，所以循环调用
        results = []
        for i, content in enumerate(test_contents):
            try:
                # Mem0 的 add 方法接受消息列表或字符串
                if isinstance(content, str):
                    # 如果是字符串，转换为消息格式
                    result = mem.add([{"role": "user", "content": content}], user_id="test-user", agent_id="test-agent", infer=infer)
                else:
                    result = mem.add(content, user_id="test-user", agent_id="test-agent", infer=infer)
                results.append(result)
            except Exception as e:
                print(f"  添加失败 (item {i}): {e}")
                import traceback
                traceback.print_exc()
        
        elapsed_time = time.time() - start_time
        ops_per_sec = num_items / elapsed_time if elapsed_time > 0 else 0
        
        print(f"\n结果:")
        print(f"  成功: {len(results)}/{num_items}")
        print(f"  耗时: {elapsed_time:.3f} 秒")
        print(f"  吞吐量: {ops_per_sec:.2f} ops/s")
        print(f"  平均延迟: {elapsed_time / num_items * 1000:.2f} ms")
        
        return ops_per_sec, elapsed_time
    except Exception as e:
        print(f"批量添加失败: {e}")
        return 0, 0

def main():
    """主函数"""
    print("=" * 60)
    print("Mem0 性能基准测试")
    print("=" * 60)
    
    # 测试 1: 单个添加 (infer=False)
    print("\n" + "=" * 60)
    print("测试 1: 单个添加 (infer=False)")
    print("=" * 60)
    ops1, time1 = benchmark_mem0_add(num_items=50, infer=False)
    
    # 测试 2: 批量添加 (infer=False)
    print("\n" + "=" * 60)
    print("测试 2: 批量添加 (infer=False)")
    print("=" * 60)
    ops2, time2 = benchmark_mem0_batch_add(num_items=100, infer=False)
    
    # 总结
    print("\n" + "=" * 60)
    print("性能总结")
    print("=" * 60)
    print(f"单个添加 (50项): {ops1:.2f} ops/s")
    print(f"批量添加 (100项): {ops2:.2f} ops/s")
    print(f"\n注意: Mem0 目标性能为 10,000 ops/s (infer=False)")

if __name__ == "__main__":
    main()
