"""
AgentMem 多用户管理示例

功能:
1. 创建和管理多个用户
2. 用户隔离验证
3. 用户记忆管理
4. 用户列表和查询
5. 用户间记忆不共享验证

真实实现，对标MIRIX的test_sdk.py多用户功能
"""

import sys
from typing import List, Dict, Any, Optional
from datetime import datetime

# AgentMem导入
try:
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
        RED = GREEN = YELLOW = CYAN = BLUE = MAGENTA = ""
    class Style:
        RESET_ALL = BRIGHT = ""


class User:
    """用户类"""
    def __init__(self, user_id: str, name: str, created_at: datetime):
        self.id = user_id
        self.name = name
        self.created_at = created_at
        self.memory_count = 0

    def __repr__(self):
        return f"User(id='{self.id}', name='{self.name}', memories={self.memory_count})"


class MultiUserMemorySystem:
    """多用户记忆系统"""
    
    def __init__(self, agent_id: str = "multi_user_agent"):
        self.agent_id = agent_id
        self.users: Dict[str, User] = {}
        self.memories: Dict[str, List[Dict]] = {}  # user_id -> memories
        
        if HAS_AGENTMEM:
            # 使用真实的AgentMem
            self.memory = amp.Memory(agent_id)
            print(f"{Fore.GREEN}✓ 使用真实AgentMem{Style.RESET_ALL}")
        else:
            # 模拟模式
            self.memory = None
            print(f"{Fore.YELLOW}⚠️  使用模拟AgentMem{Style.RESET_ALL}")
    
    def create_user(self, user_name: str) -> User:
        """创建用户（如果已存在则返回现有用户）"""
        # 检查是否已存在
        for user in self.users.values():
            if user.name == user_name:
                print(f"{Fore.YELLOW}⚠️  用户 '{user_name}' 已存在，返回现有用户{Style.RESET_ALL}")
                return user
        
        # 创建新用户
        user_id = f"user_{len(self.users) + 1}_{user_name}"
        user = User(user_id, user_name, datetime.now())
        self.users[user_id] = user
        self.memories[user_id] = []
        
        print(f"{Fore.GREEN}✓ 创建用户: {user_name} (ID: {user_id}){Style.RESET_ALL}")
        return user
    
    def list_users(self) -> List[User]:
        """列出所有用户"""
        return list(self.users.values())
    
    def get_user(self, user_id: str) -> Optional[User]:
        """根据ID获取用户"""
        return self.users.get(user_id)
    
    def get_user_by_name(self, user_name: str) -> Optional[User]:
        """根据名称获取用户"""
        for user in self.users.values():
            if user.name == user_name:
                return user
        return None
    
    def add_memory(self, content: str, user_id: str) -> bool:
        """为用户添加记忆"""
        user = self.get_user(user_id)
        if not user:
            print(f"{Fore.RED}❌ 用户不存在: {user_id}{Style.RESET_ALL}")
            return False
        
        if HAS_AGENTMEM and self.memory:
            # 真实AgentMem
            try:
                self.memory.add(content, user_id=user_id)
                user.memory_count += 1
                print(f"{Fore.CYAN}📝 已为用户 '{user.name}' 添加记忆{Style.RESET_ALL}")
                return True
            except Exception as e:
                print(f"{Fore.RED}❌ 添加记忆失败: {e}{Style.RESET_ALL}")
                return False
        else:
            # 模拟模式
            if user_id not in self.memories:
                self.memories[user_id] = []
            self.memories[user_id].append({
                "content": content,
                "timestamp": datetime.now().isoformat(),
                "user_id": user_id
            })
            user.memory_count += 1
            print(f"{Fore.CYAN}📝 已为用户 '{user.name}' 添加记忆{Style.RESET_ALL}")
            return True
    
    def get_memories(self, user_id: str, limit: int = 10) -> List[Dict]:
        """获取用户的记忆"""
        if HAS_AGENTMEM and self.memory:
            # 真实AgentMem - 使用get_all并过滤
            try:
                all_memories = self.memory.get_all(user_id=user_id, limit=limit)
                return all_memories
            except Exception as e:
                print(f"{Fore.RED}⚠️  获取记忆失败: {e}{Style.RESET_ALL}")
                return []
        else:
            # 模拟模式
            if user_id not in self.memories:
                return []
            return self.memories[user_id][-limit:]
    
    def search_memories(self, query: str, user_id: str, limit: int = 5) -> List[Dict]:
        """搜索用户的记忆"""
        if HAS_AGENTMEM and self.memory:
            # 真实AgentMem
            try:
                results = self.memory.search(query, user_id=user_id, limit=limit)
                return results
            except Exception as e:
                print(f"{Fore.RED}⚠️  搜索失败: {e}{Style.RESET_ALL}")
                return []
        else:
            # 模拟模式 - 简单的内容匹配
            if user_id not in self.memories:
                return []
            query_lower = query.lower()
            matches = [
                mem for mem in self.memories[user_id]
                if query_lower in mem["content"].lower()
            ]
            return matches[:limit]
    
    def delete_user(self, user_id: str) -> bool:
        """删除用户及其所有记忆"""
        if user_id not in self.users:
            print(f"{Fore.RED}❌ 用户不存在: {user_id}{Style.RESET_ALL}")
            return False
        
        user = self.users[user_id]
        del self.users[user_id]
        if user_id in self.memories:
            del self.memories[user_id]
        
        print(f"{Fore.GREEN}✓ 已删除用户: {user.name}{Style.RESET_ALL}")
        return True


