#!/usr/bin/env python3
"""
AgentMem Python SDK - Webhook 服务器示例

这个示例演示了如何构建一个 Webhook 服务器：
- 接收 Webhook 事件
- 处理不同类型的事件
- 与 AgentMem 集成
- 返回响应

运行方式:
```bash
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key

python webhook_server.py
```

预期输出:
```
🔔 AgentMem Webhook 服务器示例

✅ 初始化完成

🚀 步骤 1: 启动 Webhook 服务器
   ✅ 服务器启动在 http://0.0.0.0:8000

📡 步骤 2: Webhook 端点
   POST /webhook/memory - 接收记忆事件
   POST /webhook/search - 接收搜索请求
   POST /webhook/query - 接收查询请求
   GET /health - 健康检查

🧪 步骤 3: 测试 Webhook
   ✅ 发送测试事件到 /webhook/memory
   ✅ 发送测试搜索到 /webhook/search

💡 步骤 4: 使用示例
   curl -X POST http://localhost:8000/webhook/memory \
        -H "Content-Type: application/json" \
        -d '{"content": "测试记忆", "user_id": "user_1"}'

服务器运行中... (按 Ctrl+C 停止)
```
"""

import asyncio
import os
import json
from typing import Dict, Any, Optional
from datetime import datetime

try:
    from agentmem import AgentMemClient, Config, SearchQuery, MemoryType
except ImportError:
    print("⚠️  AgentMem SDK 未安装")
    print("   安装方式: pip install agentmem")
    exit(1)

try:
    from fastapi import FastAPI, HTTPException, Request
    from fastapi.responses import JSONResponse
    import uvicorn
except ImportError:
    print("⚠️  FastAPI 未安装")
    print("   安装方式: pip install fastapi uvicorn")
    exit(1)


# ============================================
# FastAPI 应用
# ============================================

app = FastAPI(
    title="AgentMem Webhook Server",
    description="接收和处理 Webhook 事件",
    version="1.0.0",
)

# 全局 AgentMem 客户端（在启动时初始化）
agentmem_client: Optional[AgentMemClient] = None


# ============================================
# 事件处理器
# ============================================

class WebhookEventHandler:
    """Webhook 事件处理器"""

    def __init__(self, client: AgentMemClient):
        self.client = client

    async def handle_memory_event(self, event: Dict[str, Any]) -> Dict[str, Any]:
        """处理记忆事件"""
        content = event.get("content", "")
        user_id = event.get("user_id", "unknown")
        agent_id = event.get("agent_id", "webhook_agent")
        memory_type = event.get("memory_type", "episodic")
        metadata = event.get("metadata", {})

        if not content:
            raise HTTPException(status_code=400, detail="content is required")

        # 添加记忆
        memory_id = await self.client.add_memory(
            content=content,
            agent_id=agent_id,
            user_id=user_id,
            memory_type=MemoryType(memory_type),
            metadata=metadata,
        )

        return {
            "success": True,
            "memory_id": memory_id,
            "message": "Memory added successfully",
        }

    async def handle_search_event(self, event: Dict[str, Any]) -> Dict[str, Any]:
        """处理搜索事件"""
        query = event.get("query", "")
        user_id = event.get("user_id", "unknown")
        limit = event.get("limit", 10)
        threshold = event.get("threshold", 0.7)

        if not query:
            raise HTTPException(status_code=400, detail="query is required")

        # 执行搜索
        search_query = SearchQuery(
            query=query,
            user_id=user_id,
            limit=limit,
            threshold=threshold,
        )

        results = await self.client.search_memories(search_query)

        return {
            "success": True,
            "query": query,
            "count": len(results),
            "results": results,
        }

    async def handle_query_event(self, event: Dict[str, Any]) -> Dict[str, Any]:
        """处理查询事件"""
        action = event.get("action", "")
        user_id = event.get("user_id", "unknown")

        if action == "get_all":
            # 获取所有记忆
            memories = await self.client.get_all_memories(
                user_id=user_id,
                limit=event.get("limit", 100),
            )
            return {
                "success": True,
                "count": len(memories),
                "memories": memories,
            }

        elif action == "stats":
            # 获取统计信息
            memories = await self.client.get_all_memories(
                user_id=user_id,
                limit=1000,
            )

            # 简单统计
            type_counts = {}
            for memory in memories:
                memory_type = memory.get("metadata", {}).get("type", "unknown")
                type_counts[memory_type] = type_counts.get(memory_type, 0) + 1

            return {
                "success": True,
                "total": len(memories),
                "by_type": type_counts,
            }

        else:
            raise HTTPException(status_code=400, detail=f"Unknown action: {action}")


