'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { FadeIn, SlideIn } from '@/components/ui/animations';
import { Calendar, Clock, User, Search, Tag, ArrowRight } from 'lucide-react';

/**
 * 博客页面组件
 * 展示 AgentMem 相关的技术文章、产品更新和行业洞察
 */
export default function BlogPage() {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');

  // 博客文章数据
  const blogPosts = [
    {
      id: 1,
      title: 'AI记忆管理市场：千亿级商业机会深度分析',
      excerpt: '向量数据库市场预计2030年达到$100亿美元，AI记忆管理作为核心基础设施，将成为企业数字化转型的关键。本文深入分析市场规模、增长驱动力和投资机会。',
      content: '详细内容...',
      author: '张伟',
      date: '2025-01-06',
      readTime: '15 分钟',
      category: 'business',
      tags: ['市场分析', '商业机会', '投资'],
      featured: true,
      image: 'https://via.placeholder.com/800x450/6366f1/ffffff?text=AI+Memory+Market+Analysis',
    },
    {
      id: 2,
      title: '学术前沿：AI Agent记忆系统研究论文精选（2024-2025）',
      excerpt: '汇总最新的AI Agent记忆系统研究论文，包括AgentRM的奖励建模、A-MEM的Zettelkasten方法、LCMR的协同过滤等前沿研究成果。',
      content: '详细内容...',
      author: '李明博士',
      date: '2025-01-05',
      readTime: '20 分钟',
      category: 'research',
      tags: ['学术论文', 'AI Agent', '记忆系统'],
      featured: true,
      image: 'https://via.placeholder.com/800x450/8b5cf6/ffffff?text=Research+Papers',
    },
    {
      id: 3,
      title: 'AgentMem如何帮助企业降低99%的LLM成本',
      excerpt: '通过智能记忆管理，企业可以大幅减少对LLM API的调用次数和token消耗。实测数据显示，AgentMem可帮助企业节省高达99%的运营成本。',
      content: '详细内容...',
      author: '王晓明',
      date: '2025-01-04',
      readTime: '12 分钟',
      category: 'business',
      tags: ['成本优化', 'ROI', '企业应用'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/10b981/ffffff?text=Cost+Reduction+99%',
    },
    {
      id: 4,
      title: '低空经济×AI记忆：万亿市场的数智化赋能',
      excerpt: '2025年中国低空经济规模突破1.5万亿元。AgentMem为无人机、eVTOL提供智能数据管理，助力低空经济数智化转型。',
      content: '详细内容...',
      author: '陈晨',
      date: '2025-01-03',
      readTime: '10 分钟',
      category: 'industry',
      tags: ['低空经济', '行业应用', '智能调度'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/3b82f6/ffffff?text=Low+Altitude+Economy',
    },
    {
      id: 5,
      title: '金融AI的记忆革命：从风控到投顾的全链路升级',
      excerpt: '金融机构如何利用AgentMem实现智能风控、精准营销和个性化投顾。真实案例：某银行坏账率降低40%，客户满意度提升85%。',
      content: '详细内容...',
      author: '刘芳',
      date: '2025-01-02',
      readTime: '14 分钟',
      category: 'industry',
      tags: ['金融科技', '风控', '客户画像'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/f59e0b/ffffff?text=FinTech+AI+Memory',
    },
    {
      id: 6,
      title: 'Rust赋能AI：为什么AgentMem选择Rust而不是Python',
      excerpt: '深入分析Rust在AI基础设施中的优势：性能提升5-10倍、内存安全、零成本抽象。对比Python方案的性能基准测试详细数据。',
      content: '详细内容...',
      author: '赵强',
      date: '2025-01-01',
      readTime: '11 分钟',
      category: 'technical',
      tags: ['Rust', '性能优化', '技术选型'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/ef4444/ffffff?text=Rust+vs+Python',
    },
    {
      id: 7,
      title: 'RecNMP论文解读：近内存处理加速个性化推荐',
      excerpt: '详细解读arXiv:1912.12953论文，探讨如何通过近内存处理技术加速推荐系统，提升10倍吞吐量并节省50%内存能耗。AgentMem的相关实现。',
      content: '详细内容...',
      author: '李明博士',
      date: '2024-12-30',
      readTime: '16 分钟',
      category: 'research',
      tags: ['论文解读', '推荐系统', '性能优化'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/8b5cf6/ffffff?text=RecNMP+Paper',
    },
    {
      id: 8,
      title: 'AI记忆的未来：从单一模型到多Agent协同',
      excerpt: '探索未来AI记忆系统的发展方向：多Agent协同、跨模态记忆融合、持续学习与知识蒸馏。AgentMem的技术路线图。',
      content: '详细内容...',
      author: 'AgentMem 团队',
      date: '2024-12-28',
      readTime: '13 分钟',
      category: 'future',
      tags: ['技术趋势', '多Agent', '未来展望'],
      featured: true,
      image: 'https://via.placeholder.com/800x450/ec4899/ffffff?text=Future+of+AI+Memory',
    },
    {
      id: 9,
      title: '实战案例：某零售巨头如何用AgentMem提升客户体验',
      excerpt: '某零售企业通过AgentMem实现跨渠道客户记忆统一，客户留存率提升45%，复购率提升60%。详细技术实现和ROI分析。',
      content: '详细内容...',
      author: '孙华',
      date: '2024-12-25',
      readTime: '9 分钟',
      category: 'case',
      tags: ['客户案例', '零售', 'ROI'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/06b6d4/ffffff?text=Retail+Case+Study',
    },
    {
      id: 10,
      title: 'LCMR论文：非结构化文本的协同过滤记忆策略',
      excerpt: '解读arXiv:1804.06201，探讨本地记忆和集中式记忆在处理非结构化文本中的应用，以及对AgentMem架构设计的启发。',
      content: '详细内容...',
      author: '李明博士',
      date: '2024-12-20',
      readTime: '14 分钟',
      category: 'research',
      tags: ['论文解读', '协同过滤', '文本处理'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/8b5cf6/ffffff?text=LCMR+Paper',
    },
    {
      id: 11,
      title: 'AgentMem v1.0 正式发布：下一代智能记忆管理平台',
      excerpt: '经过数月的开发和测试，AgentMem v1.0 正式发布。100% Mem0兼容、5-10x性能提升、8+存储后端支持。完整功能介绍和迁移指南。',
      content: '详细内容...',
      author: 'AgentMem 团队',
      date: '2024-12-15',
      readTime: '8 分钟',
      category: 'product',
      tags: ['发布', '产品更新', 'v1.0'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/6366f1/ffffff?text=AgentMem+v1.0',
    },
    {
      id: 12,
      title: '向量数据库选型指南：Pinecone vs Qdrant vs Chroma',
      excerpt: '全面对比主流向量数据库的性能、成本和适用场景。AgentMem支持8+种存储后端，如何选择最适合你的方案。',
      content: '详细内容...',
      author: '周杰',
      date: '2024-12-10',
      readTime: '12 分钟',
      category: 'technical',
      tags: ['向量数据库', '技术选型', '对比分析'],
      featured: false,
      image: 'https://via.placeholder.com/800x450/14b8a6/ffffff?text=Vector+Database+Guide',
    },
  ];

  // 分类数据
  const categories = [
    { id: 'all', name: '全部', count: blogPosts.length },
    { id: 'business', name: '商业洞察', count: blogPosts.filter(post => post.category === 'business').length },
    { id: 'research', name: '学术研究', count: blogPosts.filter(post => post.category === 'research').length },
    { id: 'industry', name: '行业应用', count: blogPosts.filter(post => post.category === 'industry').length },
    { id: 'technical', name: '技术深度', count: blogPosts.filter(post => post.category === 'technical').length },
    { id: 'case', name: '客户案例', count: blogPosts.filter(post => post.category === 'case').length },
    { id: 'future', name: '未来展望', count: blogPosts.filter(post => post.category === 'future').length },
    { id: 'product', name: '产品更新', count: blogPosts.filter(post => post.category === 'product').length },
  ];

  // 过滤文章
  const filteredPosts = blogPosts.filter(post => {
    const matchesSearch = post.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         post.excerpt.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         post.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
    const matchesCategory = selectedCategory === 'all' || post.category === selectedCategory;
    return matchesSearch && matchesCategory;
  });

  // 特色文章
  const featuredPosts = blogPosts.filter(post => post.featured);

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="container mx-auto px-4 py-16">
        {/* 页面头部 */}
        <FadeIn>
          <div className="text-center mb-16">
            <Badge className="mb-4 bg-purple-500/20 text-purple-300 border-purple-500/30">
              技术博客
            </Badge>
            <h1 className="text-4xl md:text-6xl font-bold text-white mb-6">
              AgentMem
              <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                技术博客
              </span>
            </h1>
            <p className="text-xl text-gray-300 max-w-3xl mx-auto">
              探索 AI 记忆管理的前沿技术、商业机会和学术研究。分享产品更新、行业洞察和真实案例，与开发者社区共同成长。
            </p>
          </div>
        </FadeIn>

        {/* 搜索和筛选 */}
        <FadeIn delay={200}>
          <div className="mb-12">
            <div className="flex flex-col md:flex-row gap-4 mb-8">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <Input
                  placeholder="搜索文章、标签或关键词..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10 bg-slate-800/50 border-slate-700 text-white placeholder-gray-400"
                />
              </div>
            </div>
            
            {/* 分类筛选 */}
            <div className="flex flex-wrap gap-2">
              {categories.map((category) => (
                <button
                  key={category.id}
                  onClick={() => setSelectedCategory(category.id)}
                  className={`px-4 py-2 rounded-full text-sm font-medium transition-all ${
                    selectedCategory === category.id
                      ? 'bg-purple-600 text-white'
                      : 'bg-slate-800/50 text-gray-300 hover:bg-slate-700 border border-slate-700'
                  }`}
                >
                  {category.name} ({category.count})
                </button>
              ))}
            </div>
          </div>
        </FadeIn>

        {/* 特色文章 */}
        {selectedCategory === 'all' && (
          <div className="mb-16">
            <FadeIn delay={300}>
              <h2 className="text-3xl font-bold text-white mb-8">特色文章</h2>
              <div className="grid md:grid-cols-2 gap-8">
                {featuredPosts.map((post, index) => (
                  <SlideIn key={post.id} delay={index * 100} direction="up">
                    <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 overflow-hidden group">
                      <div className="aspect-video bg-gradient-to-r from-purple-600 to-pink-600 relative overflow-hidden">
                        <img 
                          src={post.image} 
                          alt={post.title}
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                        />
                        <Badge className="absolute top-4 left-4 bg-purple-600 text-white">
                          特色
                        </Badge>
                      </div>
                      <div className="p-6">
                        <div className="flex items-center gap-4 text-sm text-gray-400 mb-3">
                          <div className="flex items-center gap-1">
                            <User className="w-4 h-4" />
                            <span>{post.author}</span>
                          </div>
                          <div className="flex items-center gap-1">
                            <Calendar className="w-4 h-4" />
                            <span>{post.date}</span>
                          </div>
                          <div className="flex items-center gap-1">
                            <Clock className="w-4 h-4" />
                            <span>{post.readTime}</span>
                          </div>
                        </div>
                        <h3 className="text-xl font-semibold text-white mb-3 group-hover:text-purple-400 transition-colors">
                          {post.title}
                        </h3>
                        <p className="text-gray-300 mb-4">{post.excerpt}</p>
                        <div className="flex items-center justify-between">
                          <div className="flex flex-wrap gap-2">
                            {post.tags.slice(0, 2).map((tag) => (
                              <Badge key={tag} variant="outline" className="text-xs border-slate-600 text-gray-400">
                                <Tag className="w-3 h-3 mr-1" />
                                {tag}
                              </Badge>
                            ))}
                          </div>
                          <Button variant="ghost" size="sm" className="text-purple-400 hover:text-purple-300">
                            阅读更多
                            <ArrowRight className="w-4 h-4 ml-1" />
                          </Button>
                        </div>
                      </div>
                    </Card>
                  </SlideIn>
                ))}
              </div>
            </FadeIn>
          </div>
        )}

        {/* 所有文章 */}
        <div>
          <FadeIn delay={400}>
            <h2 className="text-3xl font-bold text-white mb-8">
              {selectedCategory === 'all' ? '最新文章' : categories.find(c => c.id === selectedCategory)?.name}
            </h2>
            {filteredPosts.length === 0 ? (
              <Card className="bg-slate-800/50 border-slate-700 p-12 text-center">
                <p className="text-gray-400 text-lg">没有找到匹配的文章</p>
                <p className="text-gray-500 mt-2">尝试调整搜索条件或选择其他分类</p>
              </Card>
            ) : (
              <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
                {filteredPosts.map((post, index) => (
                  <SlideIn key={post.id} delay={index * 50} direction="up">
                    <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 overflow-hidden group h-full flex flex-col">
                      <div className="aspect-video bg-gradient-to-r from-purple-600 to-pink-600 relative overflow-hidden">
                        <img 
                          src={post.image} 
                          alt={post.title}
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                        />
                        {post.featured && (
                          <Badge className="absolute top-4 left-4 bg-purple-600 text-white">
                            特色
                          </Badge>
                        )}
                      </div>
                      <div className="p-6 flex-1 flex flex-col">
                        <div className="flex items-center gap-4 text-sm text-gray-400 mb-3">
                          <div className="flex items-center gap-1">
                            <User className="w-4 h-4" />
                            <span>{post.author}</span>
                          </div>
                          <div className="flex items-center gap-1">
                            <Calendar className="w-4 h-4" />
                            <span>{post.date}</span>
                          </div>
                        </div>
                        <h3 className="text-lg font-semibold text-white mb-3 group-hover:text-purple-400 transition-colors line-clamp-2">
                          {post.title}
                        </h3>
                        <p className="text-gray-300 mb-4 flex-1 line-clamp-3">{post.excerpt}</p>
                        <div className="flex items-center justify-between mt-auto">
                          <div className="flex items-center gap-1 text-sm text-gray-400">
                            <Clock className="w-4 h-4" />
                            <span>{post.readTime}</span>
                          </div>
                          <Button variant="ghost" size="sm" className="text-purple-400 hover:text-purple-300">
                            阅读
                            <ArrowRight className="w-4 h-4 ml-1" />
                          </Button>
                        </div>
                      </div>
                    </Card>
                  </SlideIn>
                ))}
              </div>
            )}
          </FadeIn>
        </div>

        {/* 订阅区域 */}
        <FadeIn delay={600}>
          <div className="mt-20">
            <Card className="bg-gradient-to-r from-purple-900/50 to-pink-900/50 border-purple-500/30 p-8 text-center">
              <h3 className="text-2xl font-bold text-white mb-4">订阅我们的博客</h3>
              <p className="text-gray-300 mb-6 max-w-2xl mx-auto">
                获取最新的技术文章、产品更新和行业洞察，第一时间了解 AgentMem 的发展动态。
              </p>
              <div className="flex flex-col sm:flex-row gap-4 max-w-md mx-auto">
                <Input 
                  placeholder="输入您的邮箱地址"
                  className="bg-slate-800/50 border-slate-700 text-white placeholder-gray-400"
                />
                <Button className="bg-purple-600 hover:bg-purple-700 whitespace-nowrap">
                  立即订阅
                </Button>
              </div>
            </Card>
          </div>
        </FadeIn>
      </div>
    </div>
  );
}