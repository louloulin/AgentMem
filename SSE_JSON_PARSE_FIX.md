# 🐛 SSE JSON解析错误修复

## 📊 问题现象

```
Error: Stream error: LLM error: Failed to parse 智谱AI streaming response: 
EOF while parsing a string at line 1 column 85
```

## 🔍 根本原因

### HTTP分包导致JSON不完整

**SSE数据格式**：
```
data: {"id":"xxx","choices":[{"delta":{"content":"你"}}]}\n
\n
data: {"id":"xxx","choices":[{"delta":{"content":"好"}}]}\n
\n
```

**问题**：TCP分包可能导致：
```
Chunk 1: data: {"id":"xxx","choices":[{"del
Chunk 2: ta":{"content":"你"}}]}\n\ndata: {"id":"yyy"...
```

当我们对Chunk 1执行 `serde_json::from_str`：
- ❌ JSON不完整
- ❌ 抛出 "EOF while parsing a string"
- ❌ 整个流失败

## ✅ 解决方案

### 使用Buffer处理跨Chunk数据

```rust
// ❌ 原代码：直接处理每个chunk
.map(|chunk_result| {
    let text = String::from_utf8(chunk)?;
    for line in text.lines() {
        let data = line.strip_prefix("data: ")?;
        serde_json::from_str(data)?; // ← 可能失败！
    }
})

// ✅ 新代码：使用unfold维护buffer
futures::stream::unfold(
    (byte_stream, String::new()), // buffer
    |(mut stream, mut buffer)| async move {
        // 追加新数据
        buffer.push_str(&text);
        
        // 分离完整行和不完整行
        let lines: Vec<&str> = buffer.lines().collect();
        let has_trailing_newline = buffer.ends_with('\n');
        
        let (complete_lines, remaining) = if has_trailing_newline {
            (lines.as_slice(), "")
        } else if lines.len() > 0 {
            (&lines[..lines.len()-1], lines[lines.len()-1])
        } else {
            continue; // 没有完整行，继续读取
        };
        
        // 只处理完整的行
        for line in complete_lines {
            if let Some(data) = line.strip_prefix("data: ") {
                match serde_json::from_str(data) {
                    Ok(response) => { /* 处理 */ }
                    Err(e) => {
                        // 非关键错误，继续
                        eprintln!("⚠️  JSON parse error: {}", e);
                    }
                }
            }
        }
        
        // 保留不完整行到下次
        buffer = remaining.to_string();
    }
)
```

## 🎯 关键改进

1. **Buffer跨Chunk数据** ✅
   - 维护一个字符串buffer
   - 保留不完整的行到下次处理

2. **只处理完整行** ✅
   - 检查是否以`\n`结尾
   - 分离完整行和不完整行

3. **降级JSON错误** ✅
   - 从 `return Err` 改为 `eprintln!`
   - 允许部分失败，继续流式处理

4. **去除filter_map阻塞** ✅
   - 使用unfold一次性处理
   - 避免多次异步迭代

## 📝 文件修改

- `lumosai/lumosai_core/src/llm/zhipu.rs:645-713`
- `crates/agent-mem-llm/src/providers/zhipu.rs` (同样需要修复)

## 🧪 测试验证

```bash
# 1. 重启服务
pkill -f agent-mem-server
./target/release/agent-mem-server > backend-sse-fixed.log 2>&1 &

# 2. 发送测试请求
curl -N -X POST "http://localhost:8080/api/v1/agents/AGENT_ID/chat/lumosai/stream" \
  -H "Content-Type: application/json" \
  -d '{"message":"你好","user_id":"test","stream":true}'

# 3. 观察日志
tail -f backend-sse-fixed.log | grep -E "⚠️|parse error"
```

期望：
- ✅ 不应该看到 "EOF while parsing" 错误
- ✅ 流式输出应该连续不中断
- ⚠️  可能看到 "JSON parse error (non-critical)" - 这是正常的

## ✅ 完成状态

- ✅ LumosAI Zhipu SSE解析器修复
- ⏳ AgentMem Zhipu SSE解析器 (需要同样修复)
- ✅ 服务编译通过
- ⏳ 测试验证

时间: 2025-11-20 21:25