# 全局事件处理器
event_handler: Optional[WebhookEventHandler] = None


# ============================================
# API 端点
# ============================================

@app.on_event("startup")
async def startup_event():
    """启动事件"""
    global agentmem_client, event_handler

    # 初始化 AgentMem 客户端
    api_base_url = os.getenv("AGENTMEM_API_BASE_URL", "http://localhost:8080")
    api_key = os.getenv("AGENTMEM_API_KEY", "demo_key")

    config = Config(
        api_base_url=api_base_url,
        api_key=api_key,
    )

    agentmem_client = AgentMemClient(config)
    await agentmem_client.__aenter__()

    # 初始化事件处理器
    event_handler = WebhookEventHandler(agentmem_client)

    print("\n✅ AgentMem 客户端已连接")


@app.on_event("shutdown")
async def shutdown_event():
    """关闭事件"""
    if agentmem_client:
        await agentmem_client.__aexit__(None, None, None)


@app.get("/")
async def root():
    """根路径"""
    return {
        "service": "AgentMem Webhook Server",
        "version": "1.0.0",
        "status": "running",
    }


@app.get("/health")
async def health():
    """健康检查"""
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat(),
    }


@app.post("/webhook/memory")
async def webhook_memory(request: Request):
    """接收记忆事件"""
    try:
        event = await request.json()
        result = await event_handler.handle_memory_event(event)
        return JSONResponse(content=result, status_code=200)
    except Exception as e:
        return JSONResponse(
            content={"success": False, "error": str(e)},
            status_code=500,
        )


@app.post("/webhook/search")
async def webhook_search(request: Request):
    """接收搜索请求"""
    try:
        event = await request.json()
        result = await event_handler.handle_search_event(event)
        return JSONResponse(content=result, status_code=200)
    except Exception as e:
        return JSONResponse(
            content={"success": False, "error": str(e)},
            status_code=500,
        )


@app.post("/webhook/query")
async def webhook_query(request: Request):
    """接收查询请求"""
    try:
        event = await request.json()
        result = await event_handler.handle_query_event(event)
        return JSONResponse(content=result, status_code=200)
    except Exception as e:
        return JSONResponse(
            content={"success": False, "error": str(e)},
            status_code=500,
        )


# ============================================
# 主函数
# ============================================

async def send_test_events():
    """发送测试事件（演示用）"""
    import httpx

    print("\n🧪 步骤 3: 测试 Webhook 端点")
    print("---")

    base_url = "http://localhost:8000"

    async with httpx.AsyncClient() as client:
        # 测试 1: 添加记忆
        print("\n   测试 1: 添加记忆")
        response = await client.post(
            f"{base_url}/webhook/memory",
            json={
                "content": "这是一条测试记忆",
                "user_id": "test_user",
                "agent_id": "test_agent",
            }
        )
        print(f"   ✅ 状态: {response.status_code}")
        print(f"   响应: {response.json()}")

        # 等待一下
        await asyncio.sleep(1)

        # 测试 2: 搜索
        print("\n   测试 2: 搜索记忆")
        response = await client.post(
            f"{base_url}/webhook/search",
            json={
                "query": "测试",
                "user_id": "test_user",
                "limit": 5,
            }
        )
        print(f"   ✅ 状态: {response.status_code}")
        result = response.json()
        print(f"   找到 {result.get('count', 0)} 条结果")

        # 测试 3: 查询统计
        print("\n   测试 3: 查询统计")
        response = await client.post(
            f"{base_url}/webhook/query",
            json={
                "action": "stats",
                "user_id": "test_user",
            }
        )
        print(f"   ✅ 状态: {response.status_code}")
        result = response.json()
        print(f"   总记忆数: {result.get('total', 0)}")


