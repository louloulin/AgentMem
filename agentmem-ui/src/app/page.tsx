"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { ThemeToggle } from "@/components/ui/theme-toggle";
import { GlobalSearch, SearchTrigger } from "@/components/ui/search";
import { LanguageSwitcher, CompactLanguageSwitcher } from "@/components/ui/language-switcher";
import { FadeIn, SlideIn, FloatingCard, GradientText, TypeWriter } from "@/components/ui/animations";
import { useLanguage } from "@/contexts/language-context";
import { Brain, Zap, Shield, Database, Cpu, Network, Code, Rocket, Github, Menu, X, Star, Users, TrendingUp, Award, CheckCircle, Quote, ArrowRight, Building, Globe } from "lucide-react";
import Link from "next/link";
import { useState, useEffect } from "react";
import Head from "next/head";
import { Target } from "lucide-react";

/**
 * 主页组件 - 展示AgentMem的核心特性和优势
 */
export default function HomePage() {
  const { t } = useLanguage();
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isSearchOpen, setIsSearchOpen] = useState(false);

  /**
   * 切换移动端菜单显示状态
   */
  const toggleMobileMenu = () => {
    setIsMobileMenuOpen(!isMobileMenuOpen);
  };

  // 键盘快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsSearchOpen(true);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  // 首页结构化数据
  const websiteData = {
    "@context": "https://schema.org",
    "@type": "WebSite",
    "name": "AgentMem",
    "url": "https://agentmem.ai",
    "description": "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎",
    "inLanguage": "zh-CN",
    "potentialAction": {
      "@type": "SearchAction",
      "target": {
        "@type": "EntryPoint",
        "urlTemplate": "https://agentmem.ai/search?q={search_term_string}"
      },
      "query-input": "required name=search_term_string"
    },
    "publisher": {
      "@type": "Organization",
      "name": "AgentMem Team",
      "url": "https://agentmem.ai"
    }
  };

  const webPageData = {
    "@context": "https://schema.org",
    "@type": "WebPage",
    "@id": "https://agentmem.ai/#webpage",
    "url": "https://agentmem.ai",
    "name": "AgentMem - 智能记忆管理平台",
    "description": "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎。为 AI 代理提供强大的记忆能力，支持语义搜索、智能推理和实时学习。",
    "inLanguage": "zh-CN",
    "isPartOf": {
      "@type": "WebSite",
      "@id": "https://agentmem.ai/#website"
    },
    "datePublished": "2024-01-15",
    "dateModified": "2024-01-15",
    "breadcrumb": {
      "@type": "BreadcrumbList",
      "itemListElement": [
        {
          "@type": "ListItem",
          "position": 1,
          "name": "首页",
          "item": "https://agentmem.ai"
        }
      ]
    },
    "mainEntity": {
      "@type": "SoftwareApplication",
      "name": "AgentMem",
      "applicationCategory": "DeveloperApplication",
      "operatingSystem": "Cross-platform",
      "programmingLanguage": "Rust"
    }
  };
  return (
    <>
      <Head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(websiteData),
          }}
        />
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(webPageData),
          }}
        />
      </Head>
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 text-white">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-[1400px] mx-auto px-6 lg:px-8">
          <div className="flex items-center h-16">
            {/* Logo区域 - 左侧固定 */}
            <div className="flex items-center flex-shrink-0 min-w-[180px]">
              <Link href="/" className="flex items-center">
                <Brain className="h-8 w-8 text-purple-400 animate-pulse-glow" />
                <span className="ml-2 text-xl font-bold text-white">AgentMem</span>
              </Link>
            </div>
            
            {/* 中间菜单区域 */}
            <div className="hidden lg:flex items-center justify-center flex-1 space-x-1">
              <Link href="#features" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                功能
              </Link>
              <Link href="#architecture" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                架构
              </Link>
              <Link href="/solutions" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                {t('nav.solutions')}
              </Link>
              <Link href="/admin" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                Admin
              </Link>
              <Link href="/demo" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                {t('nav.demo')}
              </Link>
              <Link href="/docs" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                {t('nav.docs')}
              </Link>
              <Link href="/pricing" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                {t('nav.pricing')}
              </Link>
              <Link href="/blog" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                {t('nav.blog')}
              </Link>
              <Link href="/support" className="text-slate-300 hover:text-white transition-colors px-4 py-2 rounded-md hover:bg-slate-800/50 text-sm whitespace-nowrap">
                {t('nav.support')}
              </Link>
            </div>
            
            {/* 右侧工具栏 */}
            <div className="hidden lg:flex items-center space-x-3 flex-shrink-0 min-w-[240px] justify-end">
              <SearchTrigger />
              <LanguageSwitcher />
              <ThemeToggle />
              <Button variant="outline" size="sm" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white transition-all duration-300">
                <Github className="mr-1 h-3 w-3" />
                <span className="hidden xl:inline">{t('nav.github')}</span>
              </Button>
            </div>
            {/* 移动端菜单按钮 */}
            <div className="lg:hidden flex items-center space-x-2">
              <CompactLanguageSwitcher />
              <ThemeToggle />
              <Button 
                variant="outline" 
                size="sm" 
                className="border-slate-600 text-slate-300 hover:bg-slate-800"
                onClick={toggleMobileMenu}
                aria-label="Toggle mobile menu"
              >
                {isMobileMenuOpen ? (
                  <X className="w-4 h-4" />
                ) : (
                  <Menu className="w-4 h-4" />
                )}
              </Button>
            </div>
          </div>
        </div>
        
        {/* 移动端菜单 */}
        {isMobileMenuOpen && (
          <div className="lg:hidden bg-slate-900/95 backdrop-blur-sm border-t border-slate-800">
            <div className="px-4 py-4 space-y-4">
              <SearchTrigger />
              <div className="flex flex-col space-y-3">
                <Link 
                  href="#features" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  功能
                </Link>
                <Link 
                  href="#architecture" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  架构
                </Link>
                <Link 
                  href="/solutions" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  {t('nav.solutions')}
                </Link>
                <Link 
                  href="/demo" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  {t('nav.demo')}
                </Link>
                <Link 
                  href="/docs" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  {t('nav.docs')}
                </Link>
                <Link 
                  href="/pricing" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  {t('nav.pricing')}
                </Link>
                <Link 
                  href="/blog" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  {t('nav.blog')}
                </Link>
                <Link 
                  href="/support" 
                  className="text-slate-300 hover:text-white transition-colors py-2 px-3 rounded-lg hover:bg-slate-800"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  {t('nav.support')}
                </Link>
                <Button 
                  variant="outline" 
                  className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white transition-all duration-300 w-full justify-start"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  <Github className="mr-2 h-4 w-4" />
                  {t('nav.github')}
                </Button>
              </div>
            </div>
          </div>
        )}
      </nav>

      {/* 英雄区域 */}
      <section className="relative overflow-hidden">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-24">
          <div className="text-center px-4">
            <FadeIn>
              <h1 className="text-3xl sm:text-4xl md:text-6xl font-bold text-white mb-6 leading-tight">
                <TypeWriter text={t('home.title')} speed={100} />
                <br className="hidden sm:block" />
                <span className="sm:hidden"> </span>
                <GradientText className="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-pink-400">
                  {t('home.subtitle')}
                </GradientText>
              </h1>
            </FadeIn>
            <SlideIn direction="up" delay={300}>
              <p className="text-xl text-slate-300 mb-8 max-w-3xl mx-auto">
                {t('home.description')}
              </p>
            </SlideIn>
            <SlideIn direction="up" delay={600}>
              <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
                <Link href="/admin">
                  <Button size="lg" className="bg-purple-600 hover:bg-purple-700 text-white transition-all duration-300 hover:scale-105 w-full sm:w-auto">
                    <Rocket className="mr-2 h-5 w-5" />
                    进入 Admin Dashboard
                  </Button>
                </Link>
                <Link href="/docs">
                  <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800 transition-all duration-300 w-full sm:w-auto">
                    <Code className="mr-2 h-5 w-5" />
                    {t('home.viewDocs')}
                  </Button>
                </Link>
              </div>
            </SlideIn>
            {/* 统计数据 */}
            <SlideIn direction="up" delay={900}>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 sm:gap-8 max-w-4xl mx-auto">
                <div className="text-center p-4">
                  <div className="text-2xl sm:text-3xl font-bold text-purple-400 mb-2">
                    <TypeWriter text="13" speed={200} />
                  </div>
                  <div className="text-slate-400 text-sm sm:text-base">{t('home.stats.users')}</div>
                </div>
                <div className="text-center p-4">
                  <div className="text-2xl sm:text-3xl font-bold text-purple-400 mb-2">
                    <TypeWriter text="99.9%" speed={50} />
                  </div>
                  <div className="text-slate-400 text-sm sm:text-base">{t('home.stats.uptime')}</div>
                </div>
                <div className="text-center p-4">
                  <div className="text-2xl sm:text-3xl font-bold text-purple-400 mb-2">
                    <TypeWriter text="<1ms" speed={100} />
                  </div>
                  <div className="text-slate-400 text-sm sm:text-base">响应时间</div>
                </div>
                <div className="text-center p-4">
                  <div className="text-2xl sm:text-3xl font-bold text-purple-400 mb-2">
                    <TypeWriter text="1000+" speed={30} />
                  </div>
                  <div className="text-slate-400 text-sm sm:text-base">{t('home.stats.downloads')}</div>
                </div>
              </div>
            </SlideIn>
          </div>
        </div>
        {/* 背景装饰 */}
        <div className="absolute inset-0 overflow-hidden pointer-events-none">
          <div className="absolute -top-40 -right-40 w-80 h-80 bg-purple-500/20 rounded-full blur-3xl animate-float"></div>
          <div className="absolute -bottom-40 -left-40 w-80 h-80 bg-pink-500/20 rounded-full blur-3xl animate-float" style={{animationDelay: '2s'}}></div>
        </div>
      </section>

      {/* 解决的问题 */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <FadeIn>
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold text-white mb-4">
                <GradientText>为什么需要 AI Agent 记忆系统？</GradientText>
              </h2>
              <p className="text-xl text-slate-300">传统 LLM 应用面临的核心挑战</p>
            </div>
          </FadeIn>

          {/* 问题对比 */}
          <div className="grid lg:grid-cols-2 gap-8 mb-16">
            {/* 传统方案的问题 */}
            <SlideIn direction="left" delay={100}>
              <Card className="bg-red-900/20 border-red-500/30 h-full">
                <CardHeader>
                  <div className="flex items-center mb-4">
                    <div className="w-12 h-12 bg-red-500/20 rounded-full flex items-center justify-center mr-4">
                      <X className="w-6 h-6 text-red-400" />
                    </div>
                    <CardTitle className="text-white text-2xl">传统 LLM 的局限</CardTitle>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-red-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">无状态交互</h4>
                      <p className="text-slate-300 text-sm">每次对话都是全新开始，无法记住用户信息</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-red-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">上下文窗口限制</h4>
                      <p className="text-slate-300 text-sm">受限于 4K-128K tokens，无法处理长期历史</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-red-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">成本高昂</h4>
                      <p className="text-slate-300 text-sm">每次都要传输完整历史，Token 消耗巨大</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-red-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">缺乏个性化</h4>
                      <p className="text-slate-300 text-sm">无法根据用户历史提供定制化服务</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-red-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">跨会话遗忘</h4>
                      <p className="text-slate-300 text-sm">无法在不同会话间保持记忆连续性</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </SlideIn>

            {/* AgentMem 解决方案 */}
            <SlideIn direction="right" delay={100}>
              <Card className="bg-green-900/20 border-green-500/30 h-full">
                <CardHeader>
                  <div className="flex items-center mb-4">
                    <div className="w-12 h-12 bg-green-500/20 rounded-full flex items-center justify-center mr-4">
                      <CheckCircle className="w-6 h-6 text-green-400" />
                    </div>
                    <CardTitle className="text-white text-2xl">AgentMem 解决方案</CardTitle>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">持久化记忆</h4>
                      <p className="text-slate-300 text-sm">跨会话保存用户信息和偏好，永不遗忘</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">无限容量</h4>
                      <p className="text-slate-300 text-sm">支持千万级记忆存储，不受窗口限制</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">成本降低 99%</h4>
                      <p className="text-slate-300 text-sm">智能检索相关记忆，仅传输必要信息</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">智能个性化</h4>
                      <p className="text-slate-300 text-sm">基于历史记忆的定制化回答和推荐</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold mb-1">毫秒级检索</h4>
                      <p className="text-slate-300 text-sm">语义搜索响应时间 &lt; 100ms，准确率 95%+</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </SlideIn>
          </div>

          {/* 实际案例对比 */}
          <div className="bg-gradient-to-r from-slate-800/50 to-slate-700/50 rounded-2xl p-8 border border-slate-600">
            <div className="text-center mb-8">
              <h3 className="text-2xl font-bold text-white mb-2">实际效果对比</h3>
              <p className="text-slate-300">看看有无记忆系统的巨大差异</p>
            </div>
            
            <div className="grid lg:grid-cols-2 gap-8">
              {/* 无记忆系统 */}
              <div className="bg-slate-900/50 rounded-lg p-6 border border-red-500/30">
                <div className="flex items-center mb-4">
                  <X className="w-5 h-5 text-red-400 mr-2" />
                  <h4 className="text-white font-semibold">没有记忆系统</h4>
                </div>
                <div className="space-y-3 text-sm">
                  <div className="bg-blue-900/30 p-3 rounded border-l-2 border-blue-400">
                    <span className="text-slate-400">用户:</span>
                    <span className="text-white ml-2">我叫张三，喜欢咖啡</span>
                  </div>
                  <div className="bg-purple-900/30 p-3 rounded border-l-2 border-purple-400">
                    <span className="text-slate-400">AI:</span>
                    <span className="text-white ml-2">好的，我记住了</span>
                  </div>
                  <div className="text-center text-slate-500 py-2">
                    ⏰ 5分钟后...
                  </div>
                  <div className="bg-blue-900/30 p-3 rounded border-l-2 border-blue-400">
                    <span className="text-slate-400">用户:</span>
                    <span className="text-white ml-2">我喜欢什么饮品？</span>
                  </div>
                  <div className="bg-red-900/30 p-3 rounded border-l-2 border-red-400">
                    <span className="text-slate-400">AI:</span>
                    <span className="text-white ml-2">抱歉，我不记得您的偏好...</span>
                  </div>
                </div>
              </div>

              {/* 使用 AgentMem */}
              <div className="bg-slate-900/50 rounded-lg p-6 border border-green-500/30">
                <div className="flex items-center mb-4">
                  <CheckCircle className="w-5 h-5 text-green-400 mr-2" />
                  <h4 className="text-white font-semibold">使用 AgentMem</h4>
                </div>
                <div className="space-y-3 text-sm">
                  <div className="bg-blue-900/30 p-3 rounded border-l-2 border-blue-400">
                    <span className="text-slate-400">用户:</span>
                    <span className="text-white ml-2">我叫张三，喜欢咖啡</span>
                  </div>
                  <div className="bg-purple-900/30 p-3 rounded border-l-2 border-purple-400">
                    <span className="text-slate-400">AI:</span>
                    <span className="text-white ml-2">好的，我已记住：您的名字是张三，偏好饮品是咖啡</span>
                  </div>
                  <div className="text-center text-slate-500 py-2">
                    ⏰ 一周后，新会话...
                  </div>
                  <div className="bg-blue-900/30 p-3 rounded border-l-2 border-blue-400">
                    <span className="text-slate-400">用户:</span>
                    <span className="text-white ml-2">我喜欢什么饮品？</span>
                  </div>
                  <div className="bg-green-900/30 p-3 rounded border-l-2 border-green-400">
                    <span className="text-slate-400">AI:</span>
                    <span className="text-white ml-2">根据您的偏好记录，您喜欢咖啡。需要我推荐一些咖啡店吗？</span>
                  </div>
                </div>
              </div>
            </div>

            {/* 数据对比 */}
            <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="text-center p-4 bg-slate-900/30 rounded-lg">
                <div className="text-3xl font-bold text-green-400 mb-2">99%↓</div>
                <div className="text-white font-semibold mb-1">成本降低</div>
                <div className="text-slate-400 text-sm">无需每次传输全量历史</div>
              </div>
              <div className="text-center p-4 bg-slate-900/30 rounded-lg">
                <div className="text-3xl font-bold text-purple-400 mb-2">20-50x</div>
                <div className="text-white font-semibold mb-1">性能提升</div>
                <div className="text-slate-400 text-sm">毫秒级智能检索</div>
              </div>
              <div className="text-center p-4 bg-slate-900/30 rounded-lg">
                <div className="text-3xl font-bold text-blue-400 mb-2">40%+</div>
                <div className="text-white font-semibold mb-1">满意度提升</div>
                <div className="text-slate-400 text-sm">个性化服务体验</div>
              </div>
            </div>
          </div>

          {/* 核心能力预览 */}
          <div className="mt-16 text-center">
            <SlideIn direction="up" delay={300}>
              <div className="inline-flex items-center bg-purple-900/30 rounded-full px-6 py-3 border border-purple-500/30">
                <Target className="w-5 h-5 text-purple-400 mr-2" />
                <span className="text-white font-semibold">AgentMem 核心能力</span>
                <ArrowRight className="w-5 h-5 text-purple-400 ml-2 animate-pulse" />
              </div>
            </SlideIn>
          </div>
        </div>
      </section>

      {/* 核心特性 */}
      <section id="features" className="py-20 px-4 sm:px-6 lg:px-8 relative">
        <div className="max-w-7xl mx-auto">
          <FadeIn>
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold text-white mb-4">
                <GradientText>{t('home.features.title')}</GradientText>
              </h2>
              <p className="text-xl text-slate-300">{t('home.features.subtitle')}</p>
            </div>
          </FadeIn>
          
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 lg:gap-8 px-4">
            {/* 智能推理引擎 */}
            <SlideIn direction="up" delay={100}>
              <FloatingCard className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 group">
                <CardHeader>
                  <div className="p-2 bg-purple-500/20 rounded-lg group-hover:bg-purple-500/30 transition-colors w-fit">
                    <Brain className="h-8 w-8 text-purple-400 group-hover:scale-110 transition-transform" />
                  </div>
                  <CardTitle className="text-white mt-4">智能推理引擎</CardTitle>
                  <CardDescription className="text-slate-300">
                    DeepSeek 驱动的事实提取和记忆决策
                  </CardDescription>
                </CardHeader>
                <CardContent className="text-slate-300">
                  <ul className="space-y-2">
                    <li>• 自动事实提取</li>
                    <li>• 智能冲突解决</li>
                    <li>• 上下文感知搜索</li>
                    <li>• 动态重要性评估</li>
                  </ul>
                  <div className="mt-4 flex items-center text-sm text-purple-400">
                    <Zap className="h-4 w-4 mr-1" />
                    AI 驱动
                  </div>
                </CardContent>
              </FloatingCard>
            </SlideIn>

            {/* 模块化架构 */}
            <SlideIn direction="up" delay={200}>
              <FloatingCard className="bg-slate-800/50 border-slate-700 hover:border-blue-500/50 transition-all duration-300 group">
                <CardHeader>
                  <div className="p-2 bg-blue-500/20 rounded-lg group-hover:bg-blue-500/30 transition-colors w-fit">
                    <Cpu className="h-8 w-8 text-blue-400 group-hover:scale-110 transition-transform" />
                  </div>
                  <CardTitle className="text-white mt-4">模块化架构</CardTitle>
                  <CardDescription className="text-slate-300">
                    13个专业化 Crate，职责清晰分离
                  </CardDescription>
                </CardHeader>
                <CardContent className="text-slate-300">
                  <ul className="space-y-2">
                    <li>• 核心记忆引擎</li>
                    <li>• 智能处理模块</li>
                    <li>• 多存储后端</li>
                    <li>• LLM 集成层</li>
                  </ul>
                  <div className="mt-4 flex items-center text-sm text-blue-400">
                    <Code className="h-4 w-4 mr-1" />
                    13 个模块
                  </div>
                </CardContent>
              </FloatingCard>
            </SlideIn>

            {/* 高性能架构 */}
            <SlideIn direction="up" delay={300}>
              <FloatingCard className="bg-slate-800/50 border-slate-700 hover:border-yellow-500/50 transition-all duration-300 group">
                <CardHeader>
                  <div className="p-2 bg-yellow-500/20 rounded-lg group-hover:bg-yellow-500/30 transition-colors w-fit">
                    <Zap className="h-8 w-8 text-yellow-400 group-hover:scale-110 transition-transform" />
                  </div>
                  <CardTitle className="text-white mt-4">高性能架构</CardTitle>
                  <CardDescription className="text-slate-300">
                    基于 Tokio 的异步优先设计
                  </CardDescription>
                </CardHeader>
                <CardContent className="text-slate-300">
                  <ul className="space-y-2">
                    <li>• 多级缓存系统</li>
                    <li>• 批量处理优化</li>
                    <li>• 实时性能监控</li>
                    <li>• 自适应优化</li>
                  </ul>
                  <div className="mt-4 flex items-center text-sm text-yellow-400">
                    <Cpu className="h-4 w-4 mr-1" />
                    &lt;1ms 响应
                  </div>
                </CardContent>
              </FloatingCard>
            </SlideIn>

            {/* 多存储后端 */}
            <SlideIn direction="up" delay={400}>
              <FloatingCard className="bg-slate-800/50 border-slate-700 hover:border-green-500/50 transition-all duration-300 group">
                <CardHeader>
                  <div className="p-2 bg-green-500/20 rounded-lg group-hover:bg-green-500/30 transition-colors w-fit">
                    <Database className="h-8 w-8 text-green-400 group-hover:scale-110 transition-transform" />
                  </div>
                  <CardTitle className="text-white mt-4">多存储后端</CardTitle>
                  <CardDescription className="text-slate-300">
                    支持8+种向量数据库和图数据库
                  </CardDescription>
                </CardHeader>
                <CardContent className="text-slate-300">
                  <ul className="space-y-2">
                    <li>• Pinecone, Qdrant, Chroma</li>
                    <li>• PostgreSQL, Redis</li>
                    <li>• Neo4j, Memgraph</li>
                    <li>• 内存存储优化</li>
                  </ul>
                  <div className="mt-4 flex items-center text-sm text-green-400">
                    <Database className="h-4 w-4 mr-1" />
                    8+ 存储引擎
                  </div>
                </CardContent>
              </FloatingCard>
            </SlideIn>

            {/* 企业级特性 */}
            <SlideIn direction="up" delay={500}>
              <FloatingCard className="bg-slate-800/50 border-slate-700 hover:border-red-500/50 transition-all duration-300 group">
                <CardHeader>
                  <div className="p-2 bg-red-500/20 rounded-lg group-hover:bg-red-500/30 transition-colors w-fit">
                    <Shield className="h-8 w-8 text-red-400 group-hover:scale-110 transition-transform" />
                  </div>
                  <CardTitle className="text-white mt-4">企业级特性</CardTitle>
                  <CardDescription className="text-slate-300">
                    生产就绪的安全和可靠性保障
                  </CardDescription>
                </CardHeader>
                <CardContent className="text-slate-300">
                  <ul className="space-y-2">
                    <li>• 类型安全保证</li>
                    <li>• 完整测试覆盖</li>
                    <li>• 分布式支持</li>
                    <li>• 监控和遥测</li>
                  </ul>
                  <div className="mt-4 flex items-center text-sm text-red-400">
                    <Shield className="h-4 w-4 mr-1" />
                    军用级安全
                  </div>
                </CardContent>
              </FloatingCard>
            </SlideIn>

            {/* Mem0 兼容 */}
            <SlideIn direction="up" delay={600}>
              <FloatingCard className="bg-slate-800/50 border-slate-700 hover:border-indigo-500/50 transition-all duration-300 group">
                <CardHeader>
                  <div className="p-2 bg-indigo-500/20 rounded-lg group-hover:bg-indigo-500/30 transition-colors w-fit">
                    <Network className="h-8 w-8 text-indigo-400 group-hover:scale-110 transition-transform" />
                  </div>
                  <CardTitle className="text-white mt-4">Mem0 兼容</CardTitle>
                  <CardDescription className="text-slate-300">
                    100% API 兼容，支持无缝迁移
                  </CardDescription>
                </CardHeader>
                <CardContent className="text-slate-300">
                  <ul className="space-y-2">
                    <li>• 完整 API 兼容</li>
                    <li>• 零代码迁移</li>
                    <li>• 性能提升</li>
                    <li>• 扩展功能</li>
                  </ul>
                  <div className="mt-4 flex items-center text-sm text-indigo-400">
                    <Network className="h-4 w-4 mr-1" />
                    100% 兼容
                  </div>
                </CardContent>
              </FloatingCard>
            </SlideIn>
          </div>
        </div>
      </section>

      {/* 用户案例和统计 */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <FadeIn>
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold text-white mb-4">
                <GradientText>全球企业的信赖之选</GradientText>
              </h2>
              <p className="text-xl text-slate-300">已为全球 1000+ 企业提供智能记忆管理服务</p>
            </div>
          </FadeIn>

          {/* 统计数据展示 */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 mb-16">
            <SlideIn direction="up" delay={100}>
              <div className="text-center p-6 bg-slate-800/30 rounded-lg border border-slate-700">
                <TrendingUp className="w-8 h-8 text-green-400 mx-auto mb-4" />
                <div className="text-3xl font-bold text-green-400 mb-2">500%</div>
                <div className="text-slate-300">性能提升</div>
                <div className="text-sm text-slate-400 mt-1">相比传统方案</div>
              </div>
            </SlideIn>
            <SlideIn direction="up" delay={200}>
              <div className="text-center p-6 bg-slate-800/30 rounded-lg border border-slate-700">
                <Users className="w-8 h-8 text-blue-400 mx-auto mb-4" />
                <div className="text-3xl font-bold text-blue-400 mb-2">10M+</div>
                <div className="text-slate-300">API 调用</div>
                <div className="text-sm text-slate-400 mt-1">每日处理量</div>
              </div>
            </SlideIn>
            <SlideIn direction="up" delay={300}>
              <div className="text-center p-6 bg-slate-800/30 rounded-lg border border-slate-700">
                <Globe className="w-8 h-8 text-purple-400 mx-auto mb-4" />
                <div className="text-3xl font-bold text-purple-400 mb-2">50+</div>
                <div className="text-slate-300">国家地区</div>
                <div className="text-sm text-slate-400 mt-1">全球服务覆盖</div>
              </div>
            </SlideIn>
            <SlideIn direction="up" delay={400}>
              <div className="text-center p-6 bg-slate-800/30 rounded-lg border border-slate-700">
                <Award className="w-8 h-8 text-yellow-400 mx-auto mb-4" />
                <div className="text-3xl font-bold text-yellow-400 mb-2">99.99%</div>
                <div className="text-slate-300">服务可用性</div>
                <div className="text-sm text-slate-400 mt-1">SLA 保障</div>
              </div>
            </SlideIn>
          </div>

          {/* 客户案例 */}
          <div className="grid md:grid-cols-3 gap-8 mb-16">
            <SlideIn direction="up" delay={100}>
              <Card className="bg-slate-800/50 border-slate-700 p-6">
                <div className="flex items-center mb-4">
                  <Building className="w-8 h-8 text-blue-400 mr-3" />
                  <div>
                    <h3 className="text-white font-semibold">金融科技公司</h3>
                    <p className="text-slate-400 text-sm">智能客服解决方案</p>
                  </div>
                </div>
                <blockquote className="text-slate-300 mb-4">
                  <Quote className="w-4 h-4 text-purple-400 mb-2" />
                  &quot;AgentMem 帮助我们的客服系统记住每个客户的历史对话，客户满意度提升了 85%。&quot;
                </blockquote>
                <div className="flex items-center justify-between">
                  <div className="flex text-yellow-400">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="w-4 h-4 fill-current" />
                    ))}
                  </div>
                  <span className="text-slate-400 text-sm">CTO, FinTech Corp</span>
                </div>
              </Card>
            </SlideIn>
            <SlideIn direction="up" delay={200}>
              <Card className="bg-slate-800/50 border-slate-700 p-6">
                <div className="flex items-center mb-4">
                  <Building className="w-8 h-8 text-green-400 mr-3" />
                  <div>
                    <h3 className="text-white font-semibold">医疗健康平台</h3>
                    <p className="text-slate-400 text-sm">AI 诊断助手</p>
                  </div>
                </div>
                <blockquote className="text-slate-300 mb-4">
                  <Quote className="w-4 h-4 text-purple-400 mb-2" />
                  &quot;通过 AgentMem，我们的 AI 医生能够记住患者的完整病史，诊断准确率提升了 40%。&quot;
                </blockquote>
                <div className="flex items-center justify-between">
                  <div className="flex text-yellow-400">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="w-4 h-4 fill-current" />
                    ))}
                  </div>
                  <span className="text-slate-400 text-sm">首席医疗官, HealthAI</span>
                </div>
              </Card>
            </SlideIn>
            <SlideIn direction="up" delay={300}>
              <Card className="bg-slate-800/50 border-slate-700 p-6">
                <div className="flex items-center mb-4">
                  <Building className="w-8 h-8 text-purple-400 mr-3" />
                  <div>
                    <h3 className="text-white font-semibold">教育科技公司</h3>
                    <p className="text-slate-400 text-sm">个性化学习系统</p>
                  </div>
                </div>
                <blockquote className="text-slate-300 mb-4">
                  <Quote className="w-4 h-4 text-purple-400 mb-2" />
                  &quot;学生的学习进度和偏好都被完美记录，个性化推荐的准确率达到了 95%。&quot;
                </blockquote>
                <div className="flex items-center justify-between">
                  <div className="flex text-yellow-400">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="w-4 h-4 fill-current" />
                    ))}
                  </div>
                  <span className="text-slate-400 text-sm">产品总监, EduTech</span>
                </div>
              </Card>
            </SlideIn>
          </div>

          {/* 产品亮点 */}
          <div className="bg-gradient-to-r from-purple-900/30 to-pink-900/30 rounded-2xl p-8 border border-purple-500/20">
            <div className="text-center mb-12">
              <h3 className="text-3xl font-bold text-white mb-4">为什么选择 AgentMem？</h3>
              <p className="text-xl text-slate-300">领先的技术优势，助力您的 AI 应用更智能</p>
            </div>
            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="text-center">
                <div className="w-16 h-16 bg-purple-600/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Zap className="w-8 h-8 text-purple-400" />
                </div>
                <h4 className="text-white font-semibold mb-2">极致性能</h4>
                <p className="text-slate-300 text-sm">Rust 原生实现，比 Python 方案快 5 倍</p>
              </div>
              <div className="text-center">
                <div className="w-16 h-16 bg-blue-600/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Shield className="w-8 h-8 text-blue-400" />
                </div>
                <h4 className="text-white font-semibold mb-2">企业级安全</h4>
                <p className="text-slate-300 text-sm">端到端加密，SOC 2 合规认证</p>
              </div>
              <div className="text-center">
                <div className="w-16 h-16 bg-green-600/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Network className="w-8 h-8 text-green-400" />
                </div>
                <h4 className="text-white font-semibold mb-2">无缝集成</h4>
                <p className="text-slate-300 text-sm">100% Mem0 兼容，零代码迁移</p>
              </div>
              <div className="text-center">
                <div className="w-16 h-16 bg-yellow-600/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Users className="w-8 h-8 text-yellow-400" />
                </div>
                <h4 className="text-white font-semibold mb-2">专业支持</h4>
                <p className="text-slate-300 text-sm">24/7 技术支持，专属客户成功团队</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* 技术架构 */}
      <section id="architecture" className="py-20 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-white mb-4">技术架构</h2>
            <p className="text-xl text-slate-300">分层模块化设计，支持大规模部署</p>
          </div>
          
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
            <div>
              <h3 className="text-2xl font-bold text-white mb-6">分层架构设计</h3>
              <div className="space-y-4">
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-purple-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">应用层</h4>
                    <p className="text-slate-300 text-sm">HTTP服务器、客户端、兼容层</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-blue-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">业务逻辑层</h4>
                    <p className="text-slate-300 text-sm">智能处理、性能监控、核心引擎</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-green-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">服务层</h4>
                    <p className="text-slate-300 text-sm">LLM集成、嵌入模型、分布式支持</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-yellow-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">数据层</h4>
                    <p className="text-slate-300 text-sm">存储抽象、配置管理</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-red-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">基础设施层</h4>
                    <p className="text-slate-300 text-sm">核心抽象、工具库</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="bg-slate-800/50 p-8 rounded-lg border border-slate-700">
              <h3 className="text-2xl font-bold text-white mb-6">性能指标</h3>
              <div className="grid grid-cols-2 gap-6">
                <div className="text-center">
                  <div className="text-3xl font-bold text-purple-400 mb-2">13</div>
                  <div className="text-slate-300">核心 Crate</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-blue-400 mb-2">100%</div>
                  <div className="text-slate-300">Mem0 兼容</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-green-400 mb-2">8+</div>
                  <div className="text-slate-300">存储后端</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-yellow-400 mb-2">15+</div>
                  <div className="text-slate-300">LLM 提供商</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA 区域 */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl font-bold text-white mb-6">
            准备开始使用 AgentMem？
          </h2>
          <p className="text-xl text-slate-300 mb-8">
            立即体验下一代智能记忆管理平台，为您的 AI 应用提供强大的记忆能力。
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link href="/admin">
              <Button size="lg" className="bg-purple-600 hover:bg-purple-700 text-white w-full sm:w-auto">
                <Rocket className="mr-2 h-5 w-5" />
                进入 Admin Dashboard
              </Button>
            </Link>
            <Link href="/pricing">
              <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800 w-full sm:w-auto">
                查看定价
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* 页脚 */}
      <footer className="border-t border-slate-800 bg-slate-900/50 py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div>
              <div className="flex items-center mb-4">
                <Brain className="h-6 w-6 text-purple-400" />
                <span className="ml-2 text-lg font-bold text-white">AgentMem</span>
              </div>
              <p className="text-slate-400">
                智能记忆管理平台，为 AI 代理提供先进的记忆处理能力。
              </p>
            </div>
            <div>
              <h3 className="text-white font-semibold mb-4">产品</h3>
              <ul className="space-y-2 text-slate-400">
                <li><Link href="#" className="hover:text-white transition-colors">核心引擎</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">智能推理</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">企业版</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">云服务</Link></li>
              </ul>
            </div>
            <div>
              <h3 className="text-white font-semibold mb-4">开发者</h3>
              <ul className="space-y-2 text-slate-400">
                <li><Link href="#" className="hover:text-white transition-colors">API 文档</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">快速开始</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">示例代码</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">GitHub</Link></li>
              </ul>
            </div>
            <div>
              <h3 className="text-white font-semibold mb-4">公司</h3>
              <ul className="space-y-2 text-slate-400">
                <li><Link href="#" className="hover:text-white transition-colors">关于我们</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">博客</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">职业机会</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">联系我们</Link></li>
              </ul>
            </div>
          </div>
          <Separator className="my-8 bg-slate-800" />
          <div className="flex flex-col md:flex-row justify-between items-center">
            <p className="text-slate-400">
              © 2024 AgentMem. All rights reserved.
            </p>
            <div className="flex space-x-6 mt-4 md:mt-0">
              <Link href="#" className="text-slate-400 hover:text-white transition-colors">
                隐私政策
              </Link>
              <Link href="#" className="text-slate-400 hover:text-white transition-colors">
                服务条款
              </Link>
            </div>
          </div>
        </div>
      </footer>
      
      {/* 全站搜索 */}
      <GlobalSearch 
        isOpen={isSearchOpen} 
        onClose={() => setIsSearchOpen(false)} 
      />
      </div>
    </>
  );
}