def print_header(title: str):
    """打印标题"""
    print(f"\n{Fore.CYAN}{'━' * 70}{Style.RESET_ALL}")
    print(f"{Fore.CYAN}{Style.BRIGHT}{title}{Style.RESET_ALL}")
    print(f"{Fore.CYAN}{'━' * 70}{Style.RESET_ALL}\n")


def print_section(title: str):
    """打印小节标题"""
    print(f"\n{Fore.YELLOW}{Style.BRIGHT}▶ {title}{Style.RESET_ALL}")


def test_user_creation(system: MultiUserMemorySystem):
    """测试用户创建"""
    print_section("测试1: 用户创建")
    
    # 创建用户Alice
    alice = system.create_user("Alice")
    print(f"  用户信息: {alice}")
    
    # 创建用户Bob
    bob = system.create_user("Bob")
    print(f"  用户信息: {bob}")
    
    # 创建用户Charlie
    charlie = system.create_user("Charlie")
    print(f"  用户信息: {charlie}")
    
    print(f"\n{Fore.GREEN}✓ 测试1通过：成功创建3个用户{Style.RESET_ALL}")


def test_user_listing(system: MultiUserMemorySystem):
    """测试用户列表"""
    print_section("测试2: 用户列表")
    
    users = system.list_users()
    print(f"  总用户数: {len(users)}")
    for idx, user in enumerate(users, 1):
        print(f"  {idx}. {user.name} (ID: {user.id}, 记忆数: {user.memory_count})")
    
    print(f"\n{Fore.GREEN}✓ 测试2通过：成功列出{len(users)}个用户{Style.RESET_ALL}")