def main():
    """主函数"""
    print("🔔 AgentMem Webhook 服务器示例\n")
    print("这个示例演示了:")
    print("  1. 启动 Webhook 服务器")
    print("  2. 接收记忆事件")
    print("  3. 处理搜索请求")
    print("  4. 响应查询请求")
    print()

    # 启动说明
    print("🚀 启动服务器...")
    print("   端点:")
    print("     - POST /webhook/memory - 接收记忆事件")
    print("     - POST /webhook/search - 接收搜索请求")
    print("     - POST /webhook/query - 接收查询请求")
    print("     - GET  /health - 健康检查")
    print()

    # 使用示例
    print("💡 使用示例:")
    print()
    print("   # 添加记忆")
    print("   curl -X POST http://localhost:8000/webhook/memory \\")
    print("        -H 'Content-Type: application/json' \\")
    print("        -d '{\"content\": \"测试记忆\", \"user_id\": \"user_1\"}'")
    print()
    print("   # 搜索记忆")
    print("   curl -X POST http://localhost:8000/webhook/search \\")
    print("        -H 'Content-Type: application/json' \\")
    print("        -d '{\"query\": \"测试\", \"user_id\": \"user_1\"}'")
    print()
    print("   # 查询统计")
    print("   curl -X POST http://localhost:8000/webhook/query \\")
    print("        -H 'Content-Type: application/json' \\")
    print("        -d '{\"action\": \"stats\", \"user_id\": \"user_1\"}'")
    print()

    # 运行测试
    print("⚠️  注意: 服务器启动后，将在 5 秒后自动运行测试...")
    print("   或者你可以使用上面的 curl 命令手动测试")
    print()

    # 启动服务器
    try:
        # 在后台运行测试
        async def run_server_with_test():
            # 等待服务器启动
            await asyncio.sleep(5)

            # 运行测试
            await send_test_events()

            print("\n✅ 测试完成！服务器继续运行...")
            print("   按 Ctrl+C 停止服务器\n")

        # 创建异步任务
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)

        # 启动测试任务
        loop.create_task(run_server_with_test())

        # 启动服务器
        uvicorn.run(
            app,
            host="0.0.0.0",
            port=8000,
            log_level="info",
        )

    except KeyboardInterrupt:
        print("\n\n👋 服务器已停止")


if __name__ == "__main__":
    main()


# ============================================
# 高级功能: 事件验证
# ============================================
#
# 添加 Webhook 签名验证:
#
# ```python
# import hmac
# import hashlib
#
# def verify_webhook_signature(payload: bytes, signature: str, secret: str) -> bool:
#     """验证 Webhook 签名"""
#     expected_signature = hmac.new(
#         secret.encode(),
#         payload,
#         hashlib.sha256
#     ).hexdigest()
#
#     return hmac.compare_digest(expected_signature, signature)
#
# @app.post("/webhook/memory")
# async def webhook_memory(request: Request):
#     # 获取签名
#     signature = request.headers.get("X-Webhook-Signature", "")
#
#     # 读取 payload
#     payload = await request.body()
#
#     # 验证签名
#     secret = os.getenv("WEBHOOK_SECRET", "")
#     if not verify_webhook_signature(payload, signature, secret):
#         raise HTTPException(status_code=401, detail="Invalid signature")
#
#     # 处理事件
#     event = json.loads(payload)
#     ...
# ```
#
# ============================================
# 高级功能: 异步处理
# ============================================
#
# ```python
# from fastapi import BackgroundTasks
#
# async def process_event_async(event: dict):
#     """异步处理事件"""
#     # 耗时操作
#     await asyncio.sleep(1)
#
#     # 处理逻辑
#     ...
#
# @app.post("/webhook/memory")
# async def webhook_memory(
#     request: Request,
#     background_tasks: BackgroundTasks
# ):
#     event = await request.json()
#
#     # 立即返回
#     background_tasks.add_task(process_event_async, event)
#
#     return {"success": True, "message": "Event queued"}
# ```
#
# ============================================
# 高级功能: 事件重试
# ============================================
#
# ```python
# import tenacity
#
# @tenacity.retry(
#     stop=tenacity.stop_after_attempt(3),
#     wait=tenacity.wait_exponential(multiplier=1, min=4, max=10),
# )
# async def process_with_retry(event: dict):
#     """带重试的事件处理"""
#     # 可能失败的操作
#     ...
#
# @app.post("/webhook/memory")
# async def webhook_memory(request: Request):
#     event = await request.json()
#
#     try:
#         result = await process_with_retry(event)
#         return {"success": True, "result": result}
#     except Exception as e:
#         return {"success": False, "error": str(e)}
# ```
