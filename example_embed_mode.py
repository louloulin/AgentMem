#!/usr/bin/env python3
"""
AgentMem Embed 模式示例

演示如何将 AgentMem 作为库直接嵌入到 Python 应用中使用
（无需独立服务器）

依赖: pip install agentmem-native
或从源码: cd crates/agent-mem-python && maturin develop
"""

import asyncio
from typing import List, Dict, Optional

try:
    from agentmem_native import Memory
except ImportError:
    print("❌ agentmem_native 未安装")
    print("\n安装方式:")
    print("  方法1 (推荐用户): pip install agentmem-native")
    print("  方法2 (开发者): cd crates/agent-mem-python && maturin develop")
    exit(1)


class EmbedChatBot:
    """
    嵌入式聊天机器人示例

    使用 AgentMem Embed 模式，无需独立服务器
    """

    def __init__(self, bot_name: str = "AI Assistant"):
        self.bot_name = bot_name
        # 直接创建 Memory 实例（无需服务器连接）
        self.memory = Memory()
        print(f"✅ {self.bot_name} 已启动 (Embed 模式)")

    async def remember(self, content: str) -> str:
        """记住用户说的话"""
        try:
            memory_id = await self.memory.add(content)
            print(f"   💾 已记住: {content[:50]}...")
            return memory_id
        except Exception as e:
            print(f"   ❌ 记忆失败: {e}")
            return ""

    async def recall(self, query: str, limit: int = 3) -> List[str]:
        """回忆相关内容"""
        try:
            results = await self.memory.search(query)
            return [r['content'] for r in results[:limit]]
        except Exception as e:
            print(f"   ❌ 回忆失败: {e}")
            return []

    async def chat(self, user_input: str) -> str:
        """聊天"""
        print(f"\n👤 用户: {user_input}")

        # 1. 搜索相关记忆
        context = await self.recall(user_input)

        # 2. 记住这次对话
        await self.remember(f"User said: {user_input}")

        # 3. 生成响应（简化版）
        if context:
            response = f"我记得你说过: {context[0]}"
        else:
            response = "我记住了！请继续告诉我更多。"

        print(f"🤖 {self.bot_name}: {response}")
        return response


class EmbedKnowledgeBase:
    """
    嵌入式知识库示例

    适合单机应用的知识管理
    """

    def __init__(self, name: str = "知识库"):
        self.name = name
        self.memory = Memory()
        print(f"✅ {name} 已创建 (Embed 模式)")

    async def add_knowledge(self, fact: str) -> str:
        """添加知识"""
        memory_id = await self.memory.add(fact)
        print(f"   ✅ 添加知识: {fact[:50]}...")
        return memory_id

    async def search_knowledge(self, query: str, limit: int = 5) -> List[Dict]:
        """搜索知识"""
        results = await self.memory.search(query)
        print(f"   🔍 搜索 '{query}' 找到 {len(results)} 条结果:")
        for i, result in enumerate(results[:limit], 1):
            content = result.get('content', '')
            print(f"      {i}. {content[:70]}...")
        return results

    async def get_all_knowledge(self) -> List[Dict]:
        """获取所有知识"""
        all_knowledge = await self.memory.get_all()
        print(f"   📚 {self.name} 共有 {len(all_knowledge)} 条知识")
        return all_knowledge


class EmbedUserPreferences:
    """
    嵌入式用户偏好管理示例

    在本地管理用户偏好，无需服务器
    """

    def __init__(self, user_id: str):
        self.user_id = user_id
        self.memory = Memory()
        print(f"✅ 用户 {user_id} 的偏好管理器已创建 (Embed 模式)")

    async def save_preference(self, category: str, value: str) -> str:
        """保存偏好"""
        content = f"[{category}] {value}"
        memory_id = await self.memory.add(content)
        print(f"   ✅ 保存偏好: {category} = {value}")
        return memory_id

    async def get_preferences(self, category: Optional[str] = None) -> List[str]:
        """获取偏好"""
        if category:
            query = f"[{category}]"
            results = await self.memory.search(query)
            prefs = [r['content'] for r in results]
            print(f"   📋 类别 '{category}' 的偏好: {len(prefs)} 条")
        else:
            results = await self.memory.get_all()
            prefs = [r['content'] for r in results]
            print(f"   📋 所有偏好: {len(prefs)} 条")

        for pref in prefs[:5]:
            print(f"      - {pref}")
        return prefs


