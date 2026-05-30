# AgentMem v11.0 - UI增强与功能完善计划

> **📅 日期**: 2026-05-26  
> **状态**: 待开始  
> **版本**: v11.0  
> **前置依赖**: plan36.md (v10.0 UI集成已完成)

---

## 一、现状分析

### 1.1 plan36.md 完成状态 ✅

| 模块 | 功能 | 数量 | 状态 |
|------|------|------|------|
| **UI 组件** | 7个组件 | 7/7 | ✅ |
| **后端 API** | 4个端点 | 4/4 | ✅ |
| **页面更新** | 4个页面 | 4/4 | ✅ |

### 1.2 当前状态概览

#### 已实现的 UI 功能
- ✅ Importance 评分星级/标签显示
- ✅ Decay 衰减进度条/状态徽章
- ✅ TimeRangeSelector 时间筛选
- ✅ SearchAnalyticsPanel 搜索分析面板 (Mock数据)
- ✅ MultimodalUploader 图片上传组件 (Mock后端)
- ✅ 记忆列表 Importance/Decay 列
- ✅ 记忆详情页健康显示
- ✅ 图谱节点 Importance/Decay 可视化

#### 缺失的 UI 功能
- ❌ 导出功能 (JSON/CSV)
- ❌ 记忆自动整合对话框
- ❌ 多模态管理页
- ❌ SearchAnalytics 真实数据集成
- ❌ CLIP/SigLIP 向量生成真实化

### 1.3 缺口分析

| 优先级 | 功能 | 缺口 | 影响 |
|--------|------|------|------|
| P0 | SearchAnalytics 真实数据 | 无 API 端点 | 搜索分析不可用 |
| P1 | 导出功能 | 无导出组件 | 数据导出困难 |
| P1 | 记忆整合 UI | 无整合逻辑 | 记忆管理不完整 |
| P2 | 多模态管理页 | 无管理界面 | 多模态功能孤立 |
| P2 | CLIP/SigLIP 集成 | Mock 实现 | 向量质量低 |

---

## 二、P0 任务 - SearchAnalytics 真实数据集成

### 2.1 目标

将 `SearchAnalytics` 模块集成到 API 层，提供真实的搜索统计数据。

### 2.2 需要实现

#### 后端 API
| 端点 | 方法 | 功能 |
|------|------|------|
| `POST /api/v1/analytics/search/record` | POST | 记录搜索事件 |
| `GET /api/v1/analytics/search/report` | GET | 获取分析报告 |
| `GET /api/v1/analytics/search/patterns` | GET | 获取查询模式 |
| `GET /api/v1/analytics/search/distribution` | GET | 获取结果分布 |

#### 文件变更
| 文件 | 变更 |
|------|------|
| `crates/agent-mem-server/src/routes/search_analytics.rs` | 新增 |
| `crates/agent-mem-server/src/routes/mod.rs` | 添加路由 |

### 2.3 UI 更新
| 文件 | 改动 |
|------|------|
| `agentmem-ui/src/components/charts/search-analytics-panel.tsx` | 使用真实 API |

---

## 三、P1 任务 - 导出功能

### 3.1 目标

在 `/admin/memories` 页面添加导出功能，支持 JSON/CSV 格式。

### 3.2 需要实现

#### UI 组件
| 组件 | 文件 | 功能 |
|------|------|------|
| `ExportDialog` | `components/export/export-dialog.tsx` | 导出选项对话框 |
| `ExportButton` | `components/export/export-button.tsx` | 导出按钮 |

#### 功能
- [ ] 格式选择 (JSON/CSV)
- [ ] 字段选择 (全选/部分)
- [ ] 日期范围筛选
- [ ] 下载进度显示
- [ ] 导出历史记录

#### 后端 API
| 端点 | 方法 | 功能 |
|------|------|------|
| `GET /api/v1/memories/export` | GET | 导出记忆 (支持格式参数) |

### 3.3 页面更新
| 文件 | 改动 |
|------|------|
| `agentmem-ui/src/app/admin/memories/page.tsx` | 添加导出按钮 |

---

## 四、P1 任务 - 记忆自动整合对话框

### 4.1 目标

实现记忆自动整合功能，帮助用户合并相似的记忆。

### 4.2 需要实现

