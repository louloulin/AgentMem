'use client';

import { useState, Suspense } from 'react';

// Disable static generation for this page
export const dynamic = 'force-dynamic';
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { CodeBlock } from "@/components/ui/code-block";
import { FadeIn, SlideIn } from "@/components/ui/animations";
import { 
  ScrollToTopButton, 
  PageNavigation, 
  MobilePageNavigation, 
  ScrollProgressIndicator,
  SmoothScrollLink 
} from "@/components/ui/smooth-scroll";
import { Breadcrumb, BreadcrumbContainer } from "@/components/ui/breadcrumb";
import { 
  Brain, 
  Book, 
  Code, 
  Terminal, 
  Zap, 
  Database, 
  Settings, 
  ArrowRight, 
  Search,
  Copy,
  CheckCircle,
  ExternalLink,
  Play,
  FileText,
  Layers,
  Shield,
  Cpu,
  Network,
  Globe,
  Clock
} from "lucide-react";
import Link from "next/link";

/**
 * 文档页面组件 - 展示API文档、快速开始指南和使用示例
 */
const DocsPageContent = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [activeSection, setActiveSection] = useState('quick-start');
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  // 复制代码到剪贴板
  const copyToClipboard = async (code: string, id: string) => {
    try {
      await navigator.clipboard.writeText(code);
      setCopiedCode(id);
      setTimeout(() => setCopiedCode(null), 2000);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  };

  // API 端点数据
  const apiEndpoints = [
    {
      method: 'POST',
      endpoint: '/api/v1/memories',
      description: '添加新的记忆',
      parameters: [
        { name: 'user_id', type: 'string', required: true, description: '用户ID' },
        { name: 'content', type: 'string', required: true, description: '记忆内容' },
        { name: 'metadata', type: 'object', required: false, description: '元数据' }
      ],
      example: `{
  "user_id": "user123",
  "content": "我喜欢在周末喝咖啡",
  "metadata": {
    "category": "preference",
    "importance": 0.8
  }
}`
    },
    {
      method: 'GET',
      endpoint: '/api/v1/memories/search',
      description: '搜索相关记忆',
      parameters: [
        { name: 'query', type: 'string', required: true, description: '搜索查询' },
        { name: 'user_id', type: 'string', required: true, description: '用户ID' },
        { name: 'limit', type: 'number', required: false, description: '返回数量限制' }
      ],
      example: `{
  "query": "咖啡",
  "user_id": "user123",
  "limit": 10
}`
    },
    {
      method: 'PUT',
      endpoint: '/api/v1/memories/{id}',
      description: '更新现有记忆',
      parameters: [
        { name: 'id', type: 'string', required: true, description: '记忆ID' },
        { name: 'content', type: 'string', required: false, description: '新的记忆内容' },
        { name: 'metadata', type: 'object', required: false, description: '新的元数据' }
      ],
      example: `{
  "content": "我更喜欢在周末早上喝咖啡",
  "metadata": {
    "importance": 0.9
  }
}`
    },
    {
      method: 'DELETE',
      endpoint: '/api/v1/memories/{id}',
      description: '删除指定记忆',
      parameters: [
        { name: 'id', type: 'string', required: true, description: '记忆ID' }
      ],
      example: null
    }
  ];

  // 快速开始代码示例
  const quickStartCode = `use agent_mem_core::{MemoryEngine, MemoryEngineConfig};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建记忆引擎
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config).await?;
    
    // 添加记忆
    let memory_id = engine.add_memory(
        "user123",
        "我喜欢在周末喝咖啡和阅读",
        None
    ).await?;
    
    println!("记忆已添加，ID: {}", memory_id);
    
    // 搜索记忆
    let results = engine.search_memories(
        "咖啡", 
        "user123", 
        10
    ).await?;
    
    println!("找到 {} 条相关记忆", results.len());
    Ok(())
}`;

  // 教程数据
  const tutorials = [
    {
      id: 'basic-usage',
      title: '基础使用教程',
      description: '学习 AgentMem 的基本概念和使用方法',
      duration: '15 分钟',
      level: '初级',
      topics: ['安装配置', '基本 API', '记忆管理', '搜索功能']
    },
    {
      id: 'advanced-features',
      title: '高级功能指南',
      description: '深入了解智能推理和高级配置',
      duration: '30 分钟',
      level: '中级',
      topics: ['智能推理', '自定义存储', '性能优化', '监控告警']
    },
    {
      id: 'production-deployment',
      title: '生产环境部署',
      description: '企业级部署和运维最佳实践',
      duration: '45 分钟',
      level: '高级',
      topics: ['Kubernetes 部署', '负载均衡', '安全配置', '监控运维']
    },
    {
      id: 'integration-guide',
      title: '集成开发指南',
      description: '与现有系统集成的完整指南',
      duration: '25 分钟',
      level: '中级',
      topics: ['SDK 使用', 'Webhook 配置', '第三方集成', '错误处理']
    }
  ];

  // 页面锚点配置
  const pageAnchors = [
    { id: 'quick-start', label: '快速开始' },
    { id: 'tutorials', label: '教程指南' },
    { id: 'api-reference', label: 'API 参考' },
    { id: 'installation', label: '安装指南' },
    { id: 'examples', label: '代码示例' },
    { id: 'architecture', label: '架构设计' },
    { id: 'deployment', label: '部署指南' },
    { id: 'sdk', label: 'SDK 文档' },
    { id: 'help', label: '获取帮助' }
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* 滚动进度指示器 */}
      <ScrollProgressIndicator />
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <Link href="/" className="flex items-center">
                <Brain className="h-8 w-8 text-purple-400" />
                <span className="ml-2 text-xl font-bold text-white">AgentMem</span>
              </Link>
            </div>
            <div className="flex items-center space-x-8">
              <Link href="/" className="text-slate-300 hover:text-white transition-colors">
                首页
              </Link>
              <Link href="/docs" className="text-white font-semibold">
                文档
              </Link>
              <Link href="/demo" className="text-slate-300 hover:text-white transition-colors">
                演示
              </Link>
              <Link href="/pricing" className="text-slate-300 hover:text-white transition-colors">
                定价
              </Link>
              <Link href="/blog" className="text-slate-300 hover:text-white transition-colors">
                博客
              </Link>
              <Link href="/support" className="text-slate-300 hover:text-white transition-colors">
                支持
              </Link>
              <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                <ExternalLink className="w-4 h-4 mr-2" />
                GitHub
              </Button>
            </div>
          </div>
        </div>
      </nav>

      {/* 面包屑导航 */}
      <BreadcrumbContainer>
        <Breadcrumb />
      </BreadcrumbContainer>

      {/* 页面头部 */}
      <div className="bg-gradient-to-r from-purple-900/30 to-pink-900/30 border-b border-purple-500/20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
          <FadeIn>
            <div className="text-center">
              <Badge className="mb-4 bg-purple-500/20 text-purple-300 border-purple-500/30">
                开发文档
              </Badge>
              <h1 className="text-4xl md:text-6xl font-bold text-white mb-6">
                AgentMem
                <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                  开发文档
                </span>
              </h1>
              <p className="text-xl text-gray-300 max-w-3xl mx-auto mb-8">
                完整的 API 文档、快速开始指南和最佳实践，助您快速构建智能记忆应用
              </p>
              
              {/* 搜索框 */}
              <div className="max-w-2xl mx-auto">
                <div className="relative">
                  <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                  <Input
                    placeholder="搜索文档、API 或示例代码..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-12 py-3 bg-slate-800/50 border-slate-700 text-white placeholder-gray-400 rounded-xl"
                  />
                </div>
              </div>
            </div>
          </FadeIn>
        </div>
      </div>

      {/* 文档内容 */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          {/* 侧边栏导航 */}
          <div className="lg:col-span-1">
            <div className="sticky top-24">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white flex items-center">
                    <Book className="h-5 w-5 mr-2" />
                    文档导航
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <nav className="space-y-1">
                    {[
                      { id: 'quick-start', label: '快速开始', icon: Zap },
                      { id: 'installation', label: '安装指南', icon: Terminal },
                      { id: 'tutorials', label: '教程指南', icon: FileText },
                      { id: 'api-reference', label: 'API 参考', icon: Code },
                      { id: 'examples', label: '代码示例', icon: Play },
                      { id: 'architecture', label: '架构设计', icon: Layers },
                      { id: 'deployment', label: '部署指南', icon: Settings },
                      { id: 'sdk', label: 'SDK 文档', icon: Database }
                    ].map((item) => {
                      const Icon = item.icon;
                      return (
                        <button
                          key={item.id}
                          onClick={() => setActiveSection(item.id)}
                          className={`w-full text-left flex items-center py-2 px-3 rounded-lg transition-colors ${
                            activeSection === item.id
                              ? 'bg-purple-600/20 text-purple-400 border-l-2 border-purple-400'
                              : 'text-slate-300 hover:text-purple-400 hover:bg-slate-700/50'
                          }`}
                        >
                          <Icon className="w-4 h-4 mr-2" />
                          {item.label}
                        </button>
                      );
                    })}
                  </nav>
                </CardContent>
              </Card>
            </div>
          </div>

          {/* 主要内容 */}
          <div className="lg:col-span-3">
            {/* 动态内容区域 */}
            <div className="space-y-12">
              {/* 快速开始 */}
              {activeSection === 'quick-start' && (
                <FadeIn>
                  <div id="quick-start">
                    <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                      <Zap className="h-8 w-8 text-yellow-400 mr-3" />
                      快速开始
                    </h2>
                    
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                      <SlideIn direction="up" delay={100}>
                        <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
                          <CardHeader>
                            <div className="w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center mb-4">
                              <span className="text-2xl font-bold text-purple-400">1</span>
                            </div>
                            <CardTitle className="text-white">安装 AgentMem</CardTitle>
                          </CardHeader>
                          <CardContent className="text-slate-300">
                            <p className="mb-4">使用包管理器快速安装 AgentMem</p>
                            <div className="bg-slate-900 p-3 rounded-lg">
                              <code className="text-green-400 text-sm">
                                cargo add agent-mem-core
                              </code>
                            </div>
                          </CardContent>
                        </Card>
                      </SlideIn>
                      
                      <SlideIn direction="up" delay={200}>
                        <Card className="bg-slate-800/50 border-slate-700 hover:border-blue-500/50 transition-all duration-300">
                          <CardHeader>
                            <div className="w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center mb-4">
                              <span className="text-2xl font-bold text-blue-400">2</span>
                            </div>
                            <CardTitle className="text-white">配置环境</CardTitle>
                          </CardHeader>
                          <CardContent className="text-slate-300">
                            <p className="mb-4">设置 API 密钥和存储后端</p>
                            <div className="bg-slate-900 p-3 rounded-lg">
                              <code className="text-green-400 text-sm">
                                export DEEPSEEK_API_KEY=your_key
                              </code>
                            </div>
                          </CardContent>
                        </Card>
                      </SlideIn>
                      
                      <SlideIn direction="up" delay={300}>
                        <Card className="bg-slate-800/50 border-slate-700 hover:border-green-500/50 transition-all duration-300">
                          <CardHeader>
                            <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                              <span className="text-2xl font-bold text-green-400">3</span>
                            </div>
                            <CardTitle className="text-white">开始编码</CardTitle>
                          </CardHeader>
                          <CardContent className="text-slate-300">
                            <p className="mb-4">创建您的第一个记忆应用</p>
                            <Button size="sm" className="bg-green-600 hover:bg-green-700">
                              <Play className="w-4 h-4 mr-2" />
                              运行示例
                            </Button>
                          </CardContent>
                        </Card>
                      </SlideIn>
                    </div>
                    
                    {/* 5分钟快速体验 */}
                    <Card className="bg-gradient-to-r from-purple-900/30 to-pink-900/30 border-purple-500/30 mb-8">
                      <CardHeader>
                        <CardTitle className="text-white text-xl">5分钟快速体验</CardTitle>
                        <CardDescription className="text-slate-300">
                          跟随这个简单示例，快速了解 AgentMem 的核心功能
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        <div className="bg-slate-900 p-4 rounded-lg relative">
                          <button
                            onClick={() => copyToClipboard(quickStartCode, 'quick-start')}
                            className="absolute top-2 right-2 p-2 text-gray-400 hover:text-white transition-colors"
                          >
                            {copiedCode === 'quick-start' ? (
                              <CheckCircle className="w-4 h-4 text-green-400" />
                            ) : (
                              <Copy className="w-4 h-4" />
                            )}
                          </button>
                          <pre className="text-green-400 text-sm overflow-x-auto">
                            <code>{quickStartCode}</code>
                          </pre>
                        </div>
                      </CardContent>
                    </Card>
                  </div>
                </FadeIn>
              )}
              
              {/* 教程指南 */}
              {activeSection === 'tutorials' && (
                <FadeIn>
                  <div id="tutorials">
                    <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                      <FileText className="h-8 w-8 text-blue-400 mr-3" />
                      教程指南
                    </h2>
                    
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                      {tutorials.map((tutorial, index) => (
                        <SlideIn key={tutorial.id} direction="up" delay={index * 100}>
                          <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 h-full">
                            <CardHeader>
                              <div className="flex items-center justify-between mb-2">
                                <Badge className={`${
                                  tutorial.level === '初级' ? 'bg-green-500/20 text-green-400 border-green-500/30' :
                                  tutorial.level === '中级' ? 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30' :
                                  'bg-red-500/20 text-red-400 border-red-500/30'
                                }`}>
                                  {tutorial.level}
                                </Badge>
                                <span className="text-slate-400 text-sm flex items-center">
                                  <Clock className="w-4 h-4 mr-1" />
                                  {tutorial.duration}
                                </span>
                              </div>
                              <CardTitle className="text-white">{tutorial.title}</CardTitle>
                              <CardDescription className="text-slate-300">
                                {tutorial.description}
                              </CardDescription>
                            </CardHeader>
                            <CardContent className="flex-1">
                              <div className="space-y-2 mb-4">
                                {tutorial.topics.map((topic, idx) => (
                                  <div key={idx} className="flex items-center text-slate-300">
                                    <CheckCircle className="w-4 h-4 text-green-400 mr-2" />
                                    <span className="text-sm">{topic}</span>
                                  </div>
                                ))}
                              </div>
                              <Button className="w-full bg-purple-600 hover:bg-purple-700">
                                <Play className="w-4 h-4 mr-2" />
                                开始学习
                              </Button>
                            </CardContent>
                          </Card>
                        </SlideIn>
                      ))}
                    </div>
                  </div>
                </FadeIn>
              )}
              
              {/* API 参考 */}
              {activeSection === 'api-reference' && (
                <FadeIn>
                  <div id="api-reference">
                    <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                      <Code className="h-8 w-8 text-blue-400 mr-3" />
                      API 参考
                    </h2>
                    
                    <div className="space-y-6">
                      {apiEndpoints.map((endpoint, index) => (
                        <SlideIn key={index} direction="up" delay={index * 100}>
                          <Card className="bg-slate-800/50 border-slate-700">
                            <CardHeader>
                              <div className="flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                  <Badge className={`${
                                    endpoint.method === 'GET' ? 'bg-green-500/20 text-green-400 border-green-500/30' :
                                    endpoint.method === 'POST' ? 'bg-blue-500/20 text-blue-400 border-blue-500/30' :
                                    endpoint.method === 'PUT' ? 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30' :
                                    'bg-red-500/20 text-red-400 border-red-500/30'
                                  }`}>
                                    {endpoint.method}
                                  </Badge>
                                  <code className="text-purple-400 font-mono">{endpoint.endpoint}</code>
                                </div>
                              </div>
                              <CardDescription className="text-slate-300">
                                {endpoint.description}
                              </CardDescription>
                            </CardHeader>
                            <CardContent>
                              {/* 参数列表 */}
                              <div className="mb-4">
                                <h4 className="text-white font-semibold mb-3">参数</h4>
                                <div className="space-y-2">
                                  {endpoint.parameters.map((param, idx) => (
                                    <div key={idx} className="flex items-center justify-between p-2 bg-slate-900/50 rounded">
                                      <div className="flex items-center gap-2">
                                        <code className="text-purple-400">{param.name}</code>
                                        <Badge variant="outline" className="text-xs">{param.type}</Badge>
                                        {param.required && (
                                          <Badge className="bg-red-500/20 text-red-400 border-red-500/30 text-xs">
                                            必需
                                          </Badge>
                                        )}
                                      </div>
                                      <span className="text-slate-400 text-sm">{param.description}</span>
                                    </div>
                                  ))}
                                </div>
                              </div>
                              
                              {/* 示例代码 */}
                              {endpoint.example && (
                                <div>
                                  <h4 className="text-white font-semibold mb-3">请求示例</h4>
                                  <div className="bg-slate-900 p-4 rounded-lg relative">
                                    <button
                                      onClick={() => copyToClipboard(endpoint.example!, `api-${index}`)}
                                      className="absolute top-2 right-2 p-2 text-gray-400 hover:text-white transition-colors"
                                    >
                                      {copiedCode === `api-${index}` ? (
                                        <CheckCircle className="w-4 h-4 text-green-400" />
                                      ) : (
                                        <Copy className="w-4 h-4" />
                                      )}
                                    </button>
                                    <pre className="text-green-400 text-sm overflow-x-auto">
                                      <code>{endpoint.example}</code>
                                    </pre>
                                  </div>
                                </div>
                              )}
                            </CardContent>
                          </Card>
                        </SlideIn>
                      ))}
                    </div>
                  </div>
                </FadeIn>
               )}
               
               {/* 其他部分的占位符 */}
               {activeSection === 'installation' && (
                 <FadeIn>
                   <div id="installation">
                     <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                       <Terminal className="h-8 w-8 text-green-400 mr-3" />
                       安装指南
                     </h2>
                     <Card className="bg-slate-800/50 border-slate-700">
                       <CardContent className="p-6">
                         <p className="text-slate-300 text-center py-8">
                           详细的安装指南内容正在完善中...
                         </p>
                       </CardContent>
                     </Card>
                   </div>
                 </FadeIn>
               )}
               
               {activeSection === 'examples' && (
                 <FadeIn>
                   <div id="examples">
                     <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                       <Play className="h-8 w-8 text-purple-400 mr-3" />
                       代码示例
                     </h2>
                     <Card className="bg-slate-800/50 border-slate-700">
                       <CardContent className="p-6">
                         <p className="text-slate-300 text-center py-8">
                           更多代码示例正在添加中...
                         </p>
                       </CardContent>
                     </Card>
                   </div>
                 </FadeIn>
               )}
               
               {activeSection === 'architecture' && (
                 <FadeIn>
                   <div id="architecture">
                     <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                       <Layers className="h-8 w-8 text-green-400 mr-3" />
                       架构设计
                     </h2>
                     <Card className="bg-slate-800/50 border-slate-700">
                       <CardContent className="p-6">
                         <p className="text-slate-300 text-center py-8">
                           架构设计文档正在完善中...
                         </p>
                       </CardContent>
                     </Card>
                   </div>
                 </FadeIn>
               )}
               
               {activeSection === 'deployment' && (
                 <FadeIn>
                   <div id="deployment">
                     <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                       <Settings className="h-8 w-8 text-yellow-400 mr-3" />
                       部署指南
                     </h2>
                     <Card className="bg-slate-800/50 border-slate-700">
                       <CardContent className="p-6">
                         <p className="text-slate-300 text-center py-8">
                           部署指南正在编写中...
                         </p>
                       </CardContent>
                     </Card>
                   </div>
                 </FadeIn>
               )}
               
               {activeSection === 'sdk' && (
                 <FadeIn>
                   <div id="sdk">
                     <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                       <Database className="h-8 w-8 text-blue-400 mr-3" />
                       SDK 文档
                     </h2>
                     <Card className="bg-slate-800/50 border-slate-700">
                       <CardContent className="p-6">
                         <p className="text-slate-300 text-center py-8">
                           SDK 文档正在整理中...
                         </p>
                       </CardContent>
                     </Card>
                   </div>
                 </FadeIn>
               )}
            </div>

            {/* 获取帮助 */}
            <div id="help" className="mt-16">
              <Card className="bg-gradient-to-r from-purple-600/20 to-pink-600/20 border-purple-500/30">
                <CardHeader>
                  <CardTitle className="text-white text-2xl">需要帮助？</CardTitle>
                  <CardDescription className="text-slate-300">
                    我们提供多种方式来帮助您快速上手 AgentMem
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="flex flex-col sm:flex-row gap-4">
                    <Button className="bg-purple-600 hover:bg-purple-700">
                      <ExternalLink className="w-4 h-4 mr-2" />
                      加入社区
                    </Button>
                    <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                      联系支持
                    </Button>
                    <Button variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                      查看示例
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </div>
      
      {/* 页面导航 */}
      <PageNavigation anchors={pageAnchors} className="hidden lg:block" />
      <MobilePageNavigation anchors={pageAnchors} />
      
      {/* 回到顶部按钮 */}
      <ScrollToTopButton />
    </div>
  );
};

/**
 * 文档页面
 * 使用Suspense包裹内容组件以解决预渲染问题
 */
export default function DocsPage() {
  return (
    <Suspense fallback={<div className="text-white text-center py-20">加载中...</div>}>
      <DocsPageContent />
    </Suspense>
  );
}