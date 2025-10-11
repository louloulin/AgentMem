# AgentMem 文档重组计划

**创建日期**: 2025-01-10  
**目标**: 将所有 Markdown 文档按功能分类整理到 docs/ 目录

---

## 📋 文档分类体系

### 1. **架构设计文档** (`docs/architecture/`)
- 系统架构
- 技术选型
- 设计决策

### 2. **API 文档** (`docs/api/`)
- API 参考
- 接口规范
- 使用示例

### 3. **部署文档** (`docs/deployment/`)
- 部署指南
- Docker 配置
- K8s 配置
- 生产环境

### 4. **开发文档** (`docs/development/`)
- 开发指南
- 贡献指南
- 代码规范

### 5. **测试文档** (`docs/testing/`)
- 测试报告
- 测试计划
- 性能基准

### 6. **项目管理文档** (`docs/project/`)
- 项目计划
- 进度报告
- 完成报告
- 任务追踪

### 7. **用户文档** (`docs/user-guide/`)
- 快速开始
- 使用教程
- 最佳实践
- 故障排除

### 8. **SDK 文档** (`docs/sdks/`)
- Python SDK
- Cangjie SDK
- JavaScript SDK
- Go SDK

### 9. **集成文档** (`docs/integrations/`)
- Fluvio 集成
- LLM 集成
- 存储集成
- Mem0 兼容

### 10. **历史文档** (`docs/archive/`)
- 过期文档
- 历史报告
- 旧版本文档

---

## 🗂️ 文档映射表

### 架构设计文档 → `docs/architecture/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `FLUVIO_AGENTMEM_ARCHITECTURE.md` | `docs/architecture/fluvio-integration.md` | Fluvio 集成架构 |
| `ARCHITECTURE_OPTIMIZATION_SUMMARY.md` | `docs/architecture/optimization.md` | 架构优化总结 |
| `ARCHITECTURE_ISSUES.md` | `docs/architecture/issues.md` | 架构问题分析 |
| `docs/en/architecture.md` | `docs/architecture/overview-en.md` | 架构概览(英文) |
| `docs/zh/architecture.md` | `docs/architecture/overview-zh.md` | 架构概览(中文) |
| `crates/agent-mem-core/DATABASE_SCHEMA.md` | `docs/architecture/database-schema.md` | 数据库 Schema |

### API 文档 → `docs/api/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `docs/api-reference.md` | `docs/api/reference.md` | API 参考 |
| `docs/api_reference.md` | `docs/api/reference-v2.md` | API 参考 v2 |
| `docs/en/api.md` | `docs/api/reference-en.md` | API 参考(英文) |
| `docs/zh/api.md` | `docs/api/reference-zh.md` | API 参考(中文) |
| `sdks/cangjie/API_REFERENCE.md` | `docs/api/cangjie-api.md` | Cangjie API |

### 部署文档 → `docs/deployment/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `PRODUCTION_DEPLOYMENT_GUIDE.md` | `docs/deployment/production-guide.md` | 生产部署指南 |
| `docs/deployment-guide.md` | `docs/deployment/guide.md` | 部署指南 |
| `docs/production-deployment-guide.md` | `docs/deployment/production.md` | 生产部署 |
| `docs/en/deployment.md` | `docs/deployment/guide-en.md` | 部署指南(英文) |
| `docs/zh/deployment.md` | `docs/deployment/guide-zh.md` | 部署指南(中文) |
| `docker/README.md` | `docs/deployment/docker.md` | Docker 部署 |
| `agentmem-website/DEPLOYMENT_GUIDE.md` | `docs/deployment/frontend.md` | 前端部署 |

### 开发文档 → `docs/development/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `CONTRIBUTING.md` | `docs/development/contributing.md` | 贡献指南 |
| `MIGRATION_GUIDE.md` | `docs/development/migration.md` | 迁移指南 |
| `INTELLIGENT_INTEGRATION_GUIDE.md` | `docs/development/integration.md` | 集成指南 |
| `MEM0_COMPATIBILITY.md` | `docs/development/mem0-compat.md` | Mem0 兼容 |

