#!/usr/bin/env python3
"""
AgentMem Python SDK - 个人助理示例

这个示例演示了如何构建一个智能个人助理：
- 任务管理
- 日程安排
- 信息检索
- 个性化建议

运行方式:
```bash
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key

python personal_assistant.py
```

预期输出:
```
👤 AgentMem 个人助理示例

✅ 初始化完成

📋 任务管理:
   ✅ 添加任务: "完成项目报告" -> task_001
   ✅ 添加任务: "准备会议材料" -> task_002
   ✅ 添加任务: "回复邮件" -> task_003

📅 日程安排:
   ✅ 添加日程: "明天上午10点开会" -> event_001
   ✅ 添加日程: "周五下午3点面试" -> event_002

💡 智能建议:
   搜索: "会议"
   ✅ 找到相关内容:
      - 准备会议材料 (任务)
      - 明天上午10点开会 (日程)

🎯 个人提醒:
   用户: "我今天要做什么？"
   🤖: 根据记忆，你有以下任务:
      1. 完成项目报告
      2. 准备会议材料
      3. 回复邮件

🎉 完成！
```
"""

import asyncio
import os
from typing import List, Dict, Optional
from datetime import datetime, timedelta
from enum import Enum

try:
    from agentmem import AgentMemClient, Config, SearchQuery, MemoryType
except ImportError:
    print("⚠️  AgentMem SDK 未安装")
    print("   安装方式: pip install agentmem")
    exit(1)


class ItemType(Enum):
    """记忆项类型"""
    TASK = "task"
    EVENT = "event"
    NOTE = "note"
    CONTACT = "contact"
    IDEA = "idea"


class PersonalAssistant:
    """个人助理"""

    def __init__(self, client: AgentMemClient, user_id: str):
        """初始化个人助理"""
        self.client = client
        self.user_id = user_id
        self.agent_id = "personal_assistant"

    async def add_task(self, task: str, priority: str = "medium") -> str:
        """添加任务"""
        memory_id = await self.client.add_memory(
            content=f"任务: {task}",
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=MemoryType.EPISODIC,
            metadata={
                "type": ItemType.TASK.value,
                "priority": priority,
                "status": "pending",
                "created_at": datetime.now().isoformat(),
            },
        )
        return memory_id

    async def add_event(self, event: str, event_time: str) -> str:
        """添加日程"""
        memory_id = await self.client.add_memory(
            content=f"日程: {event} - {event_time}",
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=MemoryType.EPISODIC,
            metadata={
                "type": ItemType.EVENT.value,
                "event_time": event_time,
                "created_at": datetime.now().isoformat(),
            },
        )
        return memory_id

    async def add_note(self, note: str) -> str:
        """添加笔记"""
        memory_id = await self.client.add_memory(
            content=f"笔记: {note}",
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=MemoryType.SEMANTIC,
            metadata={
                "type": ItemType.NOTE.value,
                "created_at": datetime.now().isoformat(),
            },
        )
        return memory_id

    async def add_contact(self, name: str, info: str) -> str:
        """添加联系人"""
        memory_id = await self.client.add_memory(
            content=f"联系人: {name} - {info}",
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=MemoryType.SEMANTIC,
            metadata={
                "type": ItemType.CONTACT.value,
                "name": name,
                "info": info,
                "created_at": datetime.now().isoformat(),
            },
        )
        return memory_id

    async def get_tasks(self) -> List[dict]:
        """获取所有任务"""
        all_memories = await self.client.get_all_memories(
            user_id=self.user_id,
            limit=100,
        )

        tasks = [
            m for m in all_memories
            if m.get("metadata", {}).get("type") == ItemType.TASK.value
        ]

        return tasks

    async def get_events(self) -> List[dict]:
        """获取所有日程"""
        all_memories = await self.client.get_all_memories(
            user_id=self.user_id,
            limit=100,
        )

        events = [
            m for m in all_memories
            if m.get("metadata", {}).get("type") == ItemType.EVENT.value
        ]

        return events

    async def search_all(self, query: str) -> List[dict]:
        """搜索所有相关内容"""
        search_query = SearchQuery(
            query=query,
            user_id=self.user_id,
            limit=10,
            threshold=0.6,
        )
        results = await self.client.search_memories(search_query)
        return results

    async def get_suggestions(self) -> Dict[str, List[dict]]:
        """获取智能建议"""
        suggestions = {
            "tasks": await self.get_tasks(),
            "events": await self.get_events(),
            "recent": await self.search_all("今天"),
        }
        return suggestions

    async def answer_question(self, question: str) -> str:
        """回答问题"""
        context = await self.search_all(question)

        if not context:
            return "抱歉，我没有找到相关信息。"

        # 根据问题类型生成回复
        if "任务" in question or "做什么" in question:
            tasks = await self.get_tasks()
            if tasks:
                task_list = "\n".join([
                    f"   {i+1}. {m.get('content', '').replace('任务: ', '')}"
                    for i, m in enumerate(tasks[:5])
                ])
                return f"根据记忆，你有以下任务:\n{task_list}"
            else:
                return "你没有待办任务。"

        elif "日程" in question or "安排" in question or "会议" in question:
            events = await self.get_events()
            if events:
                event_list = "\n".join([
                    f"   {i+1}. {m.get('content', '').replace('日程: ', '')}"
                    for i, m in enumerate(events[:5])
                ])
                return f"你的日程安排:\n{event_list}"
            else:
                return "你没有即将到来的日程。"

        else:
            # 返回最相关的记忆
            top_result = context[0]
            return f"我记得: {top_result.get('content', '')}"


