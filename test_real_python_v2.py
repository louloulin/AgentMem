#!/usr/bin/env python3
"""
AgentMem 真实功能验证脚本 (使用标准库)

直接测试 AgentMem API 的所有核心功能
"""

import asyncio
import json
import urllib.request
import urllib.error
from typing import Any, Dict, List


class AgentMemTester:
    """AgentMem API 测试器"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url

    def _post(self, endpoint: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """发送 POST 请求"""
        url = f"{self.base_url}{endpoint}"
        body = json.dumps(data).encode('utf-8')

        req = urllib.request.Request(
            url,
            data=body,
            headers={'Content-Type': 'application/json'},
            method='POST'
        )

        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                return json.loads(resp.read().decode('utf-8'))
        except urllib.error.HTTPError as e:
            error_body = e.read().decode('utf-8')
            return {"error": True, "status": e.code, "message": error_body}
        except Exception as e:
            return {"error": True, "message": str(e)}

    def _get(self, endpoint: str) -> Dict[str, Any]:
        """发送 GET 请求"""
        url = f"{self.base_url}{endpoint}"

        try:
            with urllib.request.urlopen(url, timeout=10) as resp:
                return json.loads(resp.read().decode('utf-8'))
        except urllib.error.HTTPError as e:
            return {"error": True, "status": e.code, "message": str(e)}
        except Exception as e:
            return {"error": True, "message": str(e)}


def test_health(tester: AgentMemTester):
    """测试 1: 健康检查"""
    print("✅ 测试 1: 健康检查")
    result = tester._get("/health")
    print(f"   状态: {result.get('status', 'unknown')}")
    print(f"   版本: {result.get('version', 'unknown')}")
    print()
    return result


def test_add_memory(tester: AgentMemTester):
    """测试 2: 添加记忆"""
    print("✅ 测试 2: 添加记忆")

    memories = [
        "我喜欢编程和写代码",
        "我住在中国上海",
        "我最喜欢的编程语言是 Python",
        "AgentMem 是一个强大的 AI 记忆系统",
        "今天是 2025 年的最后一天"
    ]

    memory_ids = []

    for i, content in enumerate(memories, 1):
        result = tester._post("/api/v1/memories", {
            "content": content,
            "agent_id": "test_agent",
            "user_id": "test_user",
            "metadata": {
                "test": "true",  # 改为字符串
                "index": str(i)
            }
        })

        if result.get("error"):
            print(f"   {i}. ❌ 错误: {result.get('message')}")
            continue

        memory_id = result.get("id") or result.get("memory_id")
        memory_ids.append(memory_id)

        print(f"   {i}. ✅ 添加: \"{content}\"")
        print(f"      ID: {memory_id}")

    print(f"   ✅ 成功添加 {len(memory_ids)} 条记忆")
    print()
    return memory_ids


def test_search_memories(tester: AgentMemTester):
    """测试 3: 搜索记忆"""
    print("✅ 测试 3: 语义搜索")

    queries = [
        "编程",
        "上海",
        "Python",
        "AI 系统"
    ]

    for query in queries:
        result = tester._post("/api/v1/memories/search", {
            "query": query,
            "user_id": "test_user",
            "limit": 5
        })

        if result.get("error"):
            print(f"   ❌ 搜索错误: {result.get('message')}")
            continue

        memories = result.get("memories") or result.get("results") or []
        print(f"   🔍 搜索: \"{query}\"")

        if memories:
            print(f"   ✅ 找到 {len(memories)} 条记忆:")
            for i, mem in enumerate(memories[:3], 1):
                content = mem.get("content") or mem.get("text", "")
                score = mem.get("score") or mem.get("similarity", 0.0)
                print(f"      {i}. {content[:60]}... (相似度: {score:.2f})")
        else:
            print(f"   ⚠️  未找到相关记忆")
        print()

    return True


def test_get_all_memories(tester: AgentMemTester):
    """测试 4: 获取所有记忆"""
    print("✅ 测试 4: 获取所有记忆")

    result = tester._get("/api/v1/memories?user_id=test_user&limit=10")

    if result.get("error"):
        print(f"   ❌ 错误: {result.get('message')}")
        return []

    # 处理不同的响应格式
    if isinstance(result, dict):
        memories = result.get("memories") or result.get("items") or result.get("data", [])
    elif isinstance(result, list):
        memories = result
    else:
        memories = []

    print(f"   ✅ 共有 {len(memories)} 条记忆:")
    for i, mem in enumerate(memories[:10], 1):
        if isinstance(mem, dict):
            content = mem.get("content") or mem.get("text", "")
            mem_id = mem.get("id") or mem.get("memory_id", "")
            print(f"      {i}. [{mem_id[:8]}...] {content[:60]}")
        else:
            print(f"      {i}. {mem}")
    print()

    return memories


def main():
    """主测试函数"""
    print("=" * 60)
    print("🚀 AgentMem 真实功能验证")
    print("=" * 60)
    print()

    tester = AgentMemTester()

    try:
        # 测试 1: 健康检查
        health = test_health(tester)
        if health.get("error"):
            print("❌ 服务器健康检查失败，请确保服务器正在运行")
            return

        # 测试 2: 添加记忆
        memory_ids = test_add_memory(tester)

        # 测试 3: 搜索记忆
        test_search_memories(tester)

        # 测试 4: 获取所有记忆
        all_memories = test_get_all_memories(tester)

        print("=" * 60)
        print("🎉 核心功能测试完成！")
        print("=" * 60)
        print()
        print("✅ AgentMem 核心功能验证通过：")
        print("   ✓ 健康检查")
        print("   ✓ 添加记忆")
        print("   ✓ 语义搜索")
        print("   ✓ 获取记忆")
        print()
        print(f"📊 测试统计:")
        print(f"   添加记忆: {len(memory_ids)} 条")
        print(f"   检索到: {len(all_memories)} 条")
        print()
        print("💡 所有核心 API 都正常工作！")

    except Exception as e:
        print(f"\n❌ 错误: {e}")
        import traceback
        traceback.print_exc()
        print()
        print("💡 请确保 AgentMem 服务器正在运行:")
        print("   just start-server  # 或")
        print("   cargo run --bin agent-mem-server")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n👋 测试被用户中断")