### 测试文档 → `docs/testing/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `PERFORMANCE_BENCHMARKS.md` | `docs/testing/benchmarks.md` | 性能基准 |
| `TESTING_AND_MONITORING_REPORT.md` | `docs/testing/monitoring.md` | 测试监控 |
| `VALIDATION_REPORT.md` | `docs/testing/validation.md` | 验证报告 |
| `FUNCTIONAL_VERIFICATION_REPORT.md` | `docs/testing/functional.md` | 功能验证 |
| `测试实施报告_Phase*.md` | `docs/testing/phase-reports/` | 阶段测试报告 |
| `agentmem-website/FRONTEND_TESTING_GUIDE.md` | `docs/testing/frontend-guide.md` | 前端测试 |

### 项目管理文档 → `docs/project/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `PROJECT_SUMMARY.md` | `docs/project/summary.md` | 项目总结 |
| `PROJECT_STATUS_QUICK_REFERENCE.md` | `docs/project/status.md` | 项目状态 |
| `AGENTMEM_PROJECT_PLAN.md` | `docs/project/plan.md` | 项目计划 |
| `P1_TASKS_PROGRESS_SUMMARY.md` | `docs/project/p1-progress.md` | P1 进度 |
| `FINAL_STATUS.md` | `docs/project/final-status.md` | 最终状态 |
| `mem14.3.md` | `docs/project/production-assessment.md` | 生产评估 |

### 用户文档 → `docs/user-guide/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `README.md` | `docs/user-guide/README.md` | 主 README |
| `README_CN.md` | `docs/user-guide/README-zh.md` | 中文 README |
| `docs/quickstart.md` | `docs/user-guide/quickstart.md` | 快速开始 |
| `docs/examples.md` | `docs/user-guide/examples.md` | 示例 |
| `docs/troubleshooting.md` | `docs/user-guide/troubleshooting.md` | 故障排除 |
| `docs/en/getting-started.md` | `docs/user-guide/getting-started-en.md` | 入门(英文) |
| `docs/zh/getting-started.md` | `docs/user-guide/getting-started-zh.md` | 入门(中文) |

### SDK 文档 → `docs/sdks/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `sdks/python/README.md` | `docs/sdks/python/README.md` | Python SDK |
| `sdks/python/CHANGELOG.md` | `docs/sdks/python/CHANGELOG.md` | Python 更新日志 |
| `sdks/cangjie/LIBRARY_README.md` | `docs/sdks/cangjie/README.md` | Cangjie SDK |
| `sdks/cangjie/BEST_PRACTICES.md` | `docs/sdks/cangjie/best-practices.md` | 最佳实践 |
| `sdks/cangjie/TROUBLESHOOTING.md` | `docs/sdks/cangjie/troubleshooting.md` | 故障排除 |

### 集成文档 → `docs/integrations/`

| 源文件 | 目标路径 | 说明 |
|--------|---------|------|
| `FLUVIO_AGENTMEM_ARCHITECTURE.md` | `docs/integrations/fluvio.md` | Fluvio 集成 |
| `PHASE3_LLM_ANALYSIS.md` | `docs/integrations/llm.md` | LLM 集成 |
| `LIBSQL_MIGRATION_COMPLETE.md` | `docs/integrations/libsql.md` | LibSQL 集成 |
| `docs/intelligent-reasoning-engine.md` | `docs/integrations/reasoning-engine.md` | 推理引擎 |

### 历史文档 → `docs/archive/`

所有过期的、重复的、临时的文档都移到这里：
- `DAY*_*.md`
- `WEEK*_*.md`
- `PHASE*_*.md`
- `P0_*.md`, `P1_*.md`, `P2_*.md`
- `FINAL_ASSESSMENT_*.md` (多个版本)
- `WORK_SUMMARY_*.md`
- `测试实施报告_*.md`
- `提交总结_*.md`
- 等等

---

## 🚀 执行步骤

### Step 1: 创建目录结构
```bash
mkdir -p docs/architecture
mkdir -p docs/api
mkdir -p docs/deployment
mkdir -p docs/development
mkdir -p docs/testing/phase-reports
mkdir -p docs/project/phases
mkdir -p docs/user-guide
mkdir -p docs/sdks/{python,cangjie,javascript,go}
mkdir -p docs/integrations
mkdir -p docs/archive/{daily,weekly,phases,tasks,assessments}
```