async def demo_task_management(assistant: PersonalAssistant):
    """演示任务管理"""
    print("\n📋 任务管理")
    print("---")

    tasks = [
        ("完成项目报告", "high"),
        ("准备会议材料", "medium"),
        ("回复邮件", "low"),
    ]

    for task, priority in tasks:
        memory_id = await assistant.add_task(task, priority)
        print(f"   ✅ 添加任务: \"{task}\" -> {memory_id}")

    # 显示所有任务
    tasks = await assistant.get_tasks()
    print(f"\n   📊 共有 {len(tasks)} 个任务")


async def demo_schedule_management(assistant: PersonalAssistant):
    """演示日程管理"""
    print("\n📅 日程安排")
    print("---")

    events = [
        ("明天上午10点开会", "2025-01-02T10:00:00"),
        ("周五下午3点面试", "2025-01-03T15:00:00"),
    ]

    for event, event_time in events:
        memory_id = await assistant.add_event(event, event_time)
        print(f"   ✅ 添加日程: \"{event}\" -> {memory_id}")

    # 显示所有日程
    events = await assistant.get_events()
    print(f"\n   📊 共有 {len(events)} 个日程")


async def demo_smart_search(assistant: PersonalAssistant):
    """演示智能搜索"""
    print("\n💡 智能搜索")
    print("---")

    queries = [
        ("会议", "搜索会议相关内容"),
        ("报告", "搜索报告相关内容"),
        ("邮件", "搜索邮件相关内容"),
    ]

    for query, description in queries:
        print(f"\n   搜索: \"{query}\" ({description})")

        results = await assistant.search_all(query)
        print(f"   ✅ 找到 {len(results)} 条相关内容:")

        for i, result in enumerate(results[:3], 1):
            content = result.get("content", "")
            metadata = result.get("metadata", {})
            item_type = metadata.get("type", "unknown")
            score = result.get("score", 0.0)
            print(f"      {i}. [{item_type}] {content} (相似度: {score:.2f})")


async def demo_qa(assistant: PersonalAssistant):
    """演示问答功能"""
    print("\n🎯 智能问答")
    print("---")

    questions = [
        "我今天要做什么？",
        "我有什么会议？",
        "我的优先任务是什么？",
    ]

    for question in questions:
        print(f"\n   用户: \"{question}\"")
        answer = await assistant.answer_question(question)
        print(f"   🤖: {answer}")


async def demo_notes_and_contacts(assistant: PersonalAssistant):
    """演示笔记和联系人"""
    print("\n📝 笔记和联系人")
    print("---")

    # 添加笔记
    notes = [
        "记住：重要项目截止日期是下周五",
        "新想法：使用 AgentMem 构建知识管理系统",
    ]

    for note in notes:
        memory_id = await assistant.add_note(note)
        print(f"   ✅ 添加笔记: \"{note}\"")

    # 添加联系人
    contacts = [
        ("张三", "项目经理，电话: 138-xxxx-xxxx"),
        ("李四", "技术总监，邮箱: lisi@example.com"),
    ]

    for name, info in contacts:
        memory_id = await assistant.add_contact(name, info)
        print(f"   ✅ 添加联系人: \"{name}\" - {info}")


