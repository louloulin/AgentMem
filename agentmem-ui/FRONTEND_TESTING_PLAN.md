# AgentMem 前端测试计划

**版本**: 1.0  
**日期**: 2025-10-01  
**状态**: 待执行（需要 Node.js 环境）

---

## 📋 测试概览

本文档提供了 AgentMem 前端管理界面的完整测试计划，包括编译测试、功能测试、集成测试和性能测试。

### 前置条件

- ✅ Node.js 18+ 已安装
- ✅ npm/pnpm/yarn 已安装
- ✅ AgentMem 后端 API 服务器运行在 `http://localhost:8080`
- ✅ PostgreSQL 数据库已配置

---

## 1. 编译测试（Compilation Tests）

### 1.1 依赖安装测试

**目的**: 验证所有依赖包可以正确安装

**命令**:
```bash
cd agentmen/agentmem-website
npm install
# 或
pnpm install
# 或
yarn install
```

**预期结果**:
- ✅ 所有依赖包成功安装
- ✅ 无依赖冲突警告
- ✅ `node_modules` 目录创建成功
- ✅ `package-lock.json` 或 `pnpm-lock.yaml` 更新

**失败处理**:
- 检查 Node.js 版本是否 >= 18
- 清除缓存: `npm cache clean --force`
- 删除 `node_modules` 和 lock 文件后重试

---

### 1.2 TypeScript 类型检查

**目的**: 验证所有 TypeScript 代码类型正确

**命令**:
```bash
npx tsc --noEmit
```

**预期结果**:
- ✅ 0 errors
- ✅ 所有类型定义正确
- ✅ 所有导入路径解析成功

**常见错误**:
- `Cannot find module '@/components/ui/...'` - 检查 `tsconfig.json` 的 paths 配置
- `Property '...' does not exist on type` - 检查 API 类型定义

---

### 1.3 生产构建测试

**目的**: 验证代码可以成功构建为生产版本

**命令**:
```bash
npm run build
```

**预期结果**:
- ✅ 构建成功完成
- ✅ `.next` 目录创建
- ✅ 所有页面成功编译
- ✅ 无 TypeScript 错误
- ✅ 无 ESLint 错误（如果启用）

**构建输出示例**:
```
Route (app)                              Size     First Load JS
┌ ○ /                                    5.2 kB         95 kB
├ ○ /admin                               2.1 kB         92 kB
├ ○ /admin/agents                        3.5 kB         93 kB
├ ○ /admin/chat                          4.2 kB         94 kB
├ ○ /admin/memories                      3.8 kB         94 kB
├ ○ /admin/users                         2.3 kB         92 kB
├ ○ /admin/settings                      2.5 kB         92 kB
└ ○ /about                               1.8 kB         91 kB

○  (Static)  prerendered as static content
```

**失败处理**:
- 检查错误日志
- 修复 TypeScript 错误
- 修复导入路径问题
- 检查环境变量配置

---

### 1.4 开发服务器启动测试

**目的**: 验证开发服务器可以正常启动

**命令**:
```bash
npm run dev
```

**预期结果**:
- ✅ 服务器启动在 `http://localhost:3000`
- ✅ 无启动错误
- ✅ 热重载功能正常

**输出示例**:
```
  ▲ Next.js 15.5.2
  - Local:        http://localhost:3000
  - Network:      http://192.168.1.100:3000

 ✓ Ready in 2.5s
```

---

## 2. 功能测试（Functional Tests）

### 2.1 管理界面布局测试

**页面**: `/admin`

**测试项**:
- [ ] 侧边栏正确显示
- [ ] 导航链接可点击
- [ ] Logo 显示正确
- [ ] 深色模式切换正常
- [ ] 响应式布局在不同屏幕尺寸下正常

**测试步骤**:
1. 访问 `http://localhost:3000/admin`
2. 检查侧边栏是否显示所有导航项
3. 点击每个导航链接，验证页面跳转
4. 切换深色模式，验证样式变化
5. 调整浏览器窗口大小，验证响应式布局

---

### 2.2 Dashboard 页面测试

**页面**: `/admin`

**测试项**:
- [ ] 统计卡片正确显示
- [ ] 数据加载正常
- [ ] 最近活动列表显示
- [ ] 图标正确渲染

**测试步骤**:
1. 访问 Dashboard
2. 验证 4 个统计卡片显示
3. 检查数据是否为模拟数据或实际数据
4. 验证活动列表显示

