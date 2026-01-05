#!/usr/bin/env python3
"""
AgentMem Python SDK - 聊天机器人示例

这个示例演示了如何使用 AgentMem 构建智能聊天机器人：
- 对话历史管理
- 上下文检索
- 个性化回复
- 多轮对话

运行方式:
```bash
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key
export OPENAI_API_KEY=sk-...  # 如果使用 LLM 生成回复

python chatbot.py
```

预期输出:
```
🤖 AgentMem 聊天机器人示例

✅ 初始化完成

💬 对话 1:
   用户: 我叫 Alice
   🤖: 很高兴认识你，Alice！
   ✅ 已保存记忆

💬 对话 2:
   用户: 我喜欢编程
   🤖: 编程很棒！
   ✅ 已保存记忆

💬 对话 3:
   用户: 我叫什么名字？
   🤖: 你叫 Alice。
   ✅ 从记忆中检索到: 我叫 Alice

🎉 对话结束！
```
"""

import asyncio
import os
from typing import Optional, List
from datetime import datetime

try:
    from agentmem import AgentMemClient, Config, SearchQuery, MemoryType
except ImportError:
    print("⚠️  AgentMem SDK 未安装")
    print("   安装方式: pip install agentmem")
    exit(1)


class SimpleChatbot:
    """简单聊天机器人"""

    def __init__(self, client: AgentMemClient, user_id: str, agent_id: str):
        """初始化聊天机器人"""
        self.client = client
        self.user_id = user_id
        self.agent_id = agent_id
        self.conversation_history: List[dict] = []

    async def save_message(self, message: str, role: str = "user") -> str:
        """保存消息到记忆"""
        memory_id = await self.client.add_memory(
            content=message,
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=MemoryType.EPISODIC,
            metadata={
                "role": role,
                "timestamp": datetime.now().isoformat(),
            },
        )
        return memory_id

    async def search_context(self, query: str, limit: int = 3) -> List[dict]:
        """搜索相关上下文"""
        search_query = SearchQuery(
            query=query,
            user_id=self.user_id,
            limit=limit,
            threshold=0.7,
        )
        results = await self.client.search_memories(search_query)
        return results

    async def generate_reply(self, user_message: str) -> str:
        """生成回复"""
        # 搜索相关上下文
        context = await self.search_context(user_message)

        # 基于上下文生成回复
        if context:
            top_result = context[0]
            content = top_result.get("content", "")

            if "名字" in user_message or "叫什么" in user_message:
                if "我叫" in content:
                    name = content.replace("我叫", "").strip()
                    return f"你的名字是{name}"
            elif "爱好" in user_message or "喜欢" in user_message:
                if "我喜欢" in content:
                    hobby = content.replace("我喜欢", "").strip()
                    return f"你喜欢{hobby}"
            elif "住" in user_message:
                if "我住在" in content:
                    place = content.replace("我住在", "").strip()
                    return f"你住在{place}"

            return f"我记得：{content}"
        else:
            return "抱歉，我不记得了。"

    async def chat(self, user_message: str) -> str:
        """处理用户消息"""
        # 保存用户消息
        await self.save_message(user_message, role="user")

        # 生成回复
        reply = await self.generate_reply(user_message)

        # 保存机器人回复
        await self.save_message(reply, role="assistant")

        return reply


async def demo_conversation(bot: SimpleChatbot):
    """演示对话"""
    print("💬 演示对话")
    print("---\n")

    conversations = [
        ("我叫 Alice", "很高兴认识你，Alice！"),
        ("我喜欢编程", "编程很棒！"),
        ("我住在上海", "上海是个好地方！"),
    ]

    for user_msg, simple_reply in conversations:
        print(f"   用户: {user_msg}")

        # 保存用户消息
        await bot.save_message(user_msg)

        # 使用简单回复
        print(f"   🤖: {simple_reply}")
        print("   ✅ 已保存记忆\n")

    # 现在测试基于记忆的回复
    questions = [
        "我叫什么名字？",
        "我有什么爱好？",
        "我住在哪里？",
    ]

    for question in questions:
        print(f"   用户: {question}")

        # 生成基于记忆的回复
        reply = await bot.chat(question)

        print(f"   🤖: {reply}")

        # 显示检索到的上下文
        context = await bot.search_context(question)
        if context:
            print(f"   ✅ 检索到: {context[0].get('content', '')}")
        print()