async def main():
    """主函数"""
    print("👤 AgentMem 个人助理示例\n")
    print("这个示例演示了:")
    print("  1. 任务管理")
    print("  2. 日程安排")
    print("  3. 信息检索")
    print("  4. 个性化建议")
    print("  5. 智能问答")
    print()

    # 初始化客户端
    api_base_url = os.getenv("AGENTMEM_API_BASE_URL", "http://localhost:8080")
    api_key = os.getenv("AGENTMEM_API_KEY", "demo_key")

    config = Config(
        api_base_url=api_base_url,
        api_key=api_key,
    )

    async with AgentMemClient(config) as client:
        print("✅ 初始化完成")

        # 创建个人助理
        assistant = PersonalAssistant(
            client=client,
            user_id="user_demo",
        )

        # 演示各种功能
        await demo_task_management(assistant)
        await demo_schedule_management(assistant)
        await demo_notes_and_contacts(assistant)
        await demo_smart_search(assistant)
        await demo_qa(assistant)

        # 显示统计
        all_memories = await client.get_all_memories(
            user_id="user_demo",
            limit=100,
        )

        print("\n📊 使用统计:")
        print(f"   总记忆数: {len(all_memories)}")

        # 按类型统计
        type_counts = {}
        for memory in all_memories:
            item_type = memory.get("metadata", {}).get("type", "unknown")
            type_counts[item_type] = type_counts.get(item_type, 0) + 1

        print("\n   按类型统计:")
        for item_type, count in sorted(type_counts.items()):
            print(f"   - {item_type}: {count}")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n👋 用户中断")
    except Exception as e:
        print(f"\n\n❌ 错误: {e}")
        exit(1)


# ============================================
# 高级功能: 智能提醒
# ============================================
#
# 实现基于时间和优先级的智能提醒:
#
# ```python
# async def check_reminders(assistant: PersonalAssistant):
#     """检查并提醒"""
#     tasks = await assistant.get_tasks()
#
#     # 找出高优先级任务
#     high_priority = [
#         t for t in tasks
#         if t.get("metadata", {}).get("priority") == "high"
#         and t.get("metadata", {}).get("status") == "pending"
#     ]
#
#     if high_priority:
#         print("⚠️  高优先级任务提醒:")
#         for task in high_priority:
#             content = task.get("content", "").replace("任务: ", "")
#             print(f"   - {content}")
#
#     # 找出即将到来的事件
#     events = await assistant.get_events()
#     now = datetime.now()
#
#     upcoming = []
#     for event in events:
#         event_time_str = event.get("metadata", {}).get("event_time")
#         if event_time_str:
#             event_time = datetime.fromisoformat(event_time_str)
#             if event_time <= now + timedelta(hours=24):
#                 upcoming.append(event)
#
#     if upcoming:
#         print("\n📅 即将到来的事件:")
#         for event in upcoming:
#             content = event.get("content", "").replace("日程: ", "")
#             print(f"   - {content}")
# ```
#
# ============================================
# 高级功能: 任务完成跟踪
# ============================================
#
# ```python
# async def complete_task(assistant: PersonalAssistant, memory_id: str):
#     """标记任务为完成"""
#     await assistant.client.update_memory(
#         memory_id=memory_id,
#         metadata={"status": "completed"}
#     )
#
# async def get_productivity_stats(assistant: PersonalAssistant):
#     """获取生产力统计"""
#     tasks = await assistant.get_tasks()
#
#     completed = sum(
#         1 for t in tasks
#         if t.get("metadata", {}).get("status") == "completed"
#     )
#
#     total = len(tasks)
#     completion_rate = (completed / total * 100) if total > 0 else 0
#
#     print(f"📊 生产力统计:")
#     print(f"   总任务: {total}")
#     print(f"   已完成: {completed}")
#     print(f"   完成率: {completion_rate:.1f}%")
# ```