---

### 2.3 Agents 管理页面测试

**页面**: `/admin/agents`

**测试项**:
- [ ] Agent 列表正确加载
- [ ] 创建 Agent 对话框打开
- [ ] 创建 Agent 功能正常
- [ ] 删除 Agent 功能正常
- [ ] 状态指示器正确显示
- [ ] 空状态提示显示

**测试步骤**:
1. 访问 `/admin/agents`
2. 点击 "Create Agent" 按钮
3. 填写表单并提交
4. 验证新 Agent 出现在列表中
5. 点击删除按钮
6. 确认删除对话框
7. 验证 Agent 从列表中移除

**API 调用验证**:
- `GET /api/v1/agents` - 获取 Agent 列表
- `POST /api/v1/agents` - 创建 Agent
- `DELETE /api/v1/agents/:id` - 删除 Agent

---

### 2.4 Chat 界面测试

**页面**: `/admin/chat`

**测试项**:
- [ ] Agent 选择下拉框正常
- [ ] 消息输入框可用
- [ ] 发送消息功能正常
- [ ] 消息历史显示
- [ ] 自动滚动到底部
- [ ] 加载指示器显示

**测试步骤**:
1. 访问 `/admin/chat`
2. 从下拉框选择一个 Agent
3. 输入消息并发送
4. 验证消息出现在聊天历史中
5. 验证 Agent 响应（模拟或实际）
6. 发送多条消息，验证自动滚动

---

### 2.5 Memories 管理页面测试

**页面**: `/admin/memories`

**测试项**:
- [ ] Memory 列表正确加载
- [ ] Agent 过滤器正常
- [ ] 类型过滤器正常
- [ ] 搜索功能正常
- [ ] 删除 Memory 功能正常
- [ ] 类型徽章颜色正确

**测试步骤**:
1. 访问 `/admin/memories`
2. 选择一个 Agent
3. 验证 Memory 列表加载
4. 使用类型过滤器
5. 使用搜索功能
6. 删除一个 Memory

**API 调用验证**:
- `GET /api/v1/agents/:id/memories` - 获取 Memory 列表
- `GET /api/v1/memories/search?query=...` - 搜索 Memories
- `DELETE /api/v1/memories/:id` - 删除 Memory

---

### 2.6 Users 管理页面测试

**页面**: `/admin/users`

**测试项**:
- [ ] 用户列表正确加载
- [ ] 用户卡片显示完整信息
- [ ] 空状态提示显示

**测试步骤**:
1. 访问 `/admin/users`
2. 验证用户列表加载
3. 检查用户卡片信息完整性

---

### 2.7 Settings 页面测试

**页面**: `/admin/settings`

**测试项**:
- [ ] API URL 输入框正常
- [ ] API Key 输入框正常
- [ ] 保存设置功能正常
- [ ] 设置持久化到 localStorage
- [ ] 系统信息正确显示

**测试步骤**:
1. 访问 `/admin/settings`
2. 修改 API URL
3. 修改 API Key
4. 点击保存
5. 刷新页面，验证设置保持
6. 检查 localStorage

---

## 3. 集成测试（Integration Tests）

### 3.1 API 集成测试

**目的**: 验证前端与后端 API 的集成

**前置条件**:
- AgentMem 后端服务器运行
- 数据库已初始化

**测试场景**:

#### 场景 1: 完整的 Agent 生命周期
1. 创建新 Agent
2. 查看 Agent 列表
3. 更新 Agent 状态
4. 与 Agent 聊天
5. 查看 Agent 的 Memories
6. 删除 Agent

#### 场景 2: Memory 管理流程
1. 选择一个 Agent
2. 查看其 Memories
3. 按类型过滤
4. 搜索特定内容
5. 删除一个 Memory

#### 场景 3: 错误处理
1. 断开后端连接
2. 尝试创建 Agent
3. 验证错误消息显示
4. 恢复连接
5. 重试操作

---

### 3.2 状态管理测试

**测试项**:
- [ ] Agent 状态正确同步
- [ ] 页面间状态保持
- [ ] 刷新后状态恢复

---

## 4. 性能测试（Performance Tests）

### 4.1 页面加载性能

**测试工具**: Chrome DevTools Lighthouse

**测试页面**: 所有 admin 页面

