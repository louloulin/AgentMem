"""
AgentMem Python 基础示例

演示最基本的使用方法
"""

import asyncio
from agentmem import Memory


async def main():
    print("=== AgentMem Python 基础示例 ===\n")

    # 1. 创建实例
    print("1. 创建 Memory 实例...")
    memory = await Memory.new()
    print("   ✓ 初始化成功\n")

    # 2. 添加记忆
    print("2. 添加记忆...")
    result1 = await memory.add("用户喜欢咖啡")
    print(f"   ✓ 添加: {result1.id}")

    result2 = await memory.add("用户住在上海")
    print(f"   ✓ 添加: {result2.id}")

    result3 = await memory.add("用户是软件工程师")
    print(f"   ✓ 添加: {result3.id}\n")

    # 3. 搜索记忆
    print("3. 搜索记忆...")
    query = "用户的爱好和职业"
    results = await memory.search(query)

    print(f"   查询: \"{query}\"")
    print(f"   找到 {len(results)} 条结果:\n")

    for i, result in enumerate(results):
        print(f"   {i+1}. {result.content} (相似度: {result.score:.2f})")

    # 4. 获取所有记忆
    print("\n4. 获取所有记忆...")
    all_memories = await memory.get_all()

    print(f"   总共 {len(all_memories)} 条记忆:\n")
    for i, mem in enumerate(all_memories):
        print(f"   {i+1}. {mem.content}")

    # 5. 清理
    print("\n5. 清理...")
    await memory.delete_all()
    print("   ✓ 已清理所有记忆")

    print("\n=== 示例完成 ===")


if __name__ == "__main__":
    asyncio.run(main())
