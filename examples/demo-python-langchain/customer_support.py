"""
AgentMem + LangGraph 集成示例 - 客服对话场景

功能:
1. LangGraph状态图集成
2. AgentMem记忆管理
3. 多用户支持
4. 对话历史管理
5. 记忆检索和上下文注入

真实实现，对标MIRIX的langgraph_integration.py
"""

import os
import sys
from typing import Annotated, List, Dict, Any
from datetime import datetime

# LangGraph和LangChain导入
try:
    from langgraph.graph import StateGraph, START, END
    from langgraph.graph.message import add_messages
    from langchain_core.messages import SystemMessage, HumanMessage, AIMessage, BaseMessage
except ImportError:
    print("❌ 需要安装 langgraph 和 langchain-core:")
    print("   pip install langgraph langchain-core")
    sys.exit(1)

# LLM提供商导入（支持多个）
try:
    from langchain_openai import ChatOpenAI
    HAS_OPENAI = True
except ImportError:
    HAS_OPENAI = False

try:
    from langchain_anthropic import ChatAnthropic
    HAS_ANTHROPIC = True
except ImportError:
    HAS_ANTHROPIC = False

# AgentMem导入
try:
    # 假设Python绑定已编译
    import agent_mem_python as amp
    HAS_AGENTMEM = True
except ImportError:
    print("⚠️  AgentMem Python绑定未找到，使用模拟模式")
    HAS_AGENTMEM = False

# 彩色输出
try:
    from colorama import init, Fore, Style
    init()
    HAS_COLOR = True
except ImportError:
    HAS_COLOR = False
    class Fore:
        RED = GREEN = YELLOW = CYAN = BLUE = ""
    class Style:
        RESET_ALL = ""


class AgentMemAdapter:
    """AgentMem适配器 - 处理真实和模拟模式"""
    
    def __init__(self, agent_id: str = "customer_support"):
        self.agent_id = agent_id
        self.users: Dict[str, str] = {}  # user_name -> user_id
        self.memories: Dict[str, List[Dict]] = {}  # user_id -> memories
        
        if HAS_AGENTMEM:
            # 使用真实的AgentMem
            self.memory = amp.Memory(agent_id)
            print(f"{Fore.GREEN}✓ 使用真实AgentMem{Style.RESET_ALL}")
        else:
            # 模拟模式
            self.memory = None
            print(f"{Fore.YELLOW}⚠️  使用模拟AgentMem{Style.RESET_ALL}")
    
    def create_user(self, user_name: str) -> Dict[str, str]:
        """创建用户"""
        if user_name in self.users:
            user_id = self.users[user_name]
            return {"id": user_id, "name": user_name}
        
        user_id = f"user_{len(self.users) + 1}"
        self.users[user_name] = user_id
        self.memories[user_id] = []
        print(f"{Fore.GREEN}✓ 创建用户: {user_name} (ID: {user_id}){Style.RESET_ALL}")
        return {"id": user_id, "name": user_name}
    
    def add_memory(self, content: str, user_id: str):
        """添加记忆"""
        if HAS_AGENTMEM and self.memory:
            # 真实AgentMem
            self.memory.add(content, user_id=user_id)
        else:
            # 模拟模式
            if user_id not in self.memories:
                self.memories[user_id] = []
            self.memories[user_id].append({
                "content": content,
                "timestamp": datetime.now().isoformat()
            })
        print(f"{Fore.CYAN}📝 记忆已保存: {content[:50]}...{Style.RESET_ALL}")
    
    def extract_memories(self, query: str, user_id: str, limit: int = 5) -> str:
        """提取相关记忆"""
        if HAS_AGENTMEM and self.memory:
            # 真实AgentMem - 使用搜索
            try:
                results = self.memory.search(query, user_id=user_id, limit=limit)
                if results:
                    memories_text = "\n".join([
                        f"- {item['content'][:100]}..."
                        for item in results[:limit]
                    ])
                    return f"相关记忆:\n{memories_text}"
                return "没有找到相关记忆。"
            except Exception as e:
                print(f"{Fore.RED}⚠️  搜索失败: {e}{Style.RESET_ALL}")
                return "没有找到相关记忆。"
        else:
            # 模拟模式 - 简单返回最近的记忆
            if user_id not in self.memories or not self.memories[user_id]:
                return "没有找到相关记忆。"
            
            recent = self.memories[user_id][-limit:]
            memories_text = "\n".join([
                f"- {mem['content'][:100]}..."
                for mem in recent
            ])
            return f"相关记忆:\n{memories_text}"


