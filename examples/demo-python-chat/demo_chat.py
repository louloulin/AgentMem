#!/usr/bin/env python3
"""
AgentMem Python SDK 智能对话演示

展示在对话场景中的应用：
1. 多轮对话记忆
2. 上下文理解
3. 用户偏好学习
4. 智能推荐
"""

import asyncio
from datetime import datetime


class IntelligentChatBot:
    """智能对话机器人"""
    
    def __init__(self, memory):
        """初始化"""
        self.memory = memory
        self.conversation_history = []
    
    async def user_says(self, message: str):
        """用户发送消息"""
        print(f"用户: {message}")
        
        # 保存到记忆系统
        await self.memory.add(f"用户说：{message}")
        
        # 添加到对话历史
        self.conversation_history.append({
            'role': 'user',
            'content': message,
            'timestamp': datetime.now().isoformat()
        })
    
    async def bot_says(self, message: str):
        """机器人回复"""
        print(f"助手: {message}")
        
        # 添加到对话历史
        self.conversation_history.append({
            'role': 'assistant',
            'content': message,
            'timestamp': datetime.now().isoformat()
        })
    
    async def recall(self, query: str):
        """回忆相关记忆"""
        print(f"\n🔍 回忆相关记忆: \"{query}\"")
        results = await self.memory.search(query)
        
        if results:
            print(f"✅ 找到 {len(results)} 条相关记忆:")
            for i, mem in enumerate(results[:3], 1):
                content = mem.get('content', 'N/A')
                print(f"   {i}. {content}")
            return results
        else:
            print("ℹ️  未找到相关记忆")
            return []
    
    async def analyze_user_interests(self):
        """分析用户兴趣"""
        print("\n🔍 分析用户兴趣...")
        
        # 获取所有记忆
        all_memories = await self.memory.get_all()
        
        if all_memories:
            print("✅ 用户画像分析：")
            
            # 简单的关键词统计
            keywords = {}
            for mem in all_memories:
                content = mem.get('content', '').lower()
                
                # 统计关键词
                if 'python' in content or '编程' in content:
                    keywords['编程'] = keywords.get('编程', 0) + 1
                if 'ai' in content or '人工智能' in content:
                    keywords['AI'] = keywords.get('AI', 0) + 1
                if '性能' in content or 'performance' in content:
                    keywords['性能'] = keywords.get('性能', 0) + 1
                if '产品' in content or 'product' in content:
                    keywords['产品'] = keywords.get('产品', 0) + 1
            
            # 显示兴趣点
            for keyword, count in sorted(keywords.items(), key=lambda x: x[1], reverse=True):
                print(f"   • 关注{keyword}（提及{count}次）")
            
            return keywords
        else:
            print("ℹ️  暂无足够数据进行分析")
            return {}


async def main():
    """主函数"""
    print("🤖 AgentMem Python SDK 智能对话演示\n")
    print("这是一个智能客服场景，展示AgentMem如何：")
    print("  ✅ 记住用户的对话历史")
    print("  ✅ 学习用户的偏好")
    print("  ✅ 提供个性化服务\n")
    
    # 导入AgentMem
    try:
        import agentmem_native
        print("✅ 成功导入 agentmem_native\n")
    except ImportError:
        print("❌ 导入失败，请先构建Python绑定：")
        print("   cd crates/agent-mem-python")
        print("   maturin develop\n")
        return
    
    # 创建Memory和ChatBot
    print("正在初始化对话系统...")
    memory = agentmem_native.Memory()
    bot = IntelligentChatBot(memory)
    print("✅ 对话系统初始化完成\n")
    
    # === 场景1：首次咨询 ===
    print("=" * 50)
    print("场景1：首次咨询")
    print("=" * 50)
    print()
    
    await bot.user_says("你好，我想了解一下你们的产品")
    await bot.bot_says("您好！很高兴为您服务。我们提供AgentMem AI记忆管理平台。")
    
    await bot.user_says("我对AI记忆管理系统很感兴趣")
    await bot.bot_says("太好了！AgentMem是业界领先的AI记忆管理平台，支持多模态、高性能。")
    
    await bot.user_says("它有什么特点？")
    await bot.bot_says("主要特点有：Rust高性能、多模态支持、知识图谱、零配置启动。")
    
    print("\n💾 对话已保存到记忆系统\n")
    
    # === 场景2：第二天继续咨询 ===
    print("=" * 50)
    print("场景2：第二天继续咨询")
    print("=" * 50)
    print()
    
    await bot.user_says("你好，我昨天问过你产品的事")
    
    # 搜索历史记忆
    memories = await bot.recall("产品 AgentMem")
    print()
    
    if memories:
        await bot.bot_says("您好！我记得您昨天咨询过我们的AgentMem产品。您想了解更多详情吗？")
    else:
        await bot.bot_says("您好！请问您想了解哪方面的信息？")
    
    await bot.user_says("是的，我想知道性能如何")
    await bot.bot_says("AgentMem采用Rust实现，性能比Python方案快2-10倍，支持高并发。")
    
    await bot.user_says("支持哪些功能？")
    await bot.bot_says("支持：多模态处理、知识图谱、BM25搜索、程序记忆、完整监控。")
    
    print()
    
    # === 场景3：个性化推荐 ===
    print("=" * 50)
    print("场景3：个性化推荐")
    print("=" * 50)
    print()
    
    # 分析用户兴趣
    interests = await bot.analyze_user_interests()
    print()
    
    # 基于兴趣推荐
    print("🎯 个性化推荐：")
    await bot.bot_says("基于您的兴趣，我推荐您关注：")
    print("  1️⃣ AgentMem性能基准测试报告")
    print("  2️⃣ 多模态功能演示视频")
    print("  3️⃣ 技术架构深度解析")
    print("  4️⃣ 企业级部署指南")
    print()
    
    await bot.user_says("太好了，请发给我技术文档")
    await bot.bot_says("好的，已为您发送技术文档链接。有问题随时咨询！")
    
    print()
    
    # === 对话历史统计 ===
    print("=" * 50)
    print("对话统计")
    print("=" * 50)
    print()
    
    print(f"📊 对话统计：")
    print(f"   • 总消息数: {len(bot.conversation_history)}")
    print(f"   • 用户消息: {sum(1 for m in bot.conversation_history if m['role'] == 'user')}")
    print(f"   • 助手回复: {sum(1 for m in bot.conversation_history if m['role'] == 'assistant')}")
    
    # 显示所有记忆
    all_memories = await bot.memory.get_all()
    print(f"   • 保存记忆: {len(all_memories)} 条")
    
    print()
    print("🎉 演示完成！\n")
    print("📊 AgentMem在智能对话中的优势：")
    print("  ✅ 长期记忆：跨会话保持用户信息")
    print("  ✅ 语义理解：智能搜索相关对话")
    print("  ✅ 个性化：基于历史提供定制服务")
    print("  ✅ 上下文感知：理解对话连贯性")


if __name__ == "__main__":
    asyncio.run(main())

