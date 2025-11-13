# AgentMem 项目结构

## 📁 顶级目录结构

```
agentmen/
├── README.md                    # 项目主文档
├── CONTRIBUTING.md              # 贡献指南
├── CHANGELOG.md                 # 变更日志
├── TROUBLESHOOTING.md           # 故障排查
├── LICENSE                      # MIT 许可证
├── Cargo.toml                   # Rust 工作空间配置
├── Cargo.lock                   # 依赖锁定文件
│
├── crates/                      # Rust 代码库
│   ├── agent-mem/               # 核心记忆库
│   ├── agent-mem-server/        # HTTP 服务器
│   ├── agent-mem-mcp/           # MCP 协议实现
│   ├── agent-mem-plugins/       # 插件系统
│   ├── agent-mem-storage/       # 存储层
│   ├── agent-mem-embeddings/    # 向量嵌入
│   ├── agent-mem-llm/           # LLM 集成
│   └── ...                      # 其他 crates
│
├── agentmem-ui/                 # Next.js 前端
│   ├── src/                     # 源代码
│   ├── public/                  # 静态资源
│   ├── package.json             # NPM 配置
│   └── next.config.ts           # Next.js 配置
│
├── docs/                        # 📚 文档中心
│   ├── README.md                # 文档首页
│   ├── INDEX.md                 # 文档索引
│   ├── getting-started/         # 快速开始
│   ├── guides/                  # 用户指南
│   ├── architecture/            # 架构文档
│   ├── api/                     # API 文档
│   ├── development/             # 开发文档
│   ├── deployment/              # 部署文档
│   ├── operations/              # 运维文档
│   ├── reports/                 # 实施报告
│   └── archive/                 # 归档文档
│
├── examples/                    # 示例代码
│   ├── demo-*/                  # 各种演示
│   └── README.md                # 示例说明
│
├── scripts/                     # 工具脚本
│   ├── cleanup-docs.sh          # 文档清理
│   ├── build.sh                 # 构建脚本
│   ├── test_*.sh                # 测试脚本
│   └── ...                      # 其他脚本
│
├── tests/                       # 集成测试
│   └── integration_*.rs         # 测试文件
│
├── benchmarks/                  # 性能基准测试
│   └── README.md
│
├── migrations/                  # 数据库迁移
│   └── *.sql                    # SQL 迁移文件
│
├── config/                      # 配置文件
│   ├── agentmem.example.toml    # 配置示例
│   └── agentmem.toml            # 实际配置
│
├── docker/                      # Docker 配置
│   ├── Dockerfile.optimized     # 优化的 Dockerfile
│   └── docker-compose.yml       # Docker Compose
│
├── k8s/                         # Kubernetes 配置
│   ├── deployment.yaml          # K8s 部署
│   └── helm/                    # Helm Charts
│
├── terraform/                   # 基础设施即代码
│   └── aws/                     # AWS 配置
│
├── monitoring/                  # 监控配置
│   ├── grafana/                 # Grafana 仪表板
│   └── prometheus/              # Prometheus 配置
│
├── sdks/                        # 多语言 SDK
│   ├── python/                  # Python SDK
│   ├── javascript/              # JavaScript SDK
│   ├── go/                      # Go SDK
│   └── cangjie/                 # 仓颉 SDK
│
├── tools/                       # 开发工具
│   ├── agentmem-cli/            # CLI 工具
│   └── performance-benchmark/   # 性能测试工具
│
├── lib/                         # 动态库
│   ├── libonnxruntime.1.22.0.dylib
│   └── libonnxruntime.dylib
│
├── data/                        # 数据目录
│   ├── agentmem.db              # SQLite 数据库
│   ├── history.db               # 历史记录
│   └── vectors.lance/           # 向量存储
│
├── dist/                        # 发布包
│   ├── server/                  # 服务器发布包
│   ├── ui/                      # UI 发布包
│   └── README.md                # 部署说明
│
├── target/                      # Rust 构建输出
│   ├── debug/                   # 调试构建
│   └── release/                 # 发布构建
│
├── build-release.sh             # 发布构建脚本
├── justfile                     # Just 任务定义
└── .gitignore                   # Git 忽略规则
```

---

## 🗂️ 核心目录说明

### crates/ - Rust 代码库
包含所有 Rust crates，采用模块化架构：

