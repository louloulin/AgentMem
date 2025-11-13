# AgentMem 文档清理与重组计划

## 当前问题

### 严重问题
- **根目录有 180+ 个 Markdown 文件** ❌
- 文档分类混乱，难以查找
- 大量重复和过时的文档
- 缺乏统一的文档索引

### 影响
- 新用户无法快速上手
- 开发者难以找到相关文档
- 项目看起来不专业
- 维护成本高

---

## 文档分类标准

### 1. 保留在根目录的文档（仅 5 个）
```
README.md              # 项目主文档
CONTRIBUTING.md        # 贡献指南
LICENSE               # 许可证
CHANGELOG.md          # 变更日志（新建）
TROUBLESHOOTING.md    # 快速故障排查
```

### 2. docs/ 目录结构
```
docs/
├── README.md                    # 文档索引
├── getting-started/             # 快速开始
│   ├── README.md
│   ├── installation.md
│   ├── quick-start.md
│   └── first-steps.md
├── guides/                      # 用户指南
│   ├── README.md
│   ├── user-guide.md
│   ├── deployment-guide.md
│   ├── configuration-guide.md
│   └── troubleshooting-guide.md
├── architecture/                # 架构文档
│   ├── README.md
│   ├── overview.md
│   ├── memory-system.md
│   ├── plugin-system.md
│   └── mcp-integration.md
├── api/                         # API 文档
│   ├── README.md
│   ├── rest-api.md
│   ├── mcp-api.md
│   └── sdk-reference.md
├── development/                 # 开发文档
│   ├── README.md
│   ├── setup.md
│   ├── building.md
│   ├── testing.md
│   └── contributing.md
├── deployment/                  # 部署文档
│   ├── README.md
│   ├── docker.md
│   ├── kubernetes.md
│   └── production.md
├── operations/                  # 运维文档
│   ├── README.md
│   ├── monitoring.md
│   ├── backup-recovery.md
│   └── performance-tuning.md
├── reports/                     # 实施报告
│   ├── README.md
│   ├── 2025-11/
│   └── archive/
└── archive/                     # 归档文档
    ├── README.md
    └── legacy/
```

---

## 文档分类清单

### A. 核心文档（保留在根目录）
- [x] README.md
- [x] CONTRIBUTING.md
- [x] LICENSE
- [x] TROUBLESHOOTING.md
- [ ] CHANGELOG.md（新建）

### B. 快速开始（docs/getting-started/）
- QUICK_START_PLUGINS.md
- QUICK_START_SEARCH.md
- QUICK_REFERENCE.md
- CLAUDE_CODE_QUICKSTART.md
- START_CLAUDE_CODE.md

### C. 用户指南（docs/guides/）
- AGENTMEM_USER_GUIDE.md
- DEPLOYMENT.md
- HOW_TO_USE_AGENTMEM_IN_CLAUDE.md
- JUSTFILE_GUIDE.md
- UI_TESTING_GUIDE.md
- VERIFICATION_GUIDE.md

### D. 架构文档（docs/architecture/）
- AGENTMEM_FINAL_ARCHITECTURE.md
- AGENTMEM_TECHNICAL_DOCUMENTATION.md
- README_AGENTMEM_ARCHITECTURE_V3.md
- COMPREHENSIVE_MEMORY_ARCHITECTURE_ANALYSIS.md
- ARCHITECTURE_ANALYSIS_COMPLETE.md
- ARCHITECTURE_ANALYSIS_COMPREHENSIVE.md

### E. API 文档（docs/api/）
- MEMORY_API_COMPARATIVE_ANALYSIS.md
- CLAUDE_CODE_MCP_COMPLETE_GUIDE.md
- CORRECT_MCP_COMMANDS.md
- REAL_CLAUDE_COMMANDS.md

### F. 开发文档（docs/development/）
- BUILD_IMPROVEMENTS.md
- ISSUE_ANALYSIS.md
- COMPILATION_FIX_REPORT.md
- EMBEDDER_FIX_REPORT.md

### G. 部署文档（docs/deployment/）
- DEPLOYMENT.md
- DOCKER 相关文档
- K8S 相关文档

### H. 实施报告（docs/reports/2025-11/）
- P0_P1_FINAL_REPORT.md
- FINAL_PROJECT_COMPLETION_REPORT.md
- FINAL_SESSION_SUMMARY_2025-11-10.md
- LATEST_UPDATE_2025-11-10.md
- 所有 *_REPORT.md 文件
- 所有 *_SUMMARY.md 文件
- 所有 *_FIX*.md 文件

### I. 归档文档（docs/archive/）
- 所有过时的文档
- 所有临时笔记（agentmem*.md, claude*.md, bp*.md, cp*.md, x*.md 等）
- 所有重复的文档
- 所有验证报告（verification_report_*.md）

---

## 执行计划

### Phase 1: 创建新的文档结构
1. 创建 docs/ 子目录
2. 创建 README.md 索引文件
3. 创建 CHANGELOG.md

### Phase 2: 移动和整理文档
1. 移动核心文档到对应目录
2. 合并重复文档
3. 更新文档内容和链接

### Phase 3: 归档过时文档
1. 移动过时文档到 archive/
2. 移动临时笔记到 archive/notes/
3. 移动旧报告到 archive/reports/

### Phase 4: 清理和验证
1. 删除空文件和无用文件
2. 更新所有文档链接
3. 验证文档结构

---

## 文档命名规范

### 规则
1. 使用小写字母和连字符：`user-guide.md`
2. 避免使用大写和下划线：~~`USER_GUIDE.md`~~
3. 使用描述性名称：`deployment-guide.md` 而不是 `deploy.md`
4. 日期格式：`YYYY-MM-DD-title.md`

### 示例
```
✅ getting-started.md
✅ api-reference.md
✅ 2025-11-13-build-improvements.md

❌ GETTING_STARTED.md
❌ API_REF.md
❌ build_improvements_20251113.md
```

---

## 优先级

### P0 - 立即执行
1. 创建 docs/ 目录结构
2. 移动核心用户文档
3. 创建文档索引

### P1 - 本周完成
1. 整理所有实施报告
2. 归档过时文档
3. 更新文档链接

### P2 - 后续优化
1. 统一文档格式
2. 添加文档搜索
3. 生成 API 文档

---

## 预期效果

### 清理后的根目录
```
agentmen/
├── README.md                 # 项目主文档
├── CONTRIBUTING.md           # 贡献指南
├── LICENSE                   # 许可证
├── CHANGELOG.md              # 变更日志
├── TROUBLESHOOTING.md        # 快速故障排查
├── Cargo.toml
├── build-release.sh
├── justfile
├── crates/
├── docs/                     # 所有文档
├── examples/
├── scripts/
└── ...
```

### 文档数量对比
- **清理前**: 根目录 180+ 个 MD 文件
- **清理后**: 根目录 5 个 MD 文件
- **改善**: 减少 97% 的根目录文件

---

## 下一步行动

1. ✅ 创建此计划文档
2. [ ] 执行 Phase 1: 创建目录结构
3. [ ] 执行 Phase 2: 移动文档
4. [ ] 执行 Phase 3: 归档文档
5. [ ] 执行 Phase 4: 清理验证