**性能指标**:
- First Contentful Paint (FCP) < 1.5s
- Largest Contentful Paint (LCP) < 2.5s
- Time to Interactive (TTI) < 3.5s
- Cumulative Layout Shift (CLS) < 0.1

**测试命令**:
```bash
npm run build
npm run start
# 在 Chrome DevTools 中运行 Lighthouse
```

---

### 4.2 Bundle 大小分析

**命令**:
```bash
npm run build
# 查看 .next/static 目录大小
du -sh .next/static
```

**预期结果**:
- 总 bundle 大小 < 500 KB (gzipped)
- 首次加载 JS < 100 KB

---

## 5. 浏览器兼容性测试

**测试浏览器**:
- [ ] Chrome (最新版本)
- [ ] Firefox (最新版本)
- [ ] Safari (最新版本)
- [ ] Edge (最新版本)

**测试项**:
- [ ] 所有页面正常渲染
- [ ] 所有功能正常工作
- [ ] 样式一致

---

## 6. 响应式设计测试

**测试设备尺寸**:
- [ ] Mobile (375px)
- [ ] Tablet (768px)
- [ ] Desktop (1024px)
- [ ] Large Desktop (1440px)

**测试项**:
- [ ] 布局适配正确
- [ ] 导航菜单适配
- [ ] 表格/卡片适配
- [ ] 对话框适配

---

## 7. 可访问性测试（Accessibility）

**测试工具**: axe DevTools

**测试项**:
- [ ] 键盘导航正常
- [ ] 屏幕阅读器兼容
- [ ] 颜色对比度符合 WCAG 2.1 AA
- [ ] ARIA 标签正确

---

## 8. 测试报告模板

### 测试执行记录

| 测试类别 | 测试项 | 状态 | 备注 |
|---------|--------|------|------|
| 编译测试 | 依赖安装 | ⚠️ 待测试 | 需要 Node.js |
| 编译测试 | TypeScript 检查 | ⚠️ 待测试 | 需要 Node.js |
| 编译测试 | 生产构建 | ⚠️ 待测试 | 需要 Node.js |
| 功能测试 | Dashboard | ⚠️ 待测试 | 需要 dev server |
| 功能测试 | Agents 管理 | ⚠️ 待测试 | 需要 dev server |
| 功能测试 | Chat 界面 | ⚠️ 待测试 | 需要 dev server |
| 功能测试 | Memories 管理 | ⚠️ 待测试 | 需要 dev server |
| 功能测试 | Users 管理 | ⚠️ 待测试 | 需要 dev server |
| 功能测试 | Settings | ⚠️ 待测试 | 需要 dev server |
| 集成测试 | API 集成 | ⚠️ 待测试 | 需要后端 API |
| 性能测试 | 页面加载 | ⚠️ 待测试 | 需要 Lighthouse |

---

## 9. 测试执行清单

### 准备阶段
- [ ] 安装 Node.js 18+
- [ ] 安装依赖包
- [ ] 启动后端 API 服务器
- [ ] 配置环境变量

### 执行阶段
- [ ] 运行编译测试
- [ ] 运行功能测试
- [ ] 运行集成测试
- [ ] 运行性能测试
- [ ] 运行兼容性测试

### 报告阶段
- [ ] 记录测试结果
- [ ] 记录发现的问题
- [ ] 创建 Bug 报告
- [ ] 更新文档

---

## 10. 常见问题和解决方案

### Q1: 编译失败，提示找不到模块
**A**: 检查 `tsconfig.json` 的 paths 配置，确保 `@/*` 映射到 `./src/*`

### Q2: API 调用失败
**A**: 检查 `.env.local` 文件，确保 `NEXT_PUBLIC_API_URL` 正确配置

### Q3: 样式不生效
**A**: 检查 Tailwind CSS 配置，运行 `npm run dev` 重新编译

### Q4: 深色模式不工作
**A**: 检查 `next-themes` 配置，确保 ThemeProvider 正确包裹应用

---

## 附录：测试命令速查表

```bash
# 安装依赖
npm install

# TypeScript 类型检查
npx tsc --noEmit

# 代码格式检查
npm run lint

# 开发服务器
npm run dev

# 生产构建
npm run build

# 启动生产服务器
npm run start

# 运行测试（如果有）
npm test
```

---

**文档维护**: 本文档应在每次测试后更新，记录实际测试结果和发现的问题。

