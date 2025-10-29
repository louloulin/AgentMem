# AgentMem 官方网站

🧠 **AgentMem** 智能记忆管理平台的官方网站，展示项目的核心特性、技术优势和商业价值。

## 🚀 项目特色

- **现代化设计**: 使用 Next.js 15 + TypeScript + Tailwind CSS
- **组件化架构**: 基于 shadcn/ui 组件库
- **响应式布局**: 完美适配桌面端和移动端
- **SEO 优化**: 完整的元数据和结构化数据
- **高性能**: 使用 Turbopack 构建，极速开发体验

## 📋 页面结构

### 🏠 首页 (`/`)
- **英雄区域**: 产品介绍和核心价值主张
- **核心特性**: 6大核心功能模块展示
- **技术架构**: 分层架构和模块化设计
- **性能基准**: 与竞品的性能对比
- **客户案例**: 成功案例和用户反馈

### 📚 文档页面 (`/docs`)
- **快速开始**: 3步快速上手指南
- **安装指南**: 多种安装方式（Cargo、Mem0兼容、Docker）
- **API 参考**: 完整的 API 文档和示例
- **架构设计**: 技术架构深度解析
- **部署指南**: 生产环境部署最佳实践

### 🎮 演示页面 (`/demo`)
- **交互式演示**: 3个核心功能的在线演示
- **代码示例**: 可复制的代码片段
- **实时输出**: 模拟真实的运行结果
- **性能对比**: 与 Mem0 的性能对比展示

### 👥 关于页面 (`/about`)
- **公司介绍**: 使命、愿景、价值观
- **核心团队**: 团队成员介绍和背景
- **商业化方向**: 4大商业化策略
- **发展历程**: 项目里程碑和成就
- **联系方式**: 多渠道联系信息

## 🚀 技术栈

- **框架**: Next.js 14 (App Router)
- **语言**: TypeScript
- **样式**: Tailwind CSS
- **组件库**: shadcn/ui
- **图标**: Lucide React
- **包管理器**: Bun
- **部署**: Vercel (推荐)

## 📋 功能特性

### 页面结构
- **首页** (`/`) - 产品介绍、核心特性、技术优势
- **文档页面** (`/docs`) - API文档、快速开始指南
- **演示页面** (`/demo`) - 在线演示和交互式示例
- **关于页面** (`/about`) - 团队介绍、商业化方向、未来规划

### 设计特色
- 🎨 现代化深色主题设计
- 🌈 紫色渐变配色方案
- 📱 完全响应式布局
- ⚡ 流畅的动画和交互效果
- 🧠 突出 AI 和记忆管理主题

### 核心组件
- 导航栏和页脚
- 特性展示卡片
- 代码演示区域
- 团队介绍模块
- 商业化方向展示
- 联系表单和CTA按钮

## 🛠️ 开发指南

### 环境要求
- Node.js 18+ 或 Bun 1.0+
- 现代浏览器支持

### 安装依赖

```bash
# 使用 bun (推荐)
bun install

# 或使用 npm
npm install

# 或使用 yarn
yarn install
```

### 开发服务器

```bash
# 启动开发服务器
bun dev

# 或
npm run dev
```

访问 [http://localhost:3000](http://localhost:3000) 查看网站。

### 构建和部署

```bash
# 构建生产版本
bun run build

# 启动生产服务器
bun start

# 或使用 npm
npm run build
npm start
```

## 📁 项目结构

```
agentmem-website/
├── src/
│   ├── app/                    # App Router 页面
│   │   ├── page.tsx           # 首页
│   │   ├── docs/              # 文档页面
│   │   ├── demo/              # 演示页面
│   │   ├── about/             # 关于页面
│   │   ├── layout.tsx         # 根布局
│   │   └── globals.css        # 全局样式
│   ├── components/            # React 组件
│   │   └── ui/               # shadcn/ui 组件
│   └── lib/                  # 工具函数
│       └── utils.ts          # 通用工具
├── public/                   # 静态资源
├── components.json           # shadcn/ui 配置
├── tailwind.config.js        # Tailwind 配置
├── next.config.ts            # Next.js 配置
└── package.json              # 项目配置
```

## 🎨 设计系统

### 颜色方案
- **主色调**: 紫色 (`purple-600`, `purple-400`)
- **辅助色**: 蓝色、绿色、黄色、红色
- **背景**: 深色渐变 (`slate-900` → `purple-900`)
- **文本**: 白色和灰色层次

### 组件规范
- 使用 shadcn/ui 组件库
- 统一的卡片样式和间距
- 一致的按钮和交互状态
- 响应式网格布局

## 📝 内容管理

### 页面内容
所有页面内容都直接在组件中管理，便于维护和更新：

- **产品特性**: 在首页组件中定义
- **API 文档**: 在文档页面中维护
- **演示代码**: 在演示页面中配置
- **团队信息**: 在关于页面中更新

### 添加新页面
1. 在 `src/app/` 下创建新目录
2. 添加 `page.tsx` 文件
3. 更新导航菜单链接
4. 添加相应的元数据

## 🚀 部署指南

### Vercel 部署 (推荐)
1. 将代码推送到 GitHub
2. 在 Vercel 中导入项目
3. 自动部署和 CI/CD

### 其他平台
- **Netlify**: 支持 Next.js
- **AWS Amplify**: 全栈部署
- **Docker**: 容器化部署

## 🔧 自定义配置

### 修改主题色彩
编辑 `src/app/globals.css` 中的 CSS 变量：

```css
:root {
  --primary: 262.1 83.3% 57.8%;  /* purple-600 */
  --primary-foreground: 210 20% 98%;
  /* 其他颜色变量... */
}
```

### 添加新组件
使用 shadcn/ui CLI 添加组件：

```bash
bunx shadcn@latest add [component-name]
```

### 修改布局
编辑 `src/app/layout.tsx` 来修改全局布局和元数据。

## 📊 性能优化

- ✅ Next.js 14 App Router
- ✅ Turbopack 构建优化
- ✅ 图片优化和懒加载
- ✅ 代码分割和动态导入
- ✅ SEO 优化和元数据
- ✅ 响应式图片和字体

## 🤝 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

MIT License - 查看 [LICENSE](LICENSE) 文件了解详情。

## 📞 联系我们

- **官网**: https://agentmem.com
- **GitHub**: https://github.com/agentmem/agentmem
- **邮箱**: hello@agentmem.com

---

**AgentMem** - 让 AI 代理拥有真正的记忆能力 🧠✨