#### UI 组件
| 组件 | 文件 | 功能 |
|------|------|------|
| `MemoryMergeDialog` | `components/memory-merge-dialog.tsx` | 整合预览对话框 |
| `MergePreview` | `components/merge-preview.tsx` | 整合预览卡片 |
| `MergeConflicts` | `components/merge-conflicts.tsx` | 冲突解决 |

#### 功能
- [ ] 相似记忆检测
- [ ] 整合预览显示
- [ ] 字段冲突解决
- [ ] 确认整合操作
- [ ] 整合历史记录

#### 后端 API
| 端点 | 方法 | 功能 |
|------|------|------|
| `POST /api/v1/memories/similar` | POST | 查找相似记忆 |
| `POST /api/v1/memories/merge` | POST | 整合记忆 |
| `GET /api/v1/memories/merge/preview` | GET | 整合预览 |

### 4.3 页面更新
| 文件 | 改动 |
|------|------|
| `agentmem-ui/src/app/admin/memories/page.tsx` | 添加整合按钮 |

---

## 五、P2 任务 - 多模态管理页

### 5.1 目标

创建独立的多模态管理页面，管理上传的图片和相似搜索。

### 5.2 需要实现

#### 新页面
| 页面 | 路由 | 功能 |
|------|------|------|
| 多模态管理 | `/admin/multimodal` | 图片列表/搜索 |

#### UI 组件
| 组件 | 文件 | 功能 |
|------|------|------|
| `ImageGrid` | `components/multimodal/image-grid.tsx` | 图片网格展示 |
| `ImageCard` | `components/multimodal/image-card.tsx` | 单个图片卡片 |
| `SimilarSearchPanel` | `components/multimodal/similar-search.tsx` | 相似搜索面板 |

#### 功能
- [ ] 图片网格展示
- [ ] 图片上传
- [ ] 相似图片搜索
- [ ] 图片删除
- [ ] 图片详情弹窗

#### 页面结构
```
/admin/multimodal
├── 图片上传区
├── 搜索栏
├── 图片网格
│   ├── ImageCard × N
│   └── 分页控制
└── 相似搜索面板 (点击图片后展开)
```

### 5.3 依赖
- 后端 `/api/v1/multimodal/*` 端点 (已在 plan36 实现)

---

## 六、P2 任务 - CLIP/SigLIP 向量生成真实化

### 6.1 目标

将 Mock 向量生成替换为真实的 CLIP/SigLIP 模型。

### 6.2 需要实现

#### 后端
| 组件 | 文件 | 功能 |
|------|------|------|
| `CLIPVectorizer` | `crates/agent-mem-core/src/vectorizer/clip.rs` | CLIP 模型封装 |
| `SigLIPVectorizer` | `crates/agent-mem-core/src/vectorizer/siglip.rs` | SigLIP 模型封装 |
| `VectorizerFactory` | `crates/agent-mem-core/src/vectorizer/factory.rs` | 向量生成器工厂 |

#### 配置
| 配置项 | 说明 |
|--------|------|
| `vectorizer.model` | 模型类型 (clip/siglip/mock) |
| `vectorizer.device` | 运行设备 (cpu/cuda) |
| `vectorizer.batch_size` | 批处理大小 |

### 6.3 注意事项
- 需要外部模型文件或 API
- GPU 支持可选
- 考虑使用云 API (OpenAI, Replicate 等)

---

## 七、技术实现路径

### 7.1 Phase 1: P0 SearchAnalytics 集成 (1天)

```
Day 1:
- [ ] 创建 search_analytics 路由
- [ ] 集成到 router
- [ ] 更新 UI 使用真实 API
- [ ] 验证功能
```

### 7.2 Phase 2: P1 导出功能 (1天)

```
Day 2:
- [ ] 创建导出对话框组件
- [ ] 添加后端导出 API
- [ ] 集成到 memories 页面
- [ ] 测试导出
```

### 7.3 Phase 3: P1 记忆整合 (1天)

```
Day 3:
- [ ] 创建整合对话框组件
- [ ] 添加后端整合 API
- [ ] 集成到 memories 页面
- [ ] 测试整合流程
```

### 7.4 Phase 4: P2 多模态管理页 (1天)

```
Day 4:
- [ ] 创建多模态页面
- [ ] 实现图片网格组件
- [ ] 实现相似搜索面板
- [ ] 测试完整流程
```

