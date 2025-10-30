#!/usr/bin/env python3
"""
简单的 MCP 工具测试脚本
测试 P1-3 修复：验证 MCP 工具调用真实 Backend API
"""

import subprocess
import json
import sys
import requests
import time

# 颜色定义
GREEN = '\033[0;32m'
RED = '\033[0;31m'
YELLOW = '\033[1;33m'
NC = '\033[0m'  # No Color

def call_mcp_tool(tool_name, arguments):
    """调用 MCP 工具"""
    mcp_server = "./target/release/agentmem-mcp-server"
    
    # 构建请求
    requests_data = [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        },
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        }
    ]
    
    # 转换为 JSONL 格式（每行一个 JSON）
    input_data = "\n".join(json.dumps(req) for req in requests_data) + "\n"
    
    # 调用 MCP server
    try:
        process = subprocess.Popen(
            [mcp_server],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        stdout, stderr = process.communicate(input=input_data, timeout=10)
        
        # 解析响应（最后一行是 tools/call 的响应）
        lines = [line for line in stdout.strip().split('\n') if line and line.startswith('{')]
        if len(lines) >= 2:
            return json.loads(lines[-1])
        else:
            print(f"{RED}❌ 未收到有效响应{NC}")
            print(f"stdout: {stdout}")
            print(f"stderr: {stderr}")
            return None
            
    except subprocess.TimeoutExpired:
        process.kill()
        print(f"{RED}❌ MCP server 超时{NC}")
        return None
    except Exception as e:
        print(f"{RED}❌ 调用失败: {e}{NC}")
        return None

def get_memory_count():
    """获取当前记忆数量"""
    try:
        response = requests.get("http://localhost:8080/api/v1/stats", timeout=5)
        if response.status_code == 200:
            data = response.json()
            return data.get("data", {}).get("total_memories", 0)
        return 0
    except:
        return 0

def main():
    print("=" * 60)
    print("测试 MCP 工具真实 API 调用")
    print("=" * 60)
    print()
    
    # 检查 Backend
    print("1️⃣  检查 Backend 服务...")
    try:
        response = requests.get("http://localhost:8080/health", timeout=5)
        if response.status_code == 200:
            print(f"{GREEN}✅ Backend 运行正常{NC}")
        else:
            print(f"{RED}❌ Backend 返回错误: {response.status_code}{NC}")
            sys.exit(1)
    except Exception as e:
        print(f"{RED}❌ Backend 未运行: {e}{NC}")
        sys.exit(1)
    
    print()
    
    # 获取基准记忆数量
    print("2️⃣  获取当前记忆数量（基准）...")
    baseline_count = get_memory_count()
    print(f"   当前记忆数量: {baseline_count}")
    print()
    
    # 测试 1: agentmem_add_memory
    print("=" * 60)
    print("测试 1: agentmem_add_memory (添加记忆)")
    print("=" * 60)
    
    add_params = {
        "content": "MCP 工具测试：这是一条通过 MCP 添加的测试记忆 " + str(int(time.time())),
        "user_id": "mcp-test-user",
        "agent_id": "default-agent",
        "memory_type": "Episodic"
    }
    
    response = call_mcp_tool("agentmem_add_memory", add_params)
    if response and "result" in response:
        print(f"{GREEN}✅ 工具调用成功{NC}")
        print("响应:")
        print(json.dumps(response["result"], indent=2, ensure_ascii=False))
        
        # 验证数据库
        print()
        print("3️⃣  验证记忆是否真实添加到数据库...")
        time.sleep(2)
        
        new_count = get_memory_count()
        print(f"   新记忆数量: {new_count}")
        
        if new_count > baseline_count:
            print(f"{GREEN}✅ 记忆已成功添加到数据库 (增加了 {new_count - baseline_count} 条){NC}")
            baseline_count = new_count
        else:
            print(f"{RED}❌ 记忆未添加到数据库（仍然是模拟数据）{NC}")
    else:
        print(f"{RED}❌ 工具调用失败{NC}")
        if response:
            print(json.dumps(response, indent=2))
    
    print()
    
    # 测试 2: agentmem_search_memories
    print("=" * 60)
    print("测试 2: agentmem_search_memories (搜索记忆)")
    print("=" * 60)
    
    search_params = {
        "query": "MCP 工具测试",
        "limit": 5
    }
    
    response = call_mcp_tool("agentmem_search_memories", search_params)
    if response and "result" in response:
        print(f"{GREEN}✅ 工具调用成功{NC}")
        print("响应:")
        result = response["result"]
        print(f"  查询: {result.get('query')}")
        print(f"  结果数量: {result.get('total_results', 0)}")
        
        if result.get('total_results', 0) > 0:
            print(f"{GREEN}✅ 搜索到真实记忆{NC}")
        else:
            print(f"{YELLOW}⚠️  未搜索到记忆（可能是向量搜索未返回结果）{NC}")
    else:
        print(f"{RED}❌ 工具调用失败{NC}")
        if response:
            print(json.dumps(response, indent=2))
    
    print()
    
    # 最终总结
    print("=" * 60)
    print("测试完成总结")
    print("=" * 60)
    print()
    
    final_count = get_memory_count()
    print(f"最终记忆数量: {final_count}")
    print(f"基准记忆数量: {baseline_count}")
    print()
    
    if final_count > baseline_count:
        print(f"{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}")
        print(f"{GREEN}✅ P1-3 修复成功！{NC}")
        print(f"{GREEN}   MCP 工具已成功集成真实 Backend API{NC}")
        print(f"{GREEN}   数据已持久化到数据库{NC}")
        print(f"{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}")
    else:
        print(f"{RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}")
        print(f"{RED}❌ P1-3 修复可能未完全成功{NC}")
        print(f"{RED}   请检查 MCP 工具实现{NC}")
        print(f"{RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}")

if __name__ == "__main__":
    main()

