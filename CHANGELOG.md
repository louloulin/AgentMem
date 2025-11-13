# Changelog

All notable changes to AgentMem will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 完整的文档重组和清理
- 故障排查指南 (TROUBLESHOOTING.md)
- 问题分析文档 (docs/development/issue-analysis.md)
- 构建改进文档 (docs/development/build-improvements.md)

### Fixed
- SQLite URL 格式错误（从 `sqlite://` 改为 `sqlite:///` 或 `file:`）
- FastEmbed 首次启动时缺少进度提示
- Next.js 构建错误（useSearchParams Suspense 边界问题）
- 数据库连接失败问题

### Changed
- 重组文档结构，将 180+ 个根目录 MD 文件移动到 docs/ 目录
- 统一启动脚本中的 DATABASE_URL 格式
- 改进构建脚本，自动复制 ONNX Runtime 库文件

## [0.4.0] - 2025-11-10

### Added
- MCP 2.0 完整实现
- 插件系统 v2.1
- 智能搜索功能
- 记忆隔离和作用域支持
- 完整的 UI 聊天界面

### Fixed
- 记忆搜索返回零结果问题
- Agent ID 可选性问题
- 向量存储问题
- 流式响应错误

### Changed
- 升级到 Next.js 15.5.2
- 改进记忆检索算法
- 优化向量搜索性能

## [0.3.0] - 2025-10-27

### Added
- 插件系统基础实现
- 天气插件示例
- 插件 UI 管理界面
- 插件注册和发现机制

### Fixed
- 编译错误修复
- Embedder 配置问题

### Changed
- 重构记忆架构
- 改进 API 设计

## [0.2.0] - 2025-10-15

### Added
- 记忆历史功能
- 流式聊天实现
- 工具调用支持
- 性能测试指南

### Fixed
- 长期记忆检索问题
- UI 用户 ID 不匹配问题

### Changed
- 优化数据库查询
- 改进错误处理

## [0.1.0] - 2025-09-01

### Added
- 初始版本发布
- 基础记忆存储和检索
- REST API
- 基础 Web UI
- Docker 支持
- 基础文档

---

## 版本说明

### 版本号规则
- **主版本号 (Major)**: 不兼容的 API 变更
- **次版本号 (Minor)**: 向后兼容的功能新增
- **修订号 (Patch)**: 向后兼容的问题修复

### 变更类型
- **Added**: 新功能
- **Changed**: 现有功能的变更
- **Deprecated**: 即将废弃的功能
- **Removed**: 已移除的功能
- **Fixed**: 问题修复
- **Security**: 安全相关的修复

---

## 链接

- [项目主页](https://github.com/yourusername/agentmem)
- [问题追踪](https://github.com/yourusername/agentmem/issues)
- [发布页面](https://github.com/yourusername/agentmem/releases)