- **agent-mem**: 核心记忆管理库
- **agent-mem-server**: HTTP REST API 服务器
- **agent-mem-mcp**: Model Context Protocol 实现
- **agent-mem-plugins**: WASM 插件系统
- **agent-mem-storage**: 存储抽象层（SQLite, LanceDB）
- **agent-mem-embeddings**: 向量嵌入（FastEmbed, OpenAI）
- **agent-mem-llm**: LLM 集成（OpenAI, 智谱 AI, Ollama）
- **agent-mem-intelligence**: 智能推理功能
- **agent-mem-core**: 核心类型和 traits
- **agent-mem-utils**: 工具函数

### agentmem-ui/ - 前端应用
基于 Next.js 15.5.2 的现代 Web UI：

- **src/app**: App Router 页面
- **src/components**: React 组件
- **src/lib**: 工具函数和 API 客户端
- **public**: 静态资源

### docs/ - 文档中心
所有项目文档的集中位置：

- **getting-started**: 快速开始和入门教程
- **guides**: 用户指南和操作手册
- **architecture**: 架构设计和技术文档
- **api**: API 参考和接口文档
- **development**: 开发指南和问题分析
- **deployment**: 部署指南和配置说明
- **operations**: 运维指南和监控文档
- **reports**: 实施报告和进度总结
- **archive**: 归档文档和历史资料

### examples/ - 示例代码
各种使用场景的示例：

- **demo-chat**: 聊天应用示例
- **demo-personal-assistant**: 个人助手示例
- **demo-fitness-assistant**: 健身助手示例
- **demo-python-***: Python SDK 示例
- **plugin-***: 插件开发示例

### scripts/ - 工具脚本
自动化脚本和工具：

- **build.sh**: 构建脚本
- **cleanup-docs.sh**: 文档清理
- **test_*.sh**: 各种测试脚本
- **backup.sh**: 数据备份
- **restore.sh**: 数据恢复

### dist/ - 发布包
构建后的发布包：

- **server/**: 后端服务器（二进制 + 库 + 脚本）
- **ui/**: 前端应用（Next.js standalone）
- **README.md**: 部署说明

---

## 📊 文件统计

### 代码文件
- Rust 源文件: 200+ 个
- TypeScript/React 文件: 100+ 个
- 测试文件: 50+ 个

### 文档文件
- 根目录文档: 4 个
- docs/ 文档: 200+ 个
- 示例文档: 20+ 个

### 配置文件
- Cargo.toml: 20+ 个（各 crate）
- package.json: 2 个（UI + Python）
- Docker/K8s 配置: 10+ 个

---

## 🎯 关键文件

### 必读文档
1. **README.md** - 项目概览和快速开始
2. **CONTRIBUTING.md** - 如何贡献代码
3. **TROUBLESHOOTING.md** - 常见问题解决
4. **docs/README.md** - 文档中心入口

### 配置文件
1. **Cargo.toml** - Rust 工作空间配置
2. **config/agentmem.toml** - 服务器配置
3. **agentmem-ui/next.config.ts** - 前端配置
4. **justfile** - 任务定义

### 构建脚本
1. **build-release.sh** - 发布构建
2. **scripts/build.sh** - 开发构建
3. **docker/Dockerfile.optimized** - Docker 构建

---

## 🔧 开发工作流

### 1. 克隆项目
```bash
git clone <repository>
cd agentmen
```

### 2. 安装依赖
```bash
# Rust
cargo build

# 前端
cd agentmem-ui
npm install
```

### 3. 开发
```bash
# 后端
cargo run --package agent-mem-server

# 前端
cd agentmem-ui
npm run dev
```

### 4. 测试
```bash
# Rust 测试
cargo test

# 前端测试
cd agentmem-ui
npm test
```

### 5. 构建发布
```bash
./build-release.sh --all
```

---

## 📝 维护指南

### 添加新功能
1. 在 `crates/` 中创建或修改代码
2. 添加测试到 `tests/`
3. 添加示例到 `examples/`
4. 更新文档到 `docs/`
5. 更新 `CHANGELOG.md`

### 添加新文档
1. 确定文档类型和目标目录
2. 使用规范的命名格式
3. 更新 `docs/INDEX.md`
4. 更新 `CHANGELOG.md`

### 发布新版本
1. 更新版本号（Cargo.toml, package.json）
2. 更新 `CHANGELOG.md`
3. 运行 `./build-release.sh --all`
4. 测试发布包
5. 创建 Git tag
6. 发布到仓库

---

## 🔗 相关链接

- [项目主页](README.md)
- [文档中心](docs/README.md)
- [贡献指南](CONTRIBUTING.md)
- [变更日志](CHANGELOG.md)
- [故障排查](TROUBLESHOOTING.md)

