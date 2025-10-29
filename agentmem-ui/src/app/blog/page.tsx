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
      title: 'AgentMem v1.0 正式发布：下一代智能记忆管理平台',
      excerpt: '经过数月的开发和测试，AgentMem v1.0 正式发布。本文详细介绍了新版本的核心功能、性能提升和使用指南。',
      content: '详细内容...',
      author: 'AgentMem 团队',
      date: '2024-01-15',
      readTime: '8 分钟',
      category: 'product',
      tags: ['发布', '产品更新', 'v1.0'],
      featured: true,
      image: 'https://trae-api-us.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20AI%20memory%20management%20platform%20dashboard%20with%20neural%20network%20visualization%20and%20data%20flow%20diagrams%2C%20sleek%20dark%20theme%20interface&image_size=landscape_16_9',
    },
    {
      id: 2,
      title: '深入理解 AI 代理记忆机制：从理论到实践',
      excerpt: '探索 AI 代理如何存储、检索和利用记忆信息，以及 AgentMem 如何优化这一过程。',
      content: '详细内容...',
      author: '李明博士',
      date: '2024-01-10',
      readTime: '12 分钟',
      category: 'technical',
      tags: ['AI', '记忆机制', '技术深度'],
      featured: false,
      image: 'https://trae-api-us.mchost.guru/api/ide/v1/text_to_image?prompt=abstract%20representation%20of%20AI%20memory%20networks%20with%20interconnected%20nodes%20and%20data%20pathways%2C%20futuristic%20blue%20and%20purple%20color%20scheme&image_size=landscape_16_9',
    },
    {
      id: 3,
      title: '企业级 AI 记忆管理最佳实践指南',
      excerpt: '为企业客户提供的完整指南，涵盖部署、配置、安全和性能优化等关键方面。',
      content: '详细内容...',
      author: '张伟',
      date: '2024-01-05',
      readTime: '15 分钟',
      category: 'guide',
      tags: ['企业', '最佳实践', '部署'],
      featured: false,
      image: 'https://trae-api-us.mchost.guru/api/ide/v1/text_to_image?prompt=enterprise%20data%20center%20with%20servers%20and%20AI%20processing%20units%2C%20professional%20corporate%20environment%20with%20modern%20technology&image_size=landscape_16_9',
    },
    {
      id: 4,
      title: 'Rust 在 AI 基础设施中的优势：性能与安全并重',
      excerpt: '分析为什么选择 Rust 作为 AgentMem 的核心开发语言，以及它在 AI 基础设施中的独特优势。',
      content: '详细内容...',
      author: '王小红',
      date: '2023-12-28',
      readTime: '10 分钟',
      category: 'technical',
      tags: ['Rust', '性能', '安全'],
      featured: false,
      image: 'https://trae-api-us.mchost.guru/api/ide/v1/text_to_image?prompt=Rust%20programming%20language%20logo%20integrated%20with%20AI%20and%20performance%20metrics%2C%20technical%20illustration%20with%20code%20elements&image_size=landscape_16_9',
    },
    {
      id: 5,
      title: '多模态记忆处理：文本、图像、音频的统一管理',
      excerpt: '介绍 AgentMem 如何处理不同类型的数据，实现真正的多模态智能记忆管理。',
      content: '详细内容...',
      author: '陈晓明',
      date: '2023-12-20',
      readTime: '9 分钟',
      category: 'technical',
      tags: ['多模态', '数据处理', '创新'],
      featured: true,
      image: 'https://trae-api-us.mchost.guru/api/ide/v1/text_to_image?prompt=multimodal%20AI%20processing%20with%20text%2C%20images%2C%20and%20audio%20waveforms%20converging%20into%20unified%20data%20stream%2C%20colorful%20visualization&image_size=landscape_16_9',
    },
    {
      id: 6,
      title: 'AgentMem 生态系统：构建开发者友好的 AI 平台',
      excerpt: '探讨如何构建一个繁荣的开发者生态系统，包括 SDK、插件市场和社区建设。',
      content: '详细内容...',
      author: 'AgentMem 团队',
      date: '2023-12-15',
      readTime: '7 分钟',
      category: 'ecosystem',
      tags: ['生态系统', '开发者', '社区'],
      featured: false,
      image: 'https://trae-api-us.mchost.guru/api/ide/v1/text_to_image?prompt=developer%20ecosystem%20with%20interconnected%20platforms%2C%20APIs%2C%20and%20community%20elements%2C%20modern%20tech%20illustration&image_size=landscape_16_9',
    },
  ];

  // 分类数据
  const categories = [
    { id: 'all', name: '全部', count: blogPosts.length },
    { id: 'product', name: '产品更新', count: blogPosts.filter(post => post.category === 'product').length },
    { id: 'technical', name: '技术文章', count: blogPosts.filter(post => post.category === 'technical').length },
    { id: 'guide', name: '使用指南', count: blogPosts.filter(post => post.category === 'guide').length },
    { id: 'ecosystem', name: '生态系统', count: blogPosts.filter(post => post.category === 'ecosystem').length },
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
              探索 AI 记忆管理的前沿技术，分享产品更新和行业洞察，与开发者社区共同成长。
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