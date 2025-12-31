"""
AgentMem Python 搜索示例

演示各种搜索功能
"""

import asyncio
from agentmem import Memory, SearchOptions


async def main():
    print("=== AgentMem Python 搜索示例 ===\n")

    memory = await Memory.new()

    # 添加示例数据
    print("1. 添加示例数据...")
    data = [
        "用户喜欢科幻电影，尤其是《星际穿越》和《银翼杀手》",
        "用户住在上海，喜欢在周末去咖啡厅",
        "用户是软件工程师，主要使用 Rust 和 Python",
        "用户最近开始学习机器学习",
        "用户喜欢吃辣，最喜欢的菜是麻婆豆腐",
    ]

    for item in data:
        await memory.add(item)
        print(f"   ✓ {item[:30]}...")

    print("\n2. 简单搜索...")
    results = await memory.search("用户的兴趣爱好")
    print(f"   查询: \"用户的兴趣爱好\"")
    print(f"   结果: {len(results)} 条\n")

    for i, result in enumerate(results[:3]):
        print(f"   {i+1}. {result.content} ({result.score:.2f})")

    print("\n3. 带阈值搜索...")
    results = await memory.searchWithOptions(
        SearchOptions(
            query="编程",
            threshold=0.7
        )
    )
    print(f"   阈值 > 0.7: {len(results)} 条结果")

    print("\n4. 限制结果数量...")
    results = await memory.searchWithOptions(
        SearchOptions(
            query="用户",
            limit=2
        )
    )
    print(f"   最多 2 条: {len(results)} 条结果")

    print("\n=== 示例完成 ===")


if __name__ == "__main__":
    asyncio.run(main())