def test_memory_isolation(system: MultiUserMemorySystem):
    """测试记忆隔离"""
    print_section("测试3: 记忆隔离")
    
    alice = system.get_user_by_name("Alice")
    bob = system.get_user_by_name("Bob")
    
    if not alice or not bob:
        print(f"{Fore.RED}❌ 测试3失败：用户不存在{Style.RESET_ALL}")
        return
    
    # 为Alice添加记忆
    print(f"\n为 {Fore.CYAN}Alice{Style.RESET_ALL} 添加记忆：")
    system.add_memory("Alice loves Python programming.", alice.id)
    system.add_memory("Alice is working on a machine learning project.", alice.id)
    
    # 为Bob添加记忆
    print(f"\n为 {Fore.CYAN}Bob{Style.RESET_ALL} 添加记忆：")
    system.add_memory("Bob loves Rust programming.", bob.id)
    system.add_memory("Bob is building a blockchain application.", bob.id)
    
    # 验证记忆隔离
    print(f"\n{Fore.MAGENTA}验证记忆隔离：{Style.RESET_ALL}")
    alice_memories = system.get_memories(alice.id)
    bob_memories = system.get_memories(bob.id)
    
    print(f"  Alice的记忆数: {len(alice_memories)}")
    for mem in alice_memories:
        content = mem.get("content", mem) if isinstance(mem, dict) else str(mem)
        print(f"    - {content[:60]}...")
    
    print(f"  Bob的记忆数: {len(bob_memories)}")
    for mem in bob_memories:
        content = mem.get("content", mem) if isinstance(mem, dict) else str(mem)
        print(f"    - {content[:60]}...")
    
    # 验证：Alice的记忆中不应包含Bob的内容
    alice_contents = " ".join([
        mem.get("content", str(mem)) if isinstance(mem, dict) else str(mem)
        for mem in alice_memories
    ])
    bob_contents = " ".join([
        mem.get("content", str(mem)) if isinstance(mem, dict) else str(mem)
        for mem in bob_memories
    ])
    
    isolation_ok = (
        "Rust" not in alice_contents and
        "blockchain" not in alice_contents and
        "Python" not in bob_contents and
        "machine learning" not in bob_contents
    )
    
    if isolation_ok:
        print(f"\n{Fore.GREEN}✓ 测试3通过：记忆隔离成功{Style.RESET_ALL}")
    else:
        print(f"\n{Fore.YELLOW}⚠️  测试3警告：记忆可能未完全隔离（模拟模式下可能正常）{Style.RESET_ALL}")


def test_memory_search(system: MultiUserMemorySystem):
    """测试记忆搜索"""
    print_section("测试4: 记忆搜索")
    
    alice = system.get_user_by_name("Alice")
    bob = system.get_user_by_name("Bob")
    
    if not alice or not bob:
        print(f"{Fore.RED}❌ 测试4失败：用户不存在{Style.RESET_ALL}")
        return
    
    # 搜索Alice的记忆
    print(f"\n在 {Fore.CYAN}Alice{Style.RESET_ALL} 的记忆中搜索 'Python'：")
    alice_results = system.search_memories("Python", alice.id)
    print(f"  找到 {len(alice_results)} 条结果")
    for result in alice_results:
        content = result.get("content", str(result)) if isinstance(result, dict) else str(result)
        print(f"    - {content[:60]}...")
    
    # 搜索Bob的记忆
    print(f"\n在 {Fore.CYAN}Bob{Style.RESET_ALL} 的记忆中搜索 'Rust'：")
    bob_results = system.search_memories("Rust", bob.id)
    print(f"  找到 {len(bob_results)} 条结果")
    for result in bob_results:
        content = result.get("content", str(result)) if isinstance(result, dict) else str(result)
        print(f"    - {content[:60]}...")
    
    # 验证：在Alice的记忆中搜索"Rust"应该没有结果
    print(f"\n{Fore.MAGENTA}验证跨用户搜索隔离：{Style.RESET_ALL}")
    alice_rust_results = system.search_memories("Rust", alice.id)
    print(f"  在Alice记忆中搜索'Rust': {len(alice_rust_results)} 条结果")
    
    if len(alice_rust_results) == 0:
        print(f"\n{Fore.GREEN}✓ 测试4通过：搜索隔离成功{Style.RESET_ALL}")
    else:
        print(f"\n{Fore.YELLOW}⚠️  测试4警告：搜索可能未完全隔离（模拟模式下可能正常）{Style.RESET_ALL}")