class State(dict):
    """LangGraph状态"""
    messages: Annotated[List[BaseMessage], add_messages]
    user_id: str
    user_name: str


def create_chatbot(agentmem: AgentMemAdapter, llm: Any):
    """创建chatbot节点"""
    
    def chatbot(state: State) -> Dict[str, Any]:
        messages = state["messages"]
        user_id = state["user_id"]
        user_name = state.get("user_name", "Customer")
        
        try:
            # 1. 提取相关记忆
            last_message = messages[-1].content if messages else ""
            memories = agentmem.extract_memories(last_message, user_id)
            
            # 2. 构建系统提示
            system_message = f"""你是一个专业的客服助手，名为AgentMem Support。
你的目标是帮助客户解决问题，提供友好和专业的服务。

客户信息:
- 姓名: {user_name}
- ID: {user_id}

{memories}

请基于以上记忆和客户的问题，提供有用的回答。
"""
            
            full_messages = [SystemMessage(content=system_message)] + messages
            
            # 3. 调用LLM
            response = llm.invoke(full_messages)
            
            # 4. 保存对话记忆
            try:
                interaction = f"用户问题: {last_message}\n\n助手回答: {response.content}"
                agentmem.add_memory(interaction, user_id)
            except Exception as e:
                print(f"{Fore.YELLOW}⚠️  保存记忆失败: {e}{Style.RESET_ALL}")
            
            return {"messages": [response]}
            
        except Exception as e:
            print(f"{Fore.RED}❌ Chatbot错误: {e}{Style.RESET_ALL}")
            # 降级响应
            fallback = AIMessage(content="抱歉，我遇到了一个问题。请稍后再试。")
            return {"messages": [fallback]}
    
    return chatbot


def create_graph(agentmem: AgentMemAdapter, llm: Any):
    """创建LangGraph"""
    graph = StateGraph(State)
    
    # 添加chatbot节点
    chatbot = create_chatbot(agentmem, llm)
    graph.add_node("chatbot", chatbot)
    
    # 添加边
    graph.add_edge(START, "chatbot")
    graph.add_edge("chatbot", END)
    
    return graph.compile()


def run_conversation(graph: Any, agentmem: AgentMemAdapter, user_input: str, user_name: str):
    """运行单次对话"""
    # 获取或创建用户
    user = agentmem.create_user(user_name)
    user_id = user["id"]
    
    # 准备状态
    state = {
        "messages": [HumanMessage(content=user_input)],
        "user_id": user_id,
        "user_name": user_name
    }
    
    print(f"\n{Fore.BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Style.RESET_ALL}")
    print(f"{Fore.CYAN}👤 {user_name}:{Style.RESET_ALL} {user_input}")
    
    # 执行图
    try:
        for event in graph.stream(state):
            for value in event.values():
                if value is not None and value.get("messages"):
                    last_message = value["messages"][-1]
                    if isinstance(last_message, AIMessage):
                        print(f"{Fore.GREEN}🤖 AgentMem Support:{Style.RESET_ALL} {last_message.content}")
    except Exception as e:
        print(f"{Fore.RED}❌ 对话执行失败: {e}{Style.RESET_ALL}")