async def demo_chatbot():
    """演示 1: 聊天机器人"""
    print("\n" + "=" * 60)
    print("🤖 演示 1: 嵌入式聊天机器人")
    print("=" * 60)

    bot = EmbedChatBot("小爱")

    # 对话 1
    await bot.chat("我喜欢喝咖啡")

    # 对话 2
    await bot.chat("我最喜欢的编程语言是 Python")

    # 对话 3（应该能回忆起之前的对话）
    await bot.chat("我喝什么？")

    # 对话 4（应该能回忆起编程语言）
    await bot.chat("我喜欢什么编程语言？")


async def demo_knowledge_base():
    """演示 2: 知识库"""
    print("\n" + "=" * 60)
    print("📚 演示 2: 嵌入式知识库")
    print("=" * 60)

    kb = EmbedKnowledgeBase("技术知识库")

    # 添加知识
    print("\n📝 添加知识:")
    await kb.add_knowledge("Rust 是一门系统编程语言，注重性能和安全性")
    await kb.add_knowledge("Python 适合快速开发和数据科学")
    await kb.add_knowledge("Go 语言擅长并发编程和云原生开发")
    await kb.add_knowledge("JavaScript 是 Web 开发的核心语言")
    await kb.add_knowledge("TypeScript 是 JavaScript 的超集，添加了类型系统")

    # 搜索知识
    print("\n🔍 搜索知识:")
    await kb.search_knowledge("编程语言", limit=3)
    print()
    await kb.search_knowledge("性能", limit=3)
    print()
    await kb.search_knowledge("Web 开发", limit=3)

    # 获取所有知识
    print("\n📚 知识库统计:")
    await kb.get_all_knowledge()


async def demo_user_preferences():
    """演示 3: 用户偏好管理"""
    print("\n" + "=" * 60)
    print("👤 演示 3: 嵌入式用户偏好管理")
    print("=" * 60)

    prefs = EmbedUserPreferences("user_001")

    # 保存偏好
    print("\n💾 保存用户偏好:")
    await prefs.save_preference("food", "喜欢喝咖啡")
    await prefs.save_preference("food", "爱吃意大利菜")
    await prefs.save_preference("hobby", "喜欢编程")
    await prefs.save_preference("hobby", "热爱徒步旅行")
    await prefs.save_preference("music", "喜欢爵士乐")

    # 获取特定类别偏好
    print("\n🔍 查询食物偏好:")
    await prefs.get_preferences("food")

    print("\n🔍 查询爱好偏好:")
    await prefs.get_preferences("hobby")

    # 获取所有偏好
    print("\n📋 查询所有偏好:")
    await prefs.get_preferences()


async def demo_performance():
    """演示 4: 性能测试"""
    print("\n" + "=" * 60)
    print("⚡ 演示 4: Embed 模式性能测试")
    print("=" * 60)

    import time

    memory = Memory()

    # 测试添加性能
    print("\n📝 测试: 批量添加 100 条记忆")
    start = time.time()

    for i in range(100):
        await memory.add(f"测试记忆 {i}: 这是一条测试数据")

    elapsed = time.time() - start
    print(f"   ✅ 完成! 耗时: {elapsed:.3f} 秒")
    print(f"   📊 平均: {elapsed/100*1000:.2f} ms/条")

    # 测试搜索性能
    print("\n🔍 测试: 搜索性能")
    start = time.time()

    results = await memory.search("测试记忆")

    elapsed = time.time() - start
    print(f"   ✅ 找到 {len(results)} 条结果")
    print(f"   ⏱️  耗时: {elapsed*1000:.2f} ms")

    # 测试获取所有性能
    print("\n📚 测试: 获取所有记忆")
    start = time.time()

    all_memories = await memory.get_all()

    elapsed = time.time() - start
    print(f"   ✅ 共有 {len(all_memories)} 条记忆")
    print(f"   ⏱️  耗时: {elapsed*1000:.2f} ms")


async def main():
    """主演示函数"""
    print("=" * 70)
    print("🚀 AgentMem Embed 模式示例")
    print("=" * 70)
    print("\n💡 Embed 模式特点:")
    print("   ✅ 无需独立服务器")
    print("   ✅ 直接导入使用")
    print("   ✅ 性能极致优化")
    print("   ✅ 部署极其简单")
    print()

    # 运行所有演示
    await demo_chatbot()
    await demo_knowledge_base()
    await demo_user_preferences()
    await demo_performance()

    print("\n" + "=" * 70)
    print("🎉 所有演示完成！")
    print("=" * 70)
    print("\n💡 你已经看到了 AgentMem Embed 模式的所有核心功能！")
    print("\n🚀 开始使用:")
    print("   1. 安装: pip install agentmem-native")
    print("   2. 导入: from agentmem_native import Memory")
    print("   3. 使用: memory = Memory()")
    print()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n👋 演示被用户中断")
    except Exception as e:
        print(f"\n\n❌ 错误: {e}")
        import traceback
        traceback.print_exc()