### Step 2: 移动核心文档
```bash
# 架构文档
mv FLUVIO_AGENTMEM_ARCHITECTURE.md docs/architecture/fluvio-integration.md
mv ARCHITECTURE_OPTIMIZATION_SUMMARY.md docs/architecture/optimization.md

# API 文档
mv docs/api-reference.md docs/api/reference.md

# 部署文档
mv PRODUCTION_DEPLOYMENT_GUIDE.md docs/deployment/production-guide.md

# 用户文档
cp README.md docs/user-guide/README.md
cp README_CN.md docs/user-guide/README-zh.md
```

### Step 3: 归档历史文档
```bash
# 日报
mv DAY*.md docs/archive/daily/

# 周报
mv WEEK*.md docs/archive/weekly/

# 阶段报告
mv PHASE*.md docs/archive/phases/

# 任务报告
mv P0_*.md P1_*.md P2_*.md docs/archive/tasks/

# 评估报告
mv FINAL_ASSESSMENT_*.md docs/archive/assessments/
mv REAL_STATUS_*.md docs/archive/assessments/
```

### Step 4: 清理重复文档
```bash
# 删除重复的 README
rm README_AGENTMEM.md README-AGENTMEM.md README_AUGMENT_AGENT.md

# 删除临时文件
rm test1.md pb1.md ffi1.md augment.md augmentcode.md

# 删除过期的中文报告
rm 任务完成确认.md 执行总结.md 提交完成.md 测试实施完成报告.md 测试计划执行总结.md
```

### Step 5: 更新索引文档
创建 `docs/README.md` 作为文档导航

---

## 📝 文档命名规范

### 命名规则
1. **全小写**: 使用小写字母
2. **连字符**: 使用 `-` 分隔单词
3. **语言后缀**: 英文无后缀，中文 `-zh`，日文 `-ja`
4. **版本号**: 如需版本，使用 `-v2`, `-v3`

### 示例
- ✅ `getting-started.md`
- ✅ `api-reference-zh.md`
- ✅ `deployment-guide-v2.md`
- ❌ `Getting_Started.MD`
- ❌ `API参考.md`

---

## 🔍 文档审查清单

### 每个文档需要包含
- [ ] 标题和简介
- [ ] 创建/更新日期
- [ ] 目录 (如果超过 3 个章节)
- [ ] 代码示例 (如适用)
- [ ] 相关链接
- [ ] 维护者信息

### 文档质量标准
- [ ] 无拼写错误
- [ ] 格式一致
- [ ] 链接有效
- [ ] 代码可运行
- [ ] 截图清晰 (如有)

---

## 📊 统计信息

### 文档总数
- **总计**: 261 个 MD 文件 (不含 node_modules 和 target)
- **根目录**: 约 150 个
- **docs/**: 约 40 个
- **sdks/**: 约 15 个
- **其他**: 约 56 个

### 预期整理后
- **docs/architecture/**: 10 个
- **docs/api/**: 8 个
- **docs/deployment/**: 10 个
- **docs/development/**: 6 个
- **docs/testing/**: 15 个
- **docs/project/**: 20 个
- **docs/user-guide/**: 10 个
- **docs/sdks/**: 15 个
- **docs/integrations/**: 8 个
- **docs/archive/**: 150+ 个

---

## ✅ 验证步骤

### 1. 检查文档完整性
```bash
# 确保所有重要文档都已移动
find . -maxdepth 1 -name "*.md" -type f | wc -l
# 应该只剩下 README.md, LICENSE, CONTRIBUTING.md 等核心文件
```

### 2. 检查链接有效性
```bash
# 使用工具检查所有文档中的链接
markdown-link-check docs/**/*.md
```

### 3. 生成文档索引
```bash
# 自动生成 docs/README.md
tree docs/ -L 2 > docs/STRUCTURE.txt
```

---

**执行时间**: 预计 30-60 分钟  
**负责人**: AgentMem 开发团队  
**审核人**: 项目维护者

