# LumosAI Streaming 快速使用指南

## 🚀 快速开始

### 1. 启动服务器

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
./start_server_no_auth.sh
```

### 2. 测试SSE Streaming

```bash
# 自动化测试脚本
./test_lumosai_real_streaming.sh

# 或手动测试
AGENT_ID=$(curl -s "http://localhost:8080/api/v1/agents" \
  -H "Authorization: Bearer test-token" | jq -r '.data[0].id')

curl -N -X POST \
  "http://localhost:8080/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  -d '{"message":"你好，请介绍一下自己","user_id":"test"}'
```

---

## 📡 API端点

### Streaming端点
```
POST /api/v1/agents/:agent_id/chat/lumosai/stream
```

### 请求格式
```json
{
  "message": "你的问题",
  "user_id": "test-user",
  "session_id": "optional-session-id",
  "metadata": {}
}
```

### 响应格式 (SSE)

#### 开始事件
```
data: {"chunk_type":"start","message":"Agent started"}
```

#### 文本增量事件
```
data: {"chunk_type":"content","content":"文本片段"}
```

#### 工具调用事件
```
data: {"chunk_type":"tool_call","tool_name":"tool_name","arguments":{}}
```

#### 完成事件
```
data: {
  "chunk_type":"done",
  "final_response":"完整响应文本",
  "total_steps":2,
  "memories_updated":true,
  "memories_count":3
}
```

#### 错误事件
```
data: {"chunk_type":"error","content":"错误信息"}
```

---

## 💻 前端集成

### JavaScript/TypeScript

```typescript
async function streamChat(agentId: string, message: string) {
  const response = await fetch(
    `http://localhost:8080/api/v1/agents/${agentId}/chat/lumosai/stream`,
    {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer test-token',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        message,
        user_id: 'test-user',
      }),
    }
  );

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n');

    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = JSON.parse(line.slice(6));
        
        switch (data.chunk_type) {
          case 'start':
            console.log('🚀 Agent started');
            break;
          case 'content':
            // 实时显示文本
            console.log(data.content);
            break;
          case 'done':
            console.log('✅ Complete:', data.final_response);
            break;
          case 'error':
            console.error('❌ Error:', data.content);
            break;
        }
      }
    }
  }
}

// 使用
streamChat('agent-id', '你好').catch(console.error);
```

### React Hook

```typescript
import { useState, useCallback } from 'react';

interface StreamMessage {
  type: 'start' | 'content' | 'done' | 'error';
  content?: string;
  finalResponse?: string;
}

function useStreamingChat(agentId: string) {
  const [messages, setMessages] = useState<StreamMessage[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);

  const sendMessage = useCallback(async (message: string) => {
    setIsStreaming(true);
    setMessages([]);

    try {
      const response = await fetch(
        `http://localhost:8080/api/v1/agents/${agentId}/chat/lumosai/stream`,
        {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer test-token',
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ message, user_id: 'test-user' }),
        }
      );

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = JSON.parse(line.slice(6));
            
            setMessages(prev => [...prev, {
              type: data.chunk_type,
              content: data.content,
              finalResponse: data.final_response,
            }]);
          }
        }
      }
    } catch (error) {
      console.error('Streaming error:', error);
    } finally {
      setIsStreaming(false);
    }
  }, [agentId]);

  return { messages, isStreaming, sendMessage };
}

// 使用
function ChatComponent() {
  const { messages, isStreaming, sendMessage } = useStreamingChat('agent-id');

  return (
    <div>
      {messages.map((msg, i) => (
        <div key={i}>
          {msg.type === 'content' && msg.content}
          {msg.type === 'done' && <div>✅ {msg.finalResponse}</div>}
        </div>
      ))}
      <button 
        onClick={() => sendMessage('你好')} 
        disabled={isStreaming}
      >
        发送
      </button>
    </div>
  );
}
```

---

## 🔧 配置选项

### 环境变量

```bash
# 修改分块大小 (默认10字符)
export SSE_CHUNK_SIZE=20

# 启用streaming调试日志
export RUST_LOG=agent_mem_server::routes::chat_lumosai=debug
```

### 编译选项

```bash
# 编译时启用lumosai feature
cargo build --release --bin agent-mem-server --features lumosai
```

---

## 🐛 故障排查

### 问题1: 连接被拒绝

```bash
# 检查服务器状态
curl http://localhost:8080/health

# 如果失败，启动服务器
./start_server_no_auth.sh
```

### 问题2: 没有收到streaming事件

**检查点**:
1. ✅ Content-Type是否为 `application/json`
2. ✅ Authorization header是否正确
3. ✅ Agent ID是否存在

**调试**:
```bash
# 查看服务器日志
tail -f backend-no-auth.log | grep lumosai
```

### 问题3: 响应太慢

**原因**: 当前实现需要等待完整LLM响应

**解决**:
- 使用更快的模型 (`glm-4-flash`)
- 减少历史消息数量
- 限制response长度

---

## 📊 性能优化

### 1. 选择快速模型

```json
{
  "llm_config": {
    "provider": "zhipu",
    "model": "glm-4-flash",  // 更快
    "temperature": 0.7
  }
}
```

### 2. 减少历史检索

当前设置为3条历史消息（已优化）

### 3. 控制响应长度

```json
{
  "message": "请简短回答：...",
  "metadata": {
    "max_tokens": 200
  }
}
```

---

## 📚 相关文档

- 📖 **完整分析**: `LumosAI_Agent_Streaming全面分析.md`
- 📊 **测试报告**: `LumosAI_SSE_Streaming测试报告.md`
- 🧪 **测试脚本**: `test_lumosai_real_streaming.sh`
- 💻 **示例代码**: `lumosai_core/examples/real_streaming_test.rs`

---

## ✨ 示例场景

### 场景1: 简单问答

```bash
curl -N -X POST "http://localhost:8080/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  -d '{"message":"1+1等于几？","user_id":"test"}'
```

### 场景2: 编程问题

```bash
curl -N -X POST "http://localhost:8080/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  -d '{"message":"用Python写一个快速排序","user_id":"test"}'
```

### 场景3: 长文本生成

```bash
curl -N -X POST "http://localhost:8080/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  -d '{"message":"写一篇关于AI的文章","user_id":"test"}'
```

---

## 🎯 最佳实践

### 1. 错误处理

```typescript
try {
  await streamChat(agentId, message);
} catch (error) {
  if (error.name === 'AbortError') {
    console.log('Stream cancelled by user');
  } else {
    console.error('Stream error:', error);
  }
}
```

### 2. 超时控制

```typescript
const controller = new AbortController();
setTimeout(() => controller.abort(), 30000); // 30秒超时

fetch(url, { signal: controller.signal });
```

### 3. 重连机制

```typescript
async function streamWithRetry(agentId, message, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      await streamChat(agentId, message);
      break;
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
    }
  }
}
```

---

**更新时间**: 2025-11-20  
**状态**: ✅ 生产就绪
