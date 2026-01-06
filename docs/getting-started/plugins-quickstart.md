# 🚀 AgentMem 插件系统 - 快速开始

## ⚡ 5分钟快速体验

### 1. 启动服务

```bash
cd agentmen
just start-full-with-plugins
```

**等待服务就绪** (~10秒)：
- ✅ 后端: http://localhost:8080
- ✅ 前端: http://localhost:3001

---

### 2. 访问插件管理页面

```bash
open http://localhost:3001/admin/plugins
```

或浏览器访问: **http://localhost:3001/admin/plugins**

---

### 3. 查看已安装插件

页面将显示：
- 📊 **统计卡片**: 总数 4 个，活跃 4 个
- 📋 **插件列表**: 
  - Hello Plugin
  - Memory Processor
  - Code Analyzer
  - LLM Plugin

---

### 4. 注册新插件（可选）

#### 方式 1: 通过 UI

1. 点击 **"Add Plugin"** 按钮
2. 填写表单:
   - Name: "My Test Plugin"
   - Version: "1.0.0"
   - Description: "A test plugin"
   - Plugin Type: "Memory Processor"
   - WASM File: 选择 `.wasm` 文件
3. 点击 **"Register Plugin"**

#### 方式 2: 通过 API

```bash
curl -X POST http://localhost:8080/api/v1/plugins \
  -H "Content-Type: application/json" \
  -H "X-User-ID: user_001" \
  -H "X-Organization-ID: org_001" \
  -d '{
    "name": "My Test Plugin",
    "description": "A test plugin for validation",
    "version": "1.0.0",
    "plugin_type": "memory_processor",
    "wasm_path": "target/wasm32-wasip1/release/memory_processor_plugin.wasm",
    "config": {}
  }' | jq
```

---

### 5. 验证功能

#### 刷新插件列表
- 点击 **"Refresh"** 按钮
- 列表自动更新

#### 查看插件详情
- 点击任一插件的 **"View Details"** 按钮
- 查看完整信息

#### API 测试
```bash
# 获取所有插件
curl http://localhost:8080/api/v1/plugins | jq

# 获取特定插件
curl http://localhost:8080/api/v1/plugins/Hello%20Plugin | jq
```

---

## 📚 常用命令

### 服务管理

```bash
# 启动全栈（后台+前端）
just start-full-with-plugins

# 仅启动后端（前台）
just start-server-with-plugins

# 仅启动前端
cd agentmem-ui && npm run dev

# 停止服务
pkill agent-mem-server
pkill -f "next dev"
```

### 插件开发

```bash
# 编译所有 WASM 插件
bash build_plugins.sh

# 编译单个插件
cd examples/hello_plugin
cargo build --target wasm32-wasip1 --release

# 查看编译结果
ls -lh target/wasm32-wasip1/release/*.wasm
```

### 测试

```bash
# 运行后端测试
cargo test --features plugins

# 运行 UI 测试
bash scripts/test_plugin_ui.sh

# 运行 E2E WASM 测试
cd crates/agent-mem-plugins
cargo test e2e_wasm_plugin_test
```

---

## 🔗 快速访问链接

| 服务 | URL |
|------|-----|
| 🖥️ **插件管理** | http://localhost:3001/admin/plugins |
| 📊 **Admin 后台** | http://localhost:3001/admin |
| 📡 **插件 API** | http://localhost:8080/api/v1/plugins |
| 📚 **API 文档** | http://localhost:8080/swagger-ui/ |
| ❤️ **健康检查** | http://localhost:8080/health |

---

## 🧪 测试清单

使用自动化测试脚本：

```bash
bash scripts/test_plugin_ui.sh
```

**手动测试**：

- [ ] 访问插件管理页面
- [ ] 查看统计卡片（显示正确数量）
- [ ] 查看插件列表（显示所有插件）
- [ ] 点击 "Add Plugin" 按钮
- [ ] 填写表单并验证
- [ ] 上传 .wasm 文件
- [ ] 提交注册
- [ ] 查看成功通知
- [ ] 列表自动刷新
- [ ] 点击 "Refresh" 按钮
- [ ] 查看插件详情

---

## 🐛 故障排查

