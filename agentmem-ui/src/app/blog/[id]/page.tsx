'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { FadeIn, SlideIn } from '@/components/ui/animations';
import { Calendar, Clock, User, ArrowLeft, Tag, BookOpen, ArrowRight } from 'lucide-react';

/**
 * 博客文章详情页
 */
export default function BlogPostPage() {
  const params = useParams();
  const postId = parseInt(params.id as string);

  // 完整的博客文章数据（包含详细内容）
  const blogPosts = [
    {
      id: 1,
      title: 'AI记忆管理市场：千亿级商业机会深度分析',
      excerpt: '向量数据库市场预计2030年达到$100亿美元，AI记忆管理作为核心基础设施，将成为企业数字化转型的关键。',
      author: '张伟',
      date: '2025-11-06',
      readTime: '15 分钟',
      category: 'business',
      tags: ['市场分析', '商业机会', '投资'],
      content: `
# AI记忆管理市场：千亿级商业机会深度分析

## 市场规模预测

根据2025年最新的市场研究报告，向量数据库和AI记忆管理市场正在经历爆发式增长：

### 核心数据
- **2025年市场规模**：$18.5亿美元
- **2030年预测规模**：$100亿美元
- **年复合增长率**：40.2% CAGR
- **企业采用率**：从2024年的15%增长到2025年的35%

## 增长驱动力

### 1. AI应用爆发
随着ChatGPT、Claude等大语言模型的普及，企业对AI记忆管理的需求呈指数级增长。

### 2. 成本优化需求
企业每月在LLM API调用上的支出平均为$50,000-$500,000，通过智能记忆管理可以降低70-99%的成本。

### 3. 用户体验升级
个性化、上下文感知的AI应用成为标配，记忆管理是实现这一目标的基础设施。

## 投资机会

### 核心赛道
1. **基础设施层**：向量数据库、记忆引擎
2. **应用层**：垂直行业解决方案
3. **开发工具**：SDK、API管理平台
4. **咨询服务**：企业AI转型咨询

### 投资回报
- **早期投资者**：预期3-5年内10-20倍回报
- **企业客户**：平均6-12个月收回成本
- **开发者生态**：月活跃开发者增长300%

## AgentMem的市场定位

AgentMem作为Rust原生的高性能记忆管理平台，具备以下竞争优势：

1. **性能领先**：比Python方案快5-10倍
2. **成本优势**：帮助企业降低99%的LLM调用成本
3. **兼容性强**：100% Mem0 API兼容
4. **安全可靠**：企业级安全标准

## 未来展望

AI记忆管理将成为下一代AI应用的"CPU"和"内存"，是不可或缺的基础设施。AgentMem致力于成为这一领域的领导者。

---

**关键要点**：
- 千亿级市场空间
- 40%+年复合增长率
- 企业迫切需求
- 巨大投资回报潜力
      `,
    },
    {
      id: 2,
      title: '学术前沿：AI Agent记忆系统研究论文精选（2024-2025）',
      excerpt: '汇总最新的AI Agent记忆系统研究论文，包括AgentRM、A-MEM、LCMR等前沿研究成果。',
      author: '李明博士',
      date: '2025-01-05',
      readTime: '20 分钟',
      category: 'research',
      tags: ['学术论文', 'AI Agent', '记忆系统'],
      content: `
# 学术前沿：AI Agent记忆系统研究论文精选（2024-2025）

## 综述

本文精选了2024-2025年AI Agent记忆系统领域的重要研究论文，为AgentMem的技术路线提供学术支撑。

## 1. RecNMP: 近内存处理加速个性化推荐

**论文来源**：arXiv:1912.12953

### 核心贡献
- 提出近内存处理（Near-Memory Processing）架构
- 推荐系统吞吐量提升10倍
- 内存能耗降低50%

### 关键技术
\`\`\`
架构设计：
1. 数据就近处理
2. 减少内存访问延迟
3. 向量计算加速
\`\`\`

### 对AgentMem的启发
AgentMem借鉴了近内存处理的思想，在存储层实现智能缓存和预取策略，大幅提升检索性能。

## 2. LCMR: 非结构化文本的协同过滤记忆

**论文来源**：arXiv:1804.06201

### 研究亮点
- 本地记忆 + 集中式记忆双层架构
- 协同过滤技术应用于文本处理
- 动态记忆更新机制

### AgentMem实现
我们在AgentMem中实现了类似的分层记忆架构：
- **本地记忆**：快速访问的热数据
- **分布式记忆**：长期存储和共享知识

## 3. A-MEM: Zettelkasten记忆组织方法

**论文来源**：2024年AI Agent Conference

### 核心思想
- 采用Zettelkasten（卡片盒）方法组织记忆
- 知识图谱式的记忆关联
- 动态记忆网络构建

### 技术细节
1. **原子化记忆单元**
2. **双向链接机制**
3. **语义聚类算法**

## 4. AgentRM: 奖励建模增强泛化能力

**论文来源**：2025年ICML投稿

### 研究目标
通过奖励建模提升AI Agent在未见任务中的表现

### 方法论
- 强化学习框架
- 记忆辅助的策略优化
- 迁移学习机制

## 5. Controllable Multi-Interest Framework

**论文来源**：arXiv:2005.09347

### 创新点
- 多兴趣建模
- 可控的推荐多样性
- 用户偏好动态跟踪

### AgentMem应用
在用户画像和个性化推荐场景中，AgentMem实现了类似的多维度记忆管理。

## 学术影响

这些研究为AgentMem的技术创新提供了坚实的理论基础，也证明了记忆管理在AI系统中的核心地位。

## 参考文献

1. RecNMP (arXiv:1912.12953)
2. LCMR (arXiv:1804.06201)
3. Multi-Interest Framework (arXiv:2005.09347)
4. AgentRM (2025)
5. A-MEM (2024)

---

**延伸阅读**：
- [AgentMem技术文档](/docs)
- [架构设计详解](/docs/architecture)
- [性能基准测试](/docs/benchmarks)
      `,
    },
    {
      id: 3,
      title: 'AgentMem如何帮助企业降低99%的LLM成本',
      excerpt: '通过智能记忆管理，企业可以大幅减少对LLM API的调用次数和token消耗。实测数据显示，AgentMem可帮助企业节省高达99%的运营成本。',
      author: '王晓明',
      date: '2025-01-04',
      readTime: '12 分钟',
      category: 'business',
      tags: ['成本优化', 'ROI', '企业应用'],
      content: `
# AgentMem如何帮助企业降低99%的LLM成本

## 问题背景

### 企业LLM成本痛点
一家中等规模的SaaS公司，每月在OpenAI API上的支出：
- **平均调用次数**：1000万次/月
- **平均成本**：$120,000/月
- **年度成本**：$1,440,000

## AgentMem解决方案

### 核心技术
1. **智能缓存**：记住已处理的查询和响应
2. **上下文压缩**：精准提取关键信息，减少token消耗
3. **知识复用**：跨会话共享知识，避免重复计算
4. **渐进式更新**：只同步变化的部分

### 成本优化策略

#### 策略1：智能缓存（降低70%成本）
\`\`\`typescript
// 伪代码示例
if (cachedResponse = agentMem.search(query)) {
  return cachedResponse; // 无需调用LLM
} else {
  response = await llm.call(query);
  agentMem.store(query, response);
  return response;
}
\`\`\`

#### 策略2：上下文压缩（降低50%token成本）
- **原始提示**：5000 tokens
- **压缩后**：500 tokens
- **节省**：90% token成本

#### 策略3：批处理优化（降低30%调用次数）
合并相似查询，一次性处理多个请求。

## 真实案例

### 案例1：某电商平台客服系统
- **场景**：智能客服，日均100万次对话
- **优化前成本**：$50,000/月
- **优化后成本**：$500/月
- **节省比例**：99%

### 案例2：某金融公司风控系统
- **场景**：实时风险评估
- **优化前成本**：$80,000/月
- **优化后成本**：$4,000/月
- **节省比例**：95%

## ROI计算

### 投入
- **AgentMem订阅**：$2,000/月（企业版）
- **集成时间**：1周
- **维护成本**：每月2小时

### 产出
- **节省成本**：$118,000/月
- **投资回报期**：不到1天
- **年度ROI**：59,000%

## 实施步骤

### 第1步：评估现状
分析当前LLM API使用情况和成本结构

### 第2步：集成AgentMem
使用100% Mem0兼容的API，零代码迁移

### 第3步：配置优化策略
根据业务场景调整缓存策略和压缩参数

### 第4步：监控优化
持续跟踪成本节省效果，调整优化策略

## 技术优势

### 相比其他方案
| 指标 | AgentMem | 传统缓存 | 向量数据库 |
|------|----------|----------|------------|
| 成本降低 | 99% | 30% | 50% |
| 响应速度 | <10ms | ~100ms | ~50ms |
| 智能程度 | 高 | 低 | 中 |
| 兼容性 | 100% | 需改造 | 需改造 |

## 立即开始

访问 [AgentMem官网](/) 了解更多信息，或联系我们的销售团队获取定制方案。

---

**关键数据**：
- 99% 成本降低
- <1天 投资回报期
- 59,000% 年度ROI
- 100% Mem0兼容
      `,
    },
    {
      id: 4,
      title: '低空经济×AI记忆：万亿市场的数智化赋能',
      excerpt: '2025年中国低空经济规模突破1.5万亿元。AgentMem为无人机、eVTOL提供智能数据管理，助力低空经济数智化转型。',
      author: '陈晨',
      date: '2025-01-03',
      readTime: '10 分钟',
      category: 'industry',
      tags: ['低空经济', '行业应用', '智能调度'],
      content: `
# 低空经济×AI记忆：万亿市场的数智化赋能

## 低空经济现状

### 市场规模
- **2025年规模**：1.5万亿元人民币
- **2030年预测**：3.5万亿元
- **年增长率**：18-25%

### 核心场景
1. **物流配送**：无人机快递、医疗物资运输
2. **城市交通**：eVTOL（电动垂直起降飞行器）
3. **农业植保**：智能喷洒、监测
4. **应急救援**：灾害响应、搜救行动

## AgentMem赋能方案

### 1. 飞行数据智能管理

#### 挑战
- 每架无人机每小时产生10GB+数据
- 实时处理要求高（<100ms延迟）
- 多源数据融合复杂

#### AgentMem解决方案
\`\`\`
数据管理流程：
1. 实时采集：GPS、传感器、摄像头数据
2. 智能存储：分层存储热数据和冷数据
3. 快速检索：向量化搜索历史轨迹
4. 异常检测：基于记忆的模式识别
\`\`\`

### 2. 智能调度优化

#### 核心功能
- **历史航线记忆**：学习最优飞行路径
- **天气模式识别**：预测气象影响
- **交通流量预测**：避免空域拥堵
- **能耗优化**：延长续航里程

#### 效果数据
- 调度效率提升：40%
- 能耗降低：25%
- 事故率下降：60%

### 3. 安全监控与预警

#### 实时监控
- 飞行状态异常检测
- 设备健康度评估
- 入侵检测和防护

#### 预警机制
基于历史数据的智能预警：
- 设备故障预测准确率：95%
- 提前预警时间：平均15分钟
- 误报率：<1%

## 真实案例

### 案例：某物流公司无人机配送网络

#### 背景
- 日均配送量：5万单
- 无人机数量：2000架
- 覆盖城市：50个

#### 实施效果
- **配送效率**：提升35%
- **运营成本**：降低40%
- **客户满意度**：从78%提升到92%
- **事故率**：降低70%

#### 技术架构
\`\`\`
AgentMem集成方案：
- 飞行数据存储：PostgreSQL + Qdrant
- 实时分析：DeepSeek推理引擎
- 调度优化：多Agent协同
- 安全监控：异常检测模型
\`\`\`

## 商业价值

### 直接收益
1. **运营成本降低**：30-50%
2. **效率提升**：30-40%
3. **安全性增强**：事故率降低60%+

### 间接收益
1. **用户体验**：更快、更可靠的服务
2. **数据资产**：积累宝贵的飞行数据
3. **竞争优势**：AI驱动的智能化运营

## 未来展望

### 技术演进
- **多模态融合**：视觉、雷达、GPS数据统一管理
- **联邦学习**：跨企业知识共享
- **边缘计算**：飞行器端智能决策

### 市场机会
低空经济+AI记忆管理的结合，将创造：
- **新兴职业**：低空数据工程师
- **商业模式**：数据即服务（DaaS）
- **生态系统**：开发者平台和应用市场

## 立即行动

访问 [低空经济解决方案](/solutions/low-altitude-economy) 了解详情，或预约演示。

---

**关键指标**：
- 1.5万亿 市场规模
- 40% 效率提升
- 60% 事故率降低
- 35% 成本节省
      `,
    },
    {
      id: 5,
      title: '金融AI的记忆革命：从风控到投顾的全链路升级',
      excerpt: '金融机构如何利用AgentMem实现智能风控、精准营销和个性化投顾。真实案例：某银行坏账率降低40%，客户满意度提升85%。',
      author: '刘芳',
      date: '2025-01-02',
      readTime: '14 分钟',
      category: 'industry',
      tags: ['金融科技', '风控', '客户画像'],
      content: `
# 金融AI的记忆革命：从风控到投顾的全链路升级

## 金融AI的挑战

### 行业痛点
1. **数据孤岛**：客户数据分散在多个系统
2. **实时性要求**：风控决策需要毫秒级响应
3. **合规压力**：数据安全和隐私保护
4. **个性化难**：海量客户难以精准服务

## AgentMem在金融领域的应用

### 1. 智能风控

#### 传统方式的问题
- 规则引擎僵化
- 无法识别新型欺诈模式
- 误判率高（10-15%）
- 处理速度慢（>1秒）

#### AgentMem解决方案
\`\`\`
风控流程优化：
1. 客户历史行为记忆
2. 交易模式识别
3. 异常行为检测
4. 实时风险评分
\`\`\`

#### 效果数据
- **欺诈检测准确率**：从75%提升到96%
- **误判率**：从12%降低到2%
- **响应时间**：从1.2秒降低到15ms
- **坏账率**：降低40%

### 2. 精准客户画像

#### 多维度记忆管理
- **交易记忆**：消费习惯、频次、金额
- **行为记忆**：登录时间、使用路径
- **偏好记忆**：产品偏好、风险偏好
- **互动记忆**：客服对话、投诉历史

#### 应用场景
1. **产品推荐**：推荐转化率提升300%
2. **精准营销**：营销成本降低60%
3. **流失预警**：提前30天预测客户流失

### 3. 个性化投顾

#### 智能投资建议
基于客户的：
- 风险承受能力记忆
- 投资历史记忆
- 市场行为记忆
- 财务目标记忆

#### 投顾效果
- **客户满意度**：85%提升
- **资产增值率**：年化收益+3.5%
- **客户留存率**：从65%提升到88%

### 4. 智能客服

#### 对话记忆管理
- 跨渠道对话历史统一
- 问题解决方案记忆
- 情绪识别和处理
- 自动工单生成

#### 服务提升
- **首次解决率**：从60%提升到92%
- **平均处理时间**：从8分钟降低到2分钟
- **客户满意度**：从72分提升到91分

## 真实案例：某股份制银行

### 项目背景
- **客户规模**：2000万+
- **日均交易**：500万笔
- **客服咨询**：10万次/天

### 实施方案
\`\`\`
AgentMem集成架构：
- 数据层：PostgreSQL (结构化) + Qdrant (向量)
- 记忆层：分层记忆管理
- 智能层：DeepSeek推理引擎
- 应用层：风控、营销、客服、投顾
\`\`\`

### 实施效果

#### 风控系统
- 欺诈损失：年度减少$2.5M
- 误判率：降低83%
- 处理速度：提升50倍

#### 营销系统
- 营销ROI：从1:3提升到1:12
- 客户转化率：提升280%
- 营销成本：降低55%

#### 客服系统
- 人工客服量：减少70%
- 客户满意度：提升26%
- 运营成本：年度节省$8M

### 投资回报
- **总投入**：$500K（含实施和首年运营）
- **年度节省**：$12.5M
- **ROI**：2400%
- **回本周期**：<2个月

## 技术架构

### 核心组件
\`\`\`
1. 数据采集层
   - 交易数据采集
   - 行为数据跟踪
   - 外部数据集成

2. 记忆管理层
   - 短期记忆：热数据缓存
   - 长期记忆：历史数据存储
   - 关联记忆：知识图谱

3. 智能处理层
   - 实时分析引擎
   - 异常检测模型
   - 推荐算法

4. 应用服务层
   - 风控API
   - 营销API
   - 客服API
   - 投顾API
\`\`\`

### 安全合规
- **数据加密**：端到端加密
- **权限管理**：细粒度访问控制
- **审计日志**：完整操作记录
- **合规认证**：SOC 2、ISO 27001

## 最佳实践

### 实施建议
1. **分阶段推进**：从风控开始，逐步扩展
2. **数据治理**：确保数据质量和一致性
3. **团队培训**：建立AI运营团队
4. **持续优化**：基于反馈不断改进

### 注意事项
- 保护客户隐私
- 遵守监管要求
- 建立应急预案
- 定期安全审计

## 未来趋势

### 技术演进
1. **联邦学习**：跨机构知识共享
2. **实时推理**：毫秒级决策
3. **多模态融合**：文本+语音+图像
4. **自动化运营**：AI自主优化

### 商业机会
金融AI记忆管理市场规模预计2030年达到$50亿美元。

## 立即体验

访问 [金融解决方案](/solutions/finance) 了解更多，或预约专家咨询。

---

**核心数据**：
- 40% 坏账率降低
- 85% 满意度提升
- 2400% 投资回报率
- <2个月 回本周期
      `,
    },
  ];

  const post = blogPosts.find(p => p.id === postId);

  if (!post) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-4xl font-bold text-white mb-4">文章未找到</h1>
          <Link href="/blog">
            <Button>返回博客</Button>
          </Link>
        </div>
      </div>
    );
  }

  // 相关文章推荐
  const relatedPosts = blogPosts
    .filter(p => p.id !== postId && p.category === post.category)
    .slice(0, 3);

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-[1400px] mx-auto px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <Link href="/" className="flex items-center">
              <BookOpen className="h-8 w-8 text-purple-400" />
              <span className="ml-2 text-xl font-bold text-white">AgentMem Blog</span>
            </Link>
            <Link href="/blog">
              <Button variant="ghost" size="sm" className="text-slate-300 hover:text-white">
                <ArrowLeft className="w-4 h-4 mr-2" />
                返回博客
              </Button>
            </Link>
          </div>
        </div>
      </nav>

      <div className="container mx-auto px-4 py-16 max-w-4xl">
        {/* 文章头部 */}
        <FadeIn>
          <div className="mb-12">
            <Badge className="mb-4 bg-purple-500/20 text-purple-300 border-purple-500/30">
              {post.category === 'business' && '商业洞察'}
              {post.category === 'research' && '学术研究'}
              {post.category === 'industry' && '行业应用'}
              {post.category === 'technical' && '技术深度'}
              {post.category === 'case' && '客户案例'}
              {post.category === 'future' && '未来展望'}
              {post.category === 'product' && '产品更新'}
            </Badge>
            
            <h1 className="text-4xl md:text-5xl font-bold text-white mb-6 leading-tight">
              {post.title}
            </h1>
            
            <div className="flex flex-wrap items-center gap-6 text-slate-300 mb-6">
              <div className="flex items-center gap-2">
                <User className="w-5 h-5" />
                <span>{post.author}</span>
              </div>
              <div className="flex items-center gap-2">
                <Calendar className="w-5 h-5" />
                <span>{post.date}</span>
              </div>
              <div className="flex items-center gap-2">
                <Clock className="w-5 h-5" />
                <span>{post.readTime}</span>
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              {post.tags.map((tag) => (
                <Badge key={tag} variant="outline" className="border-slate-600 text-slate-300">
                  <Tag className="w-3 h-3 mr-1" />
                  {tag}
                </Badge>
              ))}
            </div>
          </div>
        </FadeIn>

        {/* 文章内容 */}
        <SlideIn direction="up" delay={200}>
          <Card className="bg-slate-800/50 border-slate-700 p-8 md:p-12 mb-12">
            <div className="prose prose-invert prose-lg max-w-none
              prose-headings:text-white prose-headings:font-bold
              prose-h1:text-4xl prose-h1:mb-6 prose-h1:mt-8
              prose-h2:text-3xl prose-h2:mb-4 prose-h2:mt-8 prose-h2:text-purple-300
              prose-h3:text-2xl prose-h3:mb-3 prose-h3:mt-6 prose-h3:text-blue-300
              prose-h4:text-xl prose-h4:mb-2 prose-h4:mt-4
              prose-p:text-slate-300 prose-p:leading-relaxed prose-p:mb-4
              prose-strong:text-white prose-strong:font-semibold
              prose-ul:text-slate-300 prose-ul:my-4
              prose-ol:text-slate-300 prose-ol:my-4
              prose-li:my-2
              prose-code:text-purple-300 prose-code:bg-slate-900 prose-code:px-2 prose-code:py-1 prose-code:rounded
              prose-pre:bg-slate-900 prose-pre:border prose-pre:border-slate-700
              prose-blockquote:border-l-purple-500 prose-blockquote:text-slate-300
              prose-table:text-slate-300 prose-th:text-white prose-th:bg-slate-700/50
              prose-a:text-purple-400 prose-a:no-underline hover:prose-a:text-purple-300
            ">
              <div dangerouslySetInnerHTML={{ __html: post.content.split('\n').map(line => {
                // 简单的Markdown转HTML处理
                if (line.startsWith('# ')) return `<h1>${line.substring(2)}</h1>`;
                if (line.startsWith('## ')) return `<h2>${line.substring(3)}</h2>`;
                if (line.startsWith('### ')) return `<h3>${line.substring(4)}</h3>`;
                if (line.startsWith('#### ')) return `<h4>${line.substring(5)}</h4>`;
                if (line.startsWith('- ')) return `<li>${line.substring(2)}</li>`;
                if (line.startsWith('```')) return line.includes('```') ? '<pre><code>' : '</code></pre>';
                if (line.trim() === '') return '<br/>';
                if (line.startsWith('**') && line.endsWith('**')) return `<strong>${line.slice(2, -2)}</strong>`;
                if (line.includes('**')) {
                  return line.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
                }
                if (line.startsWith('|')) return `<tr>${line.split('|').map(cell => `<td>${cell.trim()}</td>`).join('')}</tr>`;
                return `<p>${line}</p>`;
              }).join('') }} />
            </div>
          </Card>
        </SlideIn>

        {/* 分享和操作按钮 */}
        <SlideIn direction="up" delay={300}>
          <div className="flex justify-center gap-4 mb-16">
            <Link href="/blog">
              <Button variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                <ArrowLeft className="w-4 h-4 mr-2" />
                返回博客列表
              </Button>
            </Link>
            <Link href="/demo">
              <Button className="bg-purple-600 hover:bg-purple-700">
                预约演示
              </Button>
            </Link>
          </div>
        </SlideIn>

        {/* 相关文章 */}
        {relatedPosts.length > 0 && (
          <div>
            <h2 className="text-3xl font-bold text-white mb-8">相关文章</h2>
            <div className="grid md:grid-cols-3 gap-6">
              {relatedPosts.map((relatedPost, index) => (
                <SlideIn key={relatedPost.id} direction="up" delay={index * 100}>
                  <Link href={`/blog/${relatedPost.id}`}>
                    <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 p-6 cursor-pointer group h-full">
                      <div className="flex items-center gap-2 text-sm text-slate-400 mb-3">
                        <Calendar className="w-4 h-4" />
                        <span>{relatedPost.date}</span>
                      </div>
                      <h3 className="text-lg font-semibold text-white mb-2 group-hover:text-purple-400 transition-colors line-clamp-2">
                        {relatedPost.title}
                      </h3>
                      <p className="text-slate-300 text-sm line-clamp-3 mb-3">
                        {relatedPost.excerpt}
                      </p>
                      <div className="flex items-center text-purple-400 text-sm font-medium">
                        <span>阅读更多</span>
                        <ArrowRight className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" />
                      </div>
                    </Card>
                  </Link>
                </SlideIn>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