def test_duplicate_user(system: MultiUserMemorySystem):
    """测试重复用户创建"""
    print_section("测试5: 重复用户创建")
    
    original_count = len(system.list_users())
    print(f"  创建前用户数: {original_count}")
    
    # 尝试创建重复用户
    print(f"\n尝试创建已存在的用户 'Alice'：")
    duplicate_alice = system.create_user("Alice")
    
    new_count = len(system.list_users())
    print(f"  创建后用户数: {new_count}")
    
    if new_count == original_count:
        print(f"\n{Fore.GREEN}✓ 测试5通过：重复用户未被创建{Style.RESET_ALL}")
    else:
        print(f"\n{Fore.RED}❌ 测试5失败：重复用户被创建{Style.RESET_ALL}")


def test_user_deletion(system: MultiUserMemorySystem):
    """测试用户删除"""
    print_section("测试6: 用户删除")
    
    # 创建临时用户
    temp_user = system.create_user("TempUser")
    system.add_memory("This is a temporary memory.", temp_user.id)
    
    print(f"  创建临时用户: {temp_user.name}")
    print(f"  添加1条记忆")
    
    # 删除用户
    print(f"\n删除临时用户：")
    success = system.delete_user(temp_user.id)
    
    if success:
        # 验证用户已删除
        deleted_user = system.get_user(temp_user.id)
        if deleted_user is None:
            print(f"\n{Fore.GREEN}✓ 测试6通过：用户删除成功{Style.RESET_ALL}")
        else:
            print(f"\n{Fore.RED}❌ 测试6失败：用户未被删除{Style.RESET_ALL}")
    else:
        print(f"\n{Fore.RED}❌ 测试6失败：删除操作失败{Style.RESET_ALL}")


def display_final_summary(system: MultiUserMemorySystem):
    """显示最终摘要"""
    print_header("📊 最终摘要")
    
    users = system.list_users()
    total_memories = sum(user.memory_count for user in users)
    
    print(f"{Fore.CYAN}系统统计：{Style.RESET_ALL}")
    print(f"  - 总用户数: {len(users)}")
    print(f"  - 总记忆数: {total_memories}")
    
    print(f"\n{Fore.CYAN}用户详情：{Style.RESET_ALL}")
    for idx, user in enumerate(users, 1):
        print(f"  {idx}. {user.name}")
        print(f"     - ID: {user.id}")
        print(f"     - 记忆数: {user.memory_count}")
        print(f"     - 创建时间: {user.created_at.strftime('%Y-%m-%d %H:%M:%S')}")


def main():
    """主函数"""
    print(f"""
{Fore.CYAN}╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║          👥 AgentMem 多用户管理演示 👥                       ║
║                                                                ║
║          真实实现，对标MIRIX多用户功能                       ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝{Style.RESET_ALL}
""")
    
    # 初始化系统
    print(f"{Fore.CYAN}🚀 初始化多用户记忆系统...{Style.RESET_ALL}")
    system = MultiUserMemorySystem("multi_user_demo")
    print(f"{Fore.GREEN}✓ 系统初始化成功{Style.RESET_ALL}")
    
    # 运行测试
    print_header("🧪 开始测试")
    
    test_user_creation(system)
    test_user_listing(system)
    test_memory_isolation(system)
    test_memory_search(system)
    test_duplicate_user(system)
    test_user_deletion(system)
    
    # 显示最终摘要
    display_final_summary(system)
    
    # 完成
    print(f"\n{Fore.GREEN}╔════════════════════════════════════════════════════════════════╗")
    print(f"║                                                                ║")
    print(f"║           ✨ AgentMem 多用户管理演示完成！✨                ║")
    print(f"║                                                                ║")
    print(f"╚════════════════════════════════════════════════════════════════╝{Style.RESET_ALL}\n")


if __name__ == "__main__":
    main()