def main():
    """主函数"""
    print(f"""
{Fore.CYAN}╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║        🤖 AgentMem + LangGraph 客服对话演示 🤖               ║
║                                                                ║
║          真实实现，对标MIRIX LangGraph集成                   ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝{Style.RESET_ALL}
""")
    
    # 1. 初始化AgentMem
    print(f"\n{Fore.CYAN}🚀 初始化AgentMem...{Style.RESET_ALL}")
    agentmem = AgentMemAdapter("customer_support")
    
    # 2. 初始化LLM
    print(f"\n{Fore.CYAN}🚀 初始化LLM...{Style.RESET_ALL}")
    llm = None
    
    # 尝试OpenAI
    if HAS_OPENAI and os.getenv("OPENAI_API_KEY"):
        llm = ChatOpenAI(model="gpt-3.5-turbo", temperature=0.7)
        print(f"{Fore.GREEN}✓ 使用 OpenAI GPT-3.5-Turbo{Style.RESET_ALL}")
    # 尝试Anthropic
    elif HAS_ANTHROPIC and os.getenv("ANTHROPIC_API_KEY"):
        llm = ChatAnthropic(model="claude-3-sonnet-20240229", temperature=0.7)
        print(f"{Fore.GREEN}✓ 使用 Anthropic Claude-3-Sonnet{Style.RESET_ALL}")
    else:
        print(f"{Fore.RED}❌ 未找到可用的LLM提供商{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}请设置以下环境变量之一:{Style.RESET_ALL}")
        print("  - OPENAI_API_KEY (需要安装 langchain-openai)")
        print("  - ANTHROPIC_API_KEY (需要安装 langchain-anthropic)")
        sys.exit(1)
    
    # 3. 创建LangGraph
    print(f"\n{Fore.CYAN}🚀 创建LangGraph...{Style.RESET_ALL}")
    graph = create_graph(agentmem, llm)
    print(f"{Fore.GREEN}✓ LangGraph创建成功{Style.RESET_ALL}")
    
    # 4. 演示对话
    print(f"\n{Fore.CYAN}💬 开始客服对话演示...{Style.RESET_ALL}")
    
    user_name = "Alice"
    
    # 对话1: 初次咨询
    run_conversation(
        graph, agentmem,
        "你好，我想了解一下你们的产品功能。",
        user_name
    )
    
    # 对话2: 技术问题
    run_conversation(
        graph, agentmem,
        "我遇到了一个登录问题，总是提示密码错误。",
        user_name
    )
    
    # 对话3: 后续咨询
    run_conversation(
        graph, agentmem,
        "刚才你提到的解决方案我试了，但还是不行。",
        user_name
    )
    
    # 5. 交互式对话（可选）
    print(f"\n{Fore.CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Style.RESET_ALL}")
    print(f"{Fore.CYAN}🔍 进入交互式对话模式{Style.RESET_ALL}")
    print(f"{Fore.YELLOW}提示: 输入 'quit', 'exit' 或 'bye' 退出{Style.RESET_ALL}")
    
    while True:
        try:
            user_input = input(f"\n{Fore.CYAN}你:{Style.RESET_ALL} ").strip()
            
            if not user_input:
                continue
            
            if user_input.lower() in ['quit', 'exit', 'bye', 'q']:
                print(f"\n{Fore.GREEN}👋 感谢使用AgentMem客服系统！再见！{Style.RESET_ALL}")
                break
            
            run_conversation(graph, agentmem, user_input, user_name)
            
        except KeyboardInterrupt:
            print(f"\n\n{Fore.YELLOW}⚠️  收到中断信号{Style.RESET_ALL}")
            break
        except EOFError:
            print(f"\n\n{Fore.YELLOW}⚠️  输入结束{Style.RESET_ALL}")
            break
    
    print(f"\n{Fore.CYAN}╔════════════════════════════════════════════════════════════════╗")
    print(f"║                                                                ║")
    print(f"║           ✨ AgentMem + LangGraph 演示完成！✨              ║")
    print(f"║                                                                ║")
    print(f"╚════════════════════════════════════════════════════════════════╝{Style.RESET_ALL}\n")


if __name__ == "__main__":
    main()

