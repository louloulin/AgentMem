# Chat UI 记忆面板设计方案

**日期**: 2025-11-07  
**参考**: Kimi UI设计  
**目标**: 在Chat界面右侧添加相关记忆展示面板

---

## 🎯 设计目标

参考Kimi的"网页搜索"右侧面板，实现AgentMem的"相关记忆"展示：

### Kimi设计分析（从截图）

```
┌─────────────────────────────────┬─────────────────────────┐
│      主聊天区域                  │    右侧信息面板         │
│                                 │                         │
│  [用户消息]                      │  📊 网页搜索 9         │
│  [AI回复]                        │                         │
│  [用户消息]                      │  🔗 结果1              │
│  [AI回复...]                     │  🔗 结果2              │
│                                 │  🔗 结果3              │
│  [输入框]                        │  ...                   │
│                                 │                         │
└─────────────────────────────────┴─────────────────────────┘
```

**关键特点**:
1. ✅ 右侧面板可折叠
2. ✅ 显示搜索结果数量
3. ✅ 卡片式展示每个结果
4. ✅ 点击可展开详情
5. ✅ 自动滚动同步

---

## 🎨 AgentMem 记忆面板设计

### 布局结构

```
┌──────────────────────────────────────┬────────────────────────┐
│       Chat 主对话区                   │   相关记忆面板         │
│                                      │                        │
│  👤 用户: 搜索fluvio资料              │  🧠 相关记忆 5         │
│                                      │  ─────────────────── │
│  🤖 AI: [流式回复中...]              │  📝 记忆1              │
│       Fluvio是一个...                │  标题: Fluvio概述      │
│                                      │  类型: Semantic        │
│  [Previous messages...]              │  相关度: 0.95          │
│                                      │  时间: 2小时前         │
│                                      │                        │
│  ┌────────────────────────────────┐  │  📝 记忆2              │
│  │ 尽管问...            [发送] │  │  标题: Rust数据流      │
│  └────────────────────────────────┘  │  ...                  │
│                                      │                        │
└──────────────────────────────────────┴────────────────────────┘
     70%                                      30%
```

---

## 🔧 技术实现方案

### 1. 数据结构

```typescript
// 记忆展示接口
interface MemoryDisplayItem {
  id: string;
  title: string;              // 记忆标题（从内容提取前50字）
  content: string;            // 完整内容
  memory_type: string;        // Semantic, Episodic等
  relevance_score: number;    // 相关度分数 (0-1)
  created_at: string;         // 创建时间
  scope: string;              // agent/user/session
  metadata?: {
    source?: string;
    tags?: string[];
  };
}

// 记忆搜索结果
interface MemorySearchResult {
  query: string;              // 当前用户输入
  memories: MemoryDisplayItem[];
  total_count: number;
  search_time_ms: number;
}
```

---

### 2. 组件结构

```tsx
// 新增组件
<div className="flex h-screen">
  {/* 主聊天区域 */}
  <div className="flex-1 flex flex-col">
    {/* Agent选择器 */}
    <div className="border-b p-4">...</div>
    
    {/* 消息列表 */}
    <div className="flex-1 overflow-y-auto">...</div>
    
    {/* 输入框 */}
    <div className="border-t p-4">...</div>
  </div>
  
  {/* 右侧记忆面板 */}
  <MemoryPanel
    visible={showMemoryPanel}
    memories={relevantMemories}
    loading={searchingMemories}
    onToggle={() => setShowMemoryPanel(!showMemoryPanel)}
  />
</div>
```

---

### 3. 核心Hook: `useMemorySearch`

```typescript
/**
 * 自动搜索相关记忆的Hook
 * 当用户输入时，自动搜索相关记忆
 */
function useMemorySearch(
  agentId: string | null,
  userId: string,
  enabled: boolean = true
) {
  const [memories, setMemories] = useState<MemoryDisplayItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [lastQuery, setLastQuery] = useState('');
  
  const searchMemories = useCallback(async (query: string) => {
    if (!agentId || !query.trim() || query === lastQuery) return;
    
    setLoading(true);
    setLastQuery(query);
    
    try {
      // 调用记忆搜索API
      const response = await fetch(
        `${API_BASE_URL}/api/v1/memories/search?` + 
        `agent_id=${agentId}&user_id=${userId}&query=${encodeURIComponent(query)}&limit=10`,
        {
          headers: {
            'Authorization': `Bearer ${token}`,
          },
        }
      );
      
      if (!response.ok) throw new Error('Search failed');
      
      const data = await response.json();
      
      // 转换为展示格式
      const displayMemories: MemoryDisplayItem[] = data.data.map((mem: any) => ({
        id: mem.id,
        title: extractTitle(mem.content),
        content: mem.content,
        memory_type: mem.memory_type || 'Unknown',
        relevance_score: mem.score || 0,
        created_at: mem.created_at,
        scope: mem.scope || 'unknown',
        metadata: mem.metadata,
      }));
      
      setMemories(displayMemories);
    } catch (err) {
      console.error('Memory search failed:', err);
      setMemories([]);
    } finally {
      setLoading(false);
    }
  }, [agentId, userId, lastQuery]);
  
  return { memories, loading, searchMemories };
}

// 提取标题（取前50个字符）
function extractTitle(content: string): string {
  return content.length > 50 
    ? content.substring(0, 50) + '...' 
    : content;
}
```