### 问题 1: 后端未运行

**症状**: 前端显示 "Failed to load plugins"

**解决**:
```bash
# 检查后端状态
curl http://localhost:8080/health

# 启动后端
cd agentmen
just start-server-with-plugins
```

---

### 问题 2: 前端未运行

**症状**: 浏览器无法访问 http://localhost:3001

**解决**:
```bash
# 检查前端进程
ps aux | grep "next dev"

# 启动前端
cd agentmen/agentmem-ui
npm run dev
```

---

### 问题 3: 插件列表为空

**症状**: 页面显示 "No plugins installed"

**解决**:
```bash
# 检查 API
curl http://localhost:8080/api/v1/plugins

# 注册示例插件
bash build_plugins.sh

# 通过 API 注册
curl -X POST http://localhost:8080/api/v1/plugins \
  -H "Content-Type: application/json" \
  -H "X-User-ID: user_001" \
  -H "X-Organization-ID: org_001" \
  -d '{...}' | jq
```

---

### 问题 4: 文件上传失败

**症状**: 提交表单后显示错误

**解决**:
1. 确保文件是 `.wasm` 格式
2. 确保 WASM 文件路径正确
3. 先编译插件: `bash build_plugins.sh`
4. 使用正确路径: `target/wasm32-wasip1/release/plugin.wasm`

---

## 📖 深入学习

### 核心文档
- [plugin.md](./plugin.md) - 插件系统完整设计 (v2.4)
- [PLUGIN_UI_IMPLEMENTATION.md](./PLUGIN_UI_IMPLEMENTATION.md) - UI 实现详解
- [PLUGIN_UI_FEATURES.md](./PLUGIN_UI_FEATURES.md) - UI 功能清单
- [PLUGIN_UI_COMPLETE_SUMMARY.md](./PLUGIN_UI_COMPLETE_SUMMARY.md) - 完整总结

### 示例代码
- `examples/hello_plugin/` - 简单 Hello World 插件
- `examples/memory_processor_plugin/` - 内存处理插件
- `examples/code_analyzer_plugin/` - 代码分析插件
- `examples/llm_plugin/` - LLM 集成插件

### API 参考
- `crates/agent-mem-plugin-sdk/` - 插件开发 SDK
- `crates/agent-mem-plugins/` - 插件管理器
- `agentmem-ui/src/lib/api-client.ts` - 前端 API 客户端

---

## 💡 最佳实践

### 插件开发

1. **使用 SDK**: 导入 `agent-mem-plugin-sdk` 
2. **实现 trait**: 根据插件类型实现相应 trait
3. **测试**: 编写单元测试和集成测试
4. **文档**: 添加 README 和注释
5. **版本管理**: 遵循语义化版本

### UI 使用

1. **定期刷新**: 点击 Refresh 更新列表
2. **查看详情**: 使用 View Details 了解插件信息
3. **监控状态**: 关注统计卡片的变化
4. **合理命名**: 使用清晰的插件名称和描述

### 性能优化

1. **缓存利用**: 前端自动缓存 30 秒
2. **批量操作**: 一次注册多个插件
3. **资源限制**: 配置合理的资源限制
4. **监控告警**: 关注插件性能指标

---

## 🆘 获取帮助

### 常见问题
1. 查看本文档的故障排查部分
2. 运行 `bash scripts/test_plugin_ui.sh`
3. 查看浏览器控制台错误
4. 查看后端日志: `tail -f backend-plugins.log`

### 相关资源
- GitHub Issues: 报告 bug 和功能请求
- API 文档: http://localhost:8080/swagger-ui/
- 完整文档: [plugin.md](./plugin.md)

---

## 🎉 下一步

### 探索功能
1. 尝试注册自定义插件
2. 测试不同类型的插件
3. 查看插件调用效果
4. 监控性能指标

### 参与开发
1. 阅读完整设计文档
2. 查看示例代码
3. 开发自己的插件
4. 贡献代码改进

---

**快速开始版本**: v1.0  
**最后更新**: 2025-11-05  
**文档状态**: ✅ 完成

---

<div align="center">
  <strong>🚀 开始你的插件之旅吧！</strong>
</div>

