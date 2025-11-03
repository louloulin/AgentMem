#!/bin/bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
git add -A
git commit -m "feat: 实现用户管理功能和 MIRIX 对比分析 (Phase 2)

新增功能:
- 在 AgentMemClient 中添加 User 结构体
- 实现 create_user() 方法
- 实现 list_users() 方法
- 实现 get_user_by_name() 方法
- 创建 user-management-demo 示例
- 更新 mem18.md 到 v2.0

文件变更:
- crates/agent-mem-core/src/client.rs
- Cargo.toml
- examples/user-management-demo/
- doc/technical-design/memory-systems/mem18.md

进度: 30% (基础用户管理 API 完成)"
git log -1 --oneline

