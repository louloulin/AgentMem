#!/usr/bin/env python3
"""
AgentMem Python SDK 基础演示

展示Python绑定的核心功能：
1. 创建Memory实例
2. 添加记忆
3. 搜索记忆
4. 获取所有记忆
5. 删除记忆
"""

import asyncio
from typing import List, Dict


async def main():
    """主函数"""
    print("🐍 AgentMem Python SDK 基础演示\n")
    
    # 导入AgentMem（假设已通过maturin构建）
    try:
        import agentmem_native
        print("✅ 成功导入 agentmem_native\n")
    except ImportError as e:
        print("❌ 导入失败，请先构建Python绑定：")
        print("   cd crates/agent-mem-python")
        print("   maturin develop\n")
        return
    
    # 1. 创建Memory实例
    print("1️⃣ 创建Memory实例...")
    memory = agentmem_native.Memory()
    print("✅ Memory实例创建成功\n")
    
    # 2. 添加记忆
    print("2️⃣ 添加记忆...")
    memories = [
        "Python是一门简单易学的编程语言",
        "Rust提供了出色的性能和内存安全",
        "AgentMem是高性能的AI记忆管理平台",
        "机器学习在各个领域都有广泛应用",
        "向量数据库可以实现语义相似度搜索",
    ]
    
    for content in memories:
        memory_id = await memory.add(content)
        print(f"  ✅ 添加成功: {content[:30]}... (ID: {memory_id[:8]})")
    print()
    
    # 3. 搜索记忆
    print("3️⃣ 搜索记忆...")
    queries = [
        ("编程", "搜索关于编程的记忆"),
        ("性能", "搜索关于性能的记忆"),
        ("AI", "搜索关于AI的记忆"),
    ]
    
    for query, description in queries:
        print(f"\n  🔍 {description}: \"{query}\"")
        results = await memory.search(query)
        
        if not results:
            print("    ℹ️  未找到匹配的记忆")
        else:
            print(f"    ✅ 找到 {len(results)} 条相关记忆:")
            for i, result in enumerate(results[:3], 1):
                content = result.get('content', 'N/A')
                print(f"       {i}. {content[:50]}...")
    print()
    
    # 4. 获取所有记忆
    print("4️⃣ 获取所有记忆...")
    all_memories = await memory.get_all()
    print(f"  ✅ 共有 {len(all_memories)} 条记忆:")
    for i, mem in enumerate(all_memories, 1):
        content = mem.get('content', 'N/A')
        print(f"     {i}. {content}")
    print()
    
    # 5. 删除记忆
    print("5️⃣ 删除记忆...")
    if all_memories:
        first_id = all_memories[0].get('id')
        first_content = all_memories[0].get('content', 'N/A')
        
        result = await memory.delete(first_id)
        if result:
            print(f"  ✅ 成功删除记忆: {first_content}")
    print()
    
    # 6. 验证删除
    print("6️⃣ 验证删除后的记忆数量...")
    remaining = await memory.get_all()
    print(f"  ✅ 现在有 {len(remaining)} 条记忆（已删除1条）\n")
    
    # 7. 清空所有记忆
    print("7️⃣ 清空所有记忆...")
    count = await memory.clear()
    print(f"  ✅ 成功清空 {count} 条记忆\n")
    
    print("🎉 演示完成！\n")
    print("📊 AgentMem Python SDK特性：")
    print("  ✅ 简单易用的API")
    print("  ✅ 异步支持（async/await）")
    print("  ✅ 高性能Rust后端")
    print("  ✅ 类型安全")
    print("  ✅ 零配置启动")


if __name__ == "__main__":
    # 运行异步主函数
    asyncio.run(main())