---

### 4. MemoryPanel 组件

```tsx
interface MemoryPanelProps {
  visible: boolean;
  memories: MemoryDisplayItem[];
  loading: boolean;
  onToggle: () => void;
}

function MemoryPanel({ visible, memories, loading, onToggle }: MemoryPanelProps) {
  if (!visible) {
    // 折叠状态：只显示切换按钮
    return (
      <div className="fixed right-0 top-1/2 -translate-y-1/2 z-50">
        <Button
          onClick={onToggle}
          className="rounded-l-lg rounded-r-none"
          variant="outline"
        >
          🧠 相关记忆 {memories.length > 0 && `(${memories.length})`}
        </Button>
      </div>
    );
  }
  
  return (
    <div className="w-80 border-l bg-white flex flex-col">
      {/* 头部 */}
      <div className="border-b p-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Brain className="w-5 h-5" />
          <h3 className="font-semibold">相关记忆</h3>
          {memories.length > 0 && (
            <Badge variant="secondary">{memories.length}</Badge>
          )}
        </div>
        <Button
          onClick={onToggle}
          size="sm"
          variant="ghost"
        >
          ✕
        </Button>
      </div>
      
      {/* 记忆列表 */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {loading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-6 h-6 animate-spin" />
            <span className="ml-2">搜索中...</span>
          </div>
        ) : memories.length === 0 ? (
          <div className="text-center text-gray-500 py-8">
            暂无相关记忆
          </div>
        ) : (
          memories.map((memory) => (
            <MemoryCard key={memory.id} memory={memory} />
          ))
        )}
      </div>
    </div>
  );
}
```

---

### 5. MemoryCard 组件

```tsx
function MemoryCard({ memory }: { memory: MemoryDisplayItem }) {
  const [expanded, setExpanded] = useState(false);
  
  // 计算相对时间
  const relativeTime = formatRelativeTime(memory.created_at);
  
  // 记忆类型对应的图标
  const typeIcon = {
    'Semantic': '📚',
    'Episodic': '📝',
    'Procedural': '⚙️',
    'Working': '💭',
    'Core': '⭐',
  }[memory.memory_type] || '📄';
  
  return (
    <Card className="p-3 hover:shadow-md transition-shadow cursor-pointer">
      <div onClick={() => setExpanded(!expanded)}>
        {/* 头部 */}
        <div className="flex items-start justify-between mb-2">
          <div className="flex items-center gap-2 flex-1">
            <span className="text-lg">{typeIcon}</span>
            <div className="flex-1 min-w-0">
              <p className="font-medium text-sm truncate">
                {memory.title}
              </p>
              <div className="flex items-center gap-2 mt-1 text-xs text-gray-500">
                <Badge variant="outline" className="text-xs">
                  {memory.memory_type}
                </Badge>
                <span>{relativeTime}</span>
              </div>
            </div>
          </div>
          
          {/* 相关度分数 */}
          <div className="ml-2">
            <div className="text-xs font-medium text-blue-600">
              {(memory.relevance_score * 100).toFixed(0)}%
            </div>
          </div>
        </div>
        
        {/* 内容预览 */}
        {expanded && (
          <div className="mt-3 pt-3 border-t">
            <p className="text-sm text-gray-700 whitespace-pre-wrap">
              {memory.content}
            </p>
            
            {/* 元数据 */}
            {memory.metadata && (
              <div className="mt-3 pt-3 border-t">
                <div className="text-xs text-gray-500 space-y-1">
                  {memory.metadata.source && (
                    <div>来源: {memory.metadata.source}</div>
                  )}
                  {memory.metadata.tags && (
                    <div className="flex gap-1 flex-wrap mt-1">
                      {memory.metadata.tags.map((tag, i) => (
                        <Badge key={i} variant="secondary" className="text-xs">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </Card>
  );
}

// 格式化相对时间
function formatRelativeTime(timestamp: string): string {
  const now = Date.now();
  const then = new Date(timestamp).getTime();
  const diffMs = now - then;
  
  const seconds = Math.floor(diffMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  
  if (days > 0) return `${days}天前`;
  if (hours > 0) return `${hours}小时前`;
  if (minutes > 0) return `${minutes}分钟前`;
  return '刚刚';
}
```