### 7.5 Phase 5: P2 CLIP/SigLIP 集成 (可选, 2天)

```
Day 5-6:
- [ ] 实现向量生成器接口
- [ ] 集成 CLIP/SigLIP 模型
- [ ] 添加配置支持
- [ ] 测试向量质量
```

---

## 八、验证清单

### 8.1 编译验证

```bash
# Rust 后端
cargo check -p agent-mem-core
cargo check -p agent-mem-server

# UI 前端
cd agentmem-ui && npm run build
```

### 8.2 功能验证

| 功能 | 验证方式 |
|------|----------|
| SearchAnalytics | 搜索后查看统计数据更新 |
| 导出功能 | 导出 JSON/CSV 并验证内容 |
| 记忆整合 | 选择记忆并整合，验证合并结果 |
| 多模态管理 | 上传图片并搜索相似 |
| CLIP/SigLIP | 对比 Mock 和真实向量质量 |

---

## 九、工作量估算

| 阶段 | 优先级 | 任务 | 估算时间 |
|------|--------|------|----------|
| Phase 1 | P0 | SearchAnalytics 集成 | 1天 |
| Phase 2 | P1 | 导出功能 | 1天 |
| Phase 3 | P1 | 记忆整合 | 1天 |
| Phase 4 | P2 | 多模态管理页 | 1天 |
| Phase 5 | P2 | CLIP/SigLIP 集成 | 2天 (可选) |
| **总计** | | | **4-6天** |

---

## 十、风险与依赖

### 10.1 依赖

| 功能 | 依赖项 |
|------|--------|
| SearchAnalytics | 后端 search_analytics 模块 (已实现) |
| 导出功能 | 无外部依赖 |
| 记忆整合 | 相似度算法 (可复用现有) |
| 多模态管理 | 后端 multimodal API (已实现) |
| CLIP/SigLIP | 外部模型文件/API |

### 10.2 风险

| 功能 | 风险 | 缓解措施 |
|------|------|----------|
| SearchAnalytics | 数据量影响性能 | 分页/缓存 |
| 导出功能 | 大文件导出超时 | 流式导出 |
| 记忆整合 | 整合逻辑复杂 | 分步确认 |
| CLIP/SigLIP | 模型加载慢 | 异步加载/缓存 |

---

## 十一、后续展望

### 11.1 v11.x 功能

- [ ] 记忆自动标签
- [ ] 智能记忆推荐
- [ ] 记忆分享功能
- [ ] 多语言支持增强

### 11.2 v12.x 功能

- [ ] 分布式部署支持
- [ ] 实时协作
- [ ] 移动端适配
- [ ] PWA 支持

---

**下一步**: Phase 1 - SearchAnalytics 真实数据集成

---

## 十二、2026-05-26 实现记录

### 12.1 后端 multimodal API 已完成

| 端点 | 方法 | 状态 | 验证 |
|------|------|------|------|
| `/api/v1/multimodal/upload` | POST | ✅ | cargo check 通过 |
| `/api/v1/multimodal/search` | POST | ✅ | cargo check 通过 |
| `/api/v1/multimodal/stats` | GET | ✅ | cargo check 通过 |
| `/api/v1/multimodal/health` | GET | ✅ | cargo check 通过 |

### 12.2 SearchAnalytics API

| 端点 | 方法 | 状态 | 说明 |
|------|------|------|------|
| 创建 API 路由 | - | ✅ | search_analytics.rs 已创建 |
| 集成到 router | - | ⏳ | 暂缓 - 类型复杂度高 |

### 12.3 编译验证

```bash
cargo check -p agent-mem-server
# ✅ 编译成功
```

### 12.4 完成状态

| 任务 | 状态 | 说明 |
|------|------|------|
| P0: SearchAnalytics 真实数据 | ⏳ | 后端路由已创建，集成暂缓 |
| P1: 导出功能 | ⏳ | 待实现 |
| P1: 记忆整合 | ⏳ | 待实现 |
| P2: 多模态管理页 | ⏳ | 待实现 |

### 12.5 备注

- multimodal API 已完成并可编译
- search_analytics 路由文件已创建，但 Axum 路由合并存在类型复杂性
- 建议后续 PR 中完成 search_analytics 集成
