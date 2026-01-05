#!/usr/bin/env python3
"""
AgentMem Python SDK - 5分钟快速开始示例

这个示例演示了 AgentMem Python SDK 的核心功能：
- 初始化客户端
- 添加记忆
- 语义搜索
- 显示结果

运行方式:
```bash
# 设置环境变量
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key

# 运行示例
python quick_start.py
```

预期输出:
```
🚀 AgentMem Python SDK - 5分钟快速开始

✅ 步骤 1: 初始化客户端
   API Base URL: http://localhost:8080

📝 步骤 2: 添加记忆
   ✅ 添加: "我喜欢编程" -> mem_001
   ✅ 添加: "我住在上海" -> mem_002
   ✅ 添加: "我的编程语言是 Python" -> mem_003

🔍 步骤 3: 搜索记忆
   搜索: "编程"
   ✅ 找到 2 条记忆:
      1. 我喜欢编程 (相似度: 0.95)
      2. 我的编程语言是 Python (相似度: 0.92)

📚 步骤 4: 获取所有记忆
   ✅ 共有 3 条记忆:
      1. 我喜欢编程
      2. 我住在上海
      3. 我的编程语言是 Python

🎉 完成！
```
"""

import asyncio
import os
from typing import Optional

# 假设的 AgentMem Python SDK 导入
# 实际使用时安装: pip install agentmem
try:
    from agentmem import AgentMemClient, Config, SearchQuery, MemoryType
except ImportError:
    print("⚠️  AgentMem SDK 未安装")
    print("   安装方式: pip install agentmem")
    print("   或使用本地版本:")
    print("   cd sdks/python && pip install -e .")
    exit(1)


async def main():
    """主函数"""
    print("🚀 AgentMem Python SDK - 5分钟快速开始\n")

    # ============================================
    # 步骤 1: 初始化客户端
    # ============================================
    print("✅ 步骤 1: 初始化客户端")

    # 从环境变量获取配置
    api_base_url = os.getenv("AGENTMEM_API_BASE_URL", "http://localhost:8080")
    api_key = os.getenv("AGENTMEM_API_KEY", "demo_key")

    print(f"   API Base URL: {api_base_url}")
    print(f"   API Key: {api_key[:8]}...")

    # 创建配置
    config = Config(
        api_base_url=api_base_url,
        api_key=api_key,
        timeout=30.0,
    )

    # 创建客户端
    async with AgentMemClient(config) as client:
        print("   ✅ 客户端已连接\n")

        # ============================================
        # 步骤 2: 添加记忆
        # ============================================
        print("📝 步骤 2: 添加记忆")

        # 添加第一条记忆
        result1 = await client.add_memory(
            content="我喜欢编程",
            agent_id="agent_1",
            user_id="user_1",
            memory_type=MemoryType.EPISODIC,
        )
        print(f"   ✅ 添加: \"我喜欢编程\" -> {result1}")

        # 添加第二条记忆
        result2 = await client.add_memory(
            content="我住在上海",
            agent_id="agent_1",
            user_id="user_1",
            memory_type=MemoryType.EPISODIC,
        )
        print(f"   ✅ 添加: \"我住在上海\" -> {result2}")

        # 添加第三条记忆（与第一条相关）
        result3 = await client.add_memory(
            content="我的编程语言是 Python",
            agent_id="agent_1",
            user_id="user_1",
            memory_type=MemoryType.EPISODIC,
        )
        print(f"   ✅ 添加: \"我的编程语言是 Python\" -> {result3}\n")

        # ============================================
        # 步骤 3: 搜索记忆
        # ============================================
        print("🔍 步骤 3: 搜索记忆")

        # 创建搜索查询
        query = SearchQuery(
            query="编程",
            user_id="user_1",
            limit=5,
            threshold=0.7,
        )

        # 执行搜索
        results = await client.search_memories(query)
        print(f"   搜索: \"编程\"")
        print(f"   ✅ 找到 {len(results)} 条记忆:")

        # 显示搜索结果
        for i, memory in enumerate(results, 1):
            score = memory.get("score", 0.0)
            content = memory.get("content", "")
            print(f"      {i}. {content} (相似度: {score:.2f})")
        print()

        # ============================================
        # 步骤 4: 获取所有记忆
        # ============================================
        print("📚 步骤 4: 获取所有记忆")

        all_memories = await client.get_all_memories(
            user_id="user_1",
            limit=10,
        )
        print(f"   ✅ 共有 {len(all_memories)} 条记忆:")

        for i, memory in enumerate(all_memories, 1):
            content = memory.get("content", "")
            print(f"      {i}. {content}")
        print()

        # ============================================
        # 完成
        # ============================================
        print("🎉 完成！")
        print("\n💡 下一步:")
        print("   - 查看 chatbot.py 了解如何构建聊天机器人")
        print("   - 查看 rag_qa.py 了解如何构建 RAG 系统")
        print("   - 查看 personal_assistant.py 了解个人助理")


if __name__ == "__main__":
    """入口点"""
    try:
        # 运行异步主函数
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n👋 用户中断")
    except Exception as e:
        print(f"\n\n❌ 错误: {e}")
        print("\n💡 故障排除:")
        print("   1. 确保 AgentMem 服务器正在运行")
        print("   2. 检查 API Base URL 和 API Key")
        print("   3. 查看日志了解详细错误信息")
        exit(1)


# ============================================
# 错误处理示例
# ============================================
#
# Python SDK 提供了完整的异常处理:
#
# ```python
# from agentmem import AgentMemError, ConnectionError, ValidationError
#
# try:
#     result = await client.add_memory(...)
# except ConnectionError as e:
#     print(f"连接失败: {e}")
# except ValidationError as e:
#     print(f"验证失败: {e}")
# except AgentMemError as e:
#     print(f"通用错误: {e}")
# ```
#
# ============================================
# 高级配置
# ============================================
#
# 你可以自定义客户端行为:
#
# ```python
# config = Config(
#     api_base_url="http://localhost:8080",
#     api_key="your_key",
#     timeout=30.0,              # 请求超时
#     max_retries=3,             # 最大重试次数
#     retry_delay=1.0,           # 重试延迟
#     enable_cache=True,         # 启用缓存
#     cache_ttl=300,            # 缓存过期时间（秒）
# )
#
# client = AgentMemClient(config)
# ```