---

## 🎬 交互流程

### 1. 自动搜索

```
用户输入 "搜索fluvio资料"
    ↓
  [防抖 500ms]
    ↓
  调用 searchMemories(query)
    ↓
  API: GET /api/v1/memories/search?query=fluvio
    ↓
  更新 memories 状态
    ↓
  右侧面板显示结果
```

### 2. 手动触发

```
用户点击消息旁的 "🔍" 按钮
    ↓
  使用该消息内容搜索
    ↓
  展开右侧面板
    ↓
  显示相关记忆
```

---

## 📊 API接口需求

### 记忆搜索API

**端点**: `GET /api/v1/memories/search`

**参数**:
```typescript
{
  query: string;          // 搜索关键词
  agent_id?: string;      // Agent ID (可选)
  user_id?: string;       // User ID (可选)
  limit?: number;         // 结果数量限制 (默认10)
  scope?: string;         // 范围过滤: agent/user/session
  memory_type?: string;   // 类型过滤
}
```

**响应**:
```typescript
{
  success: true,
  data: [
    {
      id: string;
      content: string;
      memory_type: string;
      score: number;         // 相关度分数 (0-1)
      created_at: string;
      scope: string;
      metadata: object;
    }
  ],
  total: number;
  search_time_ms: number;
}
```

---

## 🎨 UI样式参考

### 颜色方案

```css
/* 主题色 */
--memory-primary: #3b82f6;      /* 蓝色 - 主要强调 */
--memory-secondary: #8b5cf6;    /* 紫色 - 次要强调 */
--memory-border: #e5e7eb;       /* 边框 */
--memory-bg: #f9fafb;           /* 背景 */
--memory-hover: #f3f4f6;        /* 悬停 */

/* 记忆类型颜色 */
--semantic-color: #10b981;      /* 绿色 - 语义记忆 */
--episodic-color: #f59e0b;      /* 橙色 - 情景记忆 */
--procedural-color: #8b5cf6;    /* 紫色 - 程序记忆 */
--working-color: #6b7280;       /* 灰色 - 工作记忆 */
```

### Tailwind类

```tsx
// 面板容器
className="w-80 border-l bg-white flex flex-col h-full"

// 记忆卡片
className="p-3 rounded-lg border hover:shadow-md transition-all cursor-pointer"

// 相关度分数
className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"

// 记忆类型Badge
className="inline-flex items-center px-2 py-1 rounded text-xs font-medium"
```

---

## 🚀 实施步骤

### Phase 1: 基础布局（30分钟）

1. ✅ 修改chat page布局为flex横向
2. ✅ 添加MemoryPanel组件
3. ✅ 实现折叠/展开功能
4. ✅ 基础样式

### Phase 2: 数据集成（1小时）

1. ✅ 实现useMemorySearch Hook
2. ✅ 集成记忆搜索API
3. ✅ 处理加载和错误状态
4. ✅ 数据转换和格式化

### Phase 3: 交互优化（1小时）

1. ✅ 自动搜索（防抖）
2. ✅ 手动搜索按钮
3. ✅ 卡片展开/折叠
4. ✅ 点击记忆显示详情

### Phase 4: 视觉优化（30分钟）

1. ✅ 记忆类型图标和颜色
2. ✅ 相关度可视化
3. ✅ 时间显示优化
4. ✅ 动画和过渡效果

---

## 📝 代码修改清单

### 需要修改的文件

1. `app/admin/chat/page.tsx` - 主页面
   - 添加记忆搜索状态
   - 集成MemoryPanel
   - 修改布局结构

2. 新增 `components/MemoryPanel.tsx`
   - MemoryPanel组件
   - MemoryCard组件
   - useMemorySearch Hook

3. 新增 `lib/memory-search.ts`
   - 记忆搜索API封装
   - 数据转换工具

4. 更新 `lib/api-client.ts`
   - 添加searchMemories方法

---

## 🎯 预期效果

**修改前**:
```
[聊天界面 - 全屏]
```

**修改后**:
```
[聊天界面 70%] | [相关记忆 30%]
```

**用户体验**:
- ✅ 实时看到相关记忆
- ✅ 了解AI回复的来源
- ✅ 验证记忆准确性
- ✅ 发现相关知识

---

## ✅ 成功标准

1. 右侧面板可正常显示
2. 记忆搜索API正常调用
3. 记忆卡片正确展示
4. 交互流畅无卡顿
5. 样式美观符合Kimi风格

---

**状态**: 📝 设计完成，待实施  
**预计时间**: 3小时  
**难度**: 中等

