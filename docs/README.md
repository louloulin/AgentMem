# AgentMem 文档中心

**最后更新**: 2025-10-27  
**文档版本**: v2.0 - 重新组织  

---

## 📚 文档导航

### 🎯 核心计划文档

#### [MVP规划](/mvp-planning/)
- **[agentmem37.md](../agentmem37.md)** ⭐️ 生产MVP开发计划（主计划）
- **[MVP_IMPLEMENTATION_DETAILS_ROADMAP.md](/mvp-planning/MVP_IMPLEMENTATION_DETAILS_ROADMAP.md)** - 详细实施路线图
- **[FINAL_MVP_ANALYSIS_EXECUTIVE_SUMMARY_20251027.md](/mvp-planning/FINAL_MVP_ANALYSIS_EXECUTIVE_SUMMARY_20251027.md)** - 执行概要

#### [竞品分析](/competitive-analysis/)
- **[COMPETITIVE_ANALYSIS_AND_TECHNICAL_DEEP_DIVE.md](/competitive-analysis/COMPETITIVE_ANALYSIS_AND_TECHNICAL_DEEP_DIVE.md)** - 竞品对比与技术深度分析
- **[WEB_UI_DISCOVERY_AND_REASSESSMENT.md](/competitive-analysis/WEB_UI_DISCOVERY_AND_REASSESSMENT.md)** - Web UI发现报告

#### [性能分析](/performance/)
- 性能基准测试报告
- 性能优化方案
- 性能对比分析

#### [代码库分析](/codebase-analysis/)
- **[COMPREHENSIVE_CODEBASE_ANALYSIS_20251027.md](/codebase-analysis/COMPREHENSIVE_CODEBASE_ANALYSIS_20251027.md)** - 全面代码分析
- **[agentmem36.md](../agentmem36.md)** - 功能对比报告

---

## 📖 使用指南

### 快速开始
1. 阅读 [agentmem37.md](../agentmem37.md) 了解整体计划
2. 查看 [FINAL_MVP_ANALYSIS_EXECUTIVE_SUMMARY_20251027.md](/mvp-planning/FINAL_MVP_ANALYSIS_EXECUTIVE_SUMMARY_20251027.md) 了解执行概要
3. 深入 [MVP_IMPLEMENTATION_DETAILS_ROADMAP.md](/mvp-planning/MVP_IMPLEMENTATION_DETAILS_ROADMAP.md) 获取技术细节

### 开发者指南
- [API参考](/api/) - REST API文档
- [架构设计](/architecture/) - 系统架构
- [部署指南](/deployment/) - 生产部署
- [开发指南](/development/) - 开发规范

### SDK文档
- [Python SDK](/sdks/python/) - Python使用指南
- [仓颉SDK](/sdks/cangjie/) - 仓颉语言SDK

---

## 🗂️ 文档结构

```
docs/
├── README.md                          # 本文件 - 文档导航
├── mvp-planning/                      # MVP规划
│   ├── agentmem37.md                 # 主计划（软链接）
│   ├── MVP_IMPLEMENTATION_DETAILS_ROADMAP.md
│   └── FINAL_MVP_ANALYSIS_EXECUTIVE_SUMMARY_20251027.md
├── competitive-analysis/              # 竞品分析
│   ├── COMPETITIVE_ANALYSIS_AND_TECHNICAL_DEEP_DIVE.md
│   └── WEB_UI_DISCOVERY_AND_REASSESSMENT.md
├── performance/                       # 性能分析
│   ├── benchmarks/
│   └── optimization/
├── codebase-analysis/                 # 代码分析
│   ├── COMPREHENSIVE_CODEBASE_ANALYSIS_20251027.md
│   └── agentmem36.md                 # 软链接
├── web-ui/                           # Web UI文档
│   └── implementation/
├── python-sdk/                       # Python SDK
│   └── PYTHON_USAGE_GUIDE.md
├── implementation/                    # 实施报告
│   ├── P0_FIX_COMPLETE_20251027.md
│   └── PYTHON_BINDINGS_VERIFICATION_20251027.md
├── progress-reports/                  # 进度报告
│   └── 2025-10/
├── api/                              # API文档
├── architecture/                     # 架构文档
├── deployment/                       # 部署文档
├── development/                      # 开发文档
├── sdks/                            # SDK文档
├── user-guide/                      # 用户指南
└── archive/                         # 归档文档
    ├── assessments/
    ├── daily/
    ├── weekly/
    ├── tasks/
    └── misc/
```

---

## 🔍 按主题查找文档

### MVP和规划
- [agentmem37.md](../agentmem37.md) - 完整开发计划
- [agentmem36.md](../agentmem36.md) - 功能对比
- [agentmem35.md](/archived-legacy/agentmem35.md) - 历史版本

### 分析报告
- 竞品分析
- 性能分析
- 代码分析
- 进度分析

### 实施指南
- 性能优化
- Web UI开发
- Python绑定
- 测试验证

### 开发文档
- API参考
- 架构设计
- 部署指南
- SDK文档

---

## 📝 文档贡献指南

### 文档规范
1. 所有新文档放在对应的主题目录
2. 使用清晰的文件命名（大写+下划线或小写+连字符）
3. 包含创建日期和版本号
4. 添加到对应目录的README

### 命名规范
- 计划文档: `PLAN_*.md` 或 `agentmemXX.md`
- 分析报告: `*_ANALYSIS_*.md`
- 实施报告: `*_IMPLEMENTATION_*.md` 或 `*_COMPLETE_*.md`
- 指南文档: `*_GUIDE.md`
- 进度报告: `*_PROGRESS_*.md` 或 `*_SUMMARY_*.md`

---

## 🔗 重要链接

- [项目主README](../README.md)
- [快速开始](/user-guide/quickstart.md)
- [API参考](/api/reference.md)
- [贡献指南](/development/contributing.md)

---

**维护**: AgentMem文档团队  
**反馈**: 请提交Issue或PR
