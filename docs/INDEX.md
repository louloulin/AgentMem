# AgentMem 文档索引

## �� 文档组织结构

```
docs/
├── README.md                          # 文档中心首页
├── INDEX.md                           # 本文件 - 完整索引
│
├── getting-started/                   # 🚀 快速开始
│   ├── plugins-quickstart.md          # 插件快速开始
│   ├── search-quickstart.md           # 搜索快速开始
│   ├── quick-reference.md             # 快速参考
│   ├── claude-code-quickstart.md      # Claude Code 快速开始
│   └── start-claude-code.md           # 启动 Claude Code
│
├── guides/                            # 📖 用户指南
│   ├── user-guide.md                  # 用户手册
│   ├── deployment-guide.md            # 部署指南
│   ├── claude-integration.md          # Claude 集成指南
│   ├── justfile-guide.md              # Justfile 使用指南
│   ├── ui-testing.md                  # UI 测试指南
│   └── verification.md                # 验证指南
│
├── architecture/                      # 🏗️ 架构文档
│   ├── final-architecture.md          # 最终架构
│   ├── technical-documentation.md     # 技术文档
│   ├── architecture-v3.md             # 架构 v3
│   ├── memory-architecture-analysis.md # 记忆架构分析
│   ├── analysis-complete.md           # 架构分析完成
│   └── analysis-comprehensive.md      # 综合架构分析
│
├── api/                               # 🔌 API 文档
│   ├── memory-api-comparison.md       # Memory API 对比
│   ├── mcp-complete-guide.md          # MCP 完整指南
│   ├── mcp-commands.md                # MCP 命令参考
│   └── claude-commands.md             # Claude 命令参考
│
├── development/                       # 💻 开发文档
│   ├── build-improvements.md          # 构建改进
│   ├── issue-analysis.md              # 问题分析
│   ├── compilation-fix.md             # 编译修复
│   └── embedder-fix.md                # Embedder 修复
│
├── deployment/                        # 🚢 部署文档
│   └── (现有部署文档)
│
├── operations/                        # 🔧 运维文档
│   └── (现有运维文档)
│
├── reports/                           # 📊 实施报告
│   ├── 2025-11/                       # 2025年11月报告
│   │   ├── agent-id-*.md              # Agent ID 相关
│   │   ├── chat-*.md                  # 聊天功能相关
│   │   ├── mcp-*.md                   # MCP 相关
│   │   ├── plugin-*.md                # 插件相关
│   │   ├── search-*.md                # 搜索相关
│   │   ├── p0-*.md                    # P0 任务相关
│   │   └── final-*.md                 # 最终报告
│   └── archive/                       # 历史报告归档
│
└── archive/                           # 🗄️ 归档文档
    ├── legacy/                        # 旧版文档
    ├── notes/                         # 临时笔记
    └── reports/                       # 旧报告
```

## 🔍 按主题查找

### 快速开始
- [插件快速开始](getting-started/plugins-quickstart.md)
- [搜索快速开始](getting-started/search-quickstart.md)
- [Claude Code 快速开始](getting-started/claude-code-quickstart.md)

### 用户指南
- [完整用户手册](guides/user-guide.md)
- [部署指南](guides/deployment-guide.md)
- [Claude 集成](guides/claude-integration.md)

### 架构设计
- [最终架构](architecture/final-architecture.md)
- [技术文档](architecture/technical-documentation.md)
- [记忆架构分析](architecture/memory-architecture-analysis.md)

### API 参考
- [MCP 完整指南](api/mcp-complete-guide.md)
- [Memory API 对比](api/memory-api-comparison.md)
- [命令参考](api/mcp-commands.md)

### 开发指南
- [构建改进](development/build-improvements.md)
- [问题分析](development/issue-analysis.md)
- [编译修复](development/compilation-fix.md)

### 实施报告
- [2025年11月报告](reports/2025-11/)
- [项目完成报告](reports/2025-11/final-project-completion-report.md)
- [P0/P1 最终报告](reports/2025-11/p0-p1-final-report.md)

## 📊 文档统计

### 清理前后对比
- **清理前**: 根目录 189 个 MD 文件
- **清理后**: 根目录 4 个 MD 文件
- **改善**: 减少 98% 的根目录文件

### 文档分布
- 快速开始: 5 个文档
- 用户指南: 6 个文档
- 架构文档: 6 个文档
- API 文档: 4 个文档
- 开发文档: 4 个文档
- 实施报告: 70+ 个文档
- 归档文档: 100+ 个文档

## 🎯 推荐阅读路径

### 新用户
1. [README.md](../README.md) - 项目概览
2. [快速开始](getting-started/) - 快速上手
3. [用户手册](guides/user-guide.md) - 详细使用
4. [故障排查](../TROUBLESHOOTING.md) - 解决问题

### 开发者
1. [架构文档](architecture/) - 理解架构
2. [API 参考](api/) - API 使用
3. [开发指南](development/) - 开发环境
4. [贡献指南](../CONTRIBUTING.md) - 贡献代码

### 运维人员
1. [部署指南](guides/deployment-guide.md) - 部署应用
2. [运维文档](operations/) - 运维管理
3. [故障排查](../TROUBLESHOOTING.md) - 问题诊断

## 📝 文档维护

### 文档规范
- 使用小写字母和连字符命名：`user-guide.md`
- 避免使用大写和下划线：~~`USER_GUIDE.md`~~
- 使用描述性名称
- 日期格式：`YYYY-MM-DD-title.md`

### 文档更新
- 新文档放入对应目录
- 过时文档移至 archive/
- 更新本索引文件
- 更新 CHANGELOG.md

## 🔗 相关链接

- [项目主页](../README.md)
- [贡献指南](../CONTRIBUTING.md)
- [变更日志](../CHANGELOG.md)
- [故障排查](../TROUBLESHOOTING.md)
- [许可证](../LICENSE)