async def interactive_chat(bot: SimpleChatbot):
    """交互式聊天"""
    print("\n💬 交互式聊天（输入 'quit' 退出）")
    print("---\n")

    turn = 1
    while True:
        try:
            user_msg = input(f"   你[{turn}]: ").strip()

            if user_msg.lower() == "quit":
                print("   👋 再见！")
                break

            if not user_msg:
                continue

            # 生成回复
            reply = await bot.chat(user_msg)

            print(f"   🤖: {reply}\n")
            turn += 1

        except KeyboardInterrupt:
            print("\n   👋 再见！")
            break
        except Exception as e:
            print(f"   ❌ 错误: {e}\n")


async def main():
    """主函数"""
    print("🤖 AgentMem 聊天机器人示例\n")
    print("这个示例演示了:")
    print("  1. 对话历史管理")
    print("  2. 上下文检索")
    print("  3. 个性化回复")
    print("  4. 多轮对话")
    print()

    # 初始化客户端
    api_base_url = os.getenv("AGENTMEM_API_BASE_URL", "http://localhost:8080")
    api_key = os.getenv("AGENTMEM_API_KEY", "demo_key")

    config = Config(
        api_base_url=api_base_url,
        api_key=api_key,
    )

    async with AgentMemClient(config) as client:
        print("✅ 初始化完成\n")

        # 创建聊天机器人
        bot = SimpleChatbot(
            client=client,
            user_id="user_demo",
            agent_id="chatbot_demo",
        )

        # 运行演示对话
        await demo_conversation(bot)

        # 可选：交互式聊天
        print("是否开始交互式聊天？(y/n): ", end="")
        try:
            choice = input().strip().lower()
            if choice == 'y':
                await interactive_chat(bot)
        except KeyboardInterrupt:
            print("\n")

        # 显示统计
        all_memories = await client.get_all_memories(
            user_id="user_demo",
            limit=100,
        )

        print("\n📊 对话统计:")
        print(f"   总记忆数: {len(all_memories)}")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n👋 用户中断")
    except Exception as e:
        print(f"\n\n❌ 错误: {e}")
        exit(1)


# ============================================
# 高级示例: 使用 LLM 生成回复
# ============================================
#
# 如果你想使用真实的 LLM 生成回复:
#
# ```python
# import openai
#
# class LLMChatbot(SimpleChatbot):
#     def __init__(self, client, user_id, agent_id, openai_api_key):
#         super().__init__(client, user_id, agent_id)
#         openai.api_key = openai_api_key
#
#     async def generate_reply(self, user_message: str) -> str:
#         # 搜索相关上下文
#         context = await self.search_context(user_message)
#
#         # 构建提示
#         if context:
#             context_text = "\n".join([
#                 f"- {m.get('content', '')}"
#                 for m in context[:3]
#             ])
#             prompt = f"""基于以下上下文回答用户问题:
#
# 上下文:
# {context_text}
#
# 问题: {user_message}
#
# 回答:"""
#         else:
#             prompt = user_message
#
#         # 调用 OpenAI API
#         response = await openai.ChatCompletion.acreate(
#             model="gpt-4",
#             messages=[
#                 {"role": "system", "content": "你是一个有帮助的助手。"},
#                 {"role": "user", "content": prompt}
#             ]
#         )
#
#         return response.choices[0].message.content
# ```
#
# ============================================
# 高级示例: 多轮对话管理
# ============================================
#
# 对于复杂的多轮对话，可以使用会话管理:
#
# ```python
# class ConversationManager:
#     def __init__(self, client: AgentMemClient):
#         self.client = client
#         self.session_id: Optional[str] = None
#
#     async def start_session(self) -> str:
#         """开始新会话"""
#         self.session_id = await self.client.create_session(
#             user_id="user_1",
#             metadata={"started_at": datetime.now().isoformat()}
#         )
#         return self.session_id
#
#     async def add_turn(self, user_message: str, bot_reply: str):
#         """添加对话轮次"""
#         if self.session_id:
#             await self.client.add_memory(
#                 content=f"User: {user_message}\nBot: {bot_reply}",
#                 session_id=self.session_id,
#             )
#
#     async def get_conversation_history(self) -> List[dict]:
#         """获取对话历史"""
#         if self.session_id:
#             return await self.client.get_session_memories(self.session_id)
#         return []
# ```
