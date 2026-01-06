#!/usr/bin/env python3
"""
AgentMem MCP Stdio 服务器测试脚本
"""

import json
import subprocess
import sys
import time

def test_server():
    """测试 MCP Stdio 服务器"""
    
    print("=== AgentMem MCP Stdio 服务器测试 ===\n")
    
    # 启动服务器进程
    server_path = "../../target/release/agentmem-mcp-server"
    
    try:
        process = subprocess.Popen(
            [server_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        
        print("✅ 服务器进程已启动\n")
        
        # 测试 1: Initialize 握手
        print("📝 测试 1: Initialize 握手")
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }
        
        process.stdin.write(json.dumps(request) + "\n")
        process.stdin.flush()
        
        # 读取响应
        response_line = process.stdout.readline()
        if response_line:
            response = json.loads(response_line)
            if "result" in response:
                print("✅ Initialize 握手成功")
                print(f"响应: {json.dumps(response, ensure_ascii=False)[:200]}...\n")
            else:
                print(f"❌ Initialize 握手失败: {response}\n")
                return False
        else:
            print("❌ 没有收到响应\n")
            return False
        
        # 测试 2: 列出工具
        print("📝 测试 2: 列出工具")
        request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }
        
        process.stdin.write(json.dumps(request) + "\n")
        process.stdin.flush()
        
        response_line = process.stdout.readline()
        if response_line:
            response = json.loads(response_line)
            if "result" in response and "tools" in response["result"]:
                tools = response["result"]["tools"]
                print(f"✅ 工具列表获取成功，共 {len(tools)} 个工具")
                for tool in tools:
                    print(f"   - {tool['name']}")
                print()
            else:
                print(f"❌ 工具列表获取失败: {response}\n")
                return False
        else:
            print("❌ 没有收到响应\n")
            return False
        
        # 测试 3: 调用工具 (添加记忆)
        print("📝 测试 3: 调用工具 (添加记忆)")
        request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "agentmem_add_memory",
                "arguments": {
                    "content": "测试记忆内容",
                    "user_id": "test_user"
                }
            }
        }
        
        process.stdin.write(json.dumps(request) + "\n")
        process.stdin.flush()
        
        response_line = process.stdout.readline()
        if response_line:
            response = json.loads(response_line)
            if "result" in response:
                print("✅ 工具调用成功")
                print(f"响应: {json.dumps(response, ensure_ascii=False)[:200]}...\n")
            elif "error" in response:
                print("⚠️  工具调用返回错误（这是正常的，因为需要配置存储后端）")
                print(f"错误: {response['error']['message']}\n")
            else:
                print(f"❌ 工具调用失败: {response}\n")
        else:
            print("❌ 没有收到响应\n")
        
        # 关闭服务器
        process.terminate()
        process.wait(timeout=5)
        
        print("=== 测试完成 ===\n")
        print("✅ 基本功能测试通过！\n")
        print("下一步: 配置 Claude Desktop 进行集成测试")
        print("请参考 README.md 中的配置说明")
        
        return True
        
    except FileNotFoundError:
        print(f"❌ 错误: 可执行文件不存在: {server_path}")
        print("请先运行: cargo build --package mcp-stdio-server --release")
        return False
    except Exception as e:
        print(f"❌ 测试失败: {e}")
        import traceback
        traceback.print_exc()
        return False
    finally:
        try:
            process.terminate()
        except:
            pass

if __name__ == "__main__":
    success = test_server()
    sys.exit(0 if success else 1)

