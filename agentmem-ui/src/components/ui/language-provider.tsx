"use client";

import React, { createContext, useContext, useState, useEffect } from 'react';

type Language = 'zh' | 'en';

interface LanguageContextType {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string) => string;
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

// 翻译字典
const translations = {
  zh: {
    // 导航
    'nav.features': '特性',
    'nav.architecture': '架构',
    'nav.demo': '演示',
    'nav.docs': '文档',
    'nav.github': 'GitHub',
    'nav.menu': '菜单',
    
    // 首页
    'hero.title': '智能记忆管理',
    'hero.subtitle': '新时代',
    'hero.description': 'AgentMem 是基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎，为 AI 应用提供高性能、可扩展的记忆存储与检索解决方案。',
    'hero.getStarted': '立即开始',
    'hero.viewDocs': '查看文档',
    
    // 统计数据
    'stats.modules': '核心模块',
    'stats.availability': '可用性',
    'stats.responseTime': '响应时间',
    'stats.developers': '开发者',
    
    // 特性
    'features.title': '核心特性',
    'features.subtitle': '基于现代技术栈构建，为 AI 应用提供企业级的记忆管理解决方案',
    'features.ai.title': '智能推理引擎',
    'features.ai.description': '集成 DeepSeek 推理引擎，提供先进的语义理解和智能检索能力，让记忆管理更加智能化。',
    'features.ai.badge': 'AI 驱动',
    'features.performance.title': '高性能架构',
    'features.performance.description': '基于 Rust 构建的高性能核心，支持并发处理，毫秒级响应时间，满足大规模应用需求。',
    'features.performance.badge': '<1ms 响应',
    'features.security.title': '企业级安全',
    'features.security.description': '内置多层安全机制，支持数据加密、访问控制和审计日志，确保企业数据安全。',
    'features.security.badge': '军用级加密',
    'features.storage.title': '多存储支持',
    'features.storage.description': '支持多种存储后端，包括 PostgreSQL、Redis、Qdrant 等，灵活适配不同业务场景。',
    'features.storage.badge': '5+ 存储引擎',
    'features.modular.title': '模块化设计',
    'features.modular.description': '13 个专业化 Crate 模块，清晰的职责分离，支持按需集成和自定义扩展。',
    'features.modular.badge': '13 个模块',
    'features.compatible.title': 'API 兼容',
    'features.compatible.description': '100% Mem0 API 兼容，无缝迁移现有应用，同时提供更强大的扩展功能。',
    'features.compatible.badge': '100% 兼容',
    
    // 搜索
    'search.placeholder': '搜索文档、API 和示例...',
    'search.noResults': '未找到相关结果',
    'search.docs': '文档',
    'search.api': 'API',
    'search.examples': '示例',
    
    // 主题
    'theme.light': '浅色模式',
    'theme.dark': '深色模式',
    'theme.toggle': '切换主题',
    
    // FAQ
    'faq.title': '常见问题',
    'faq.subtitle': '找到关于 AgentMem 的常见问题解答，如果您有其他问题，请随时联系我们的支持团队。',
    'faq.noAnswer': '没有找到您要的答案？',
    'faq.supportDescription': '我们的技术支持团队随时为您提供帮助',
    'faq.contactSupport': '联系支持',
    'faq.joinCommunity': '加入社区',
  },
  en: {
    // Navigation
    'nav.features': 'Features',
    'nav.architecture': 'Architecture',
    'nav.demo': 'Demo',
    'nav.docs': 'Docs',
    'nav.github': 'GitHub',
    'nav.menu': 'Menu',
    
    // Hero
    'hero.title': 'Intelligent Memory Management',
    'hero.subtitle': 'New Era',
    'hero.description': 'AgentMem is a next-generation intelligent memory management platform built on Rust, integrating DeepSeek inference engine to provide high-performance, scalable memory storage and retrieval solutions for AI applications.',
    'hero.getStarted': 'Get Started',
    'hero.viewDocs': 'View Docs',
    
    // Stats
    'stats.modules': 'Core Modules',
    'stats.availability': 'Availability',
    'stats.responseTime': 'Response Time',
    'stats.developers': 'Developers',
    
    // Features
    'features.title': 'Core Features',
    'features.subtitle': 'Built on modern technology stack, providing enterprise-grade memory management solutions for AI applications',
    'features.ai.title': 'AI Inference Engine',
    'features.ai.description': 'Integrated DeepSeek inference engine provides advanced semantic understanding and intelligent retrieval capabilities, making memory management more intelligent.',
    'features.ai.badge': 'AI Powered',
    'features.performance.title': 'High Performance',
    'features.performance.description': 'High-performance core built on Rust, supporting concurrent processing with millisecond response times for large-scale applications.',
    'features.performance.badge': '<1ms Response',
    'features.security.title': 'Enterprise Security',
    'features.security.description': 'Built-in multi-layer security mechanisms supporting data encryption, access control, and audit logs to ensure enterprise data security.',
    'features.security.badge': 'Military Grade',
    'features.storage.title': 'Multi-Storage',
    'features.storage.description': 'Supports multiple storage backends including PostgreSQL, Redis, Qdrant, etc., flexibly adapting to different business scenarios.',
    'features.storage.badge': '5+ Storage Engines',
    'features.modular.title': 'Modular Design',
    'features.modular.description': '13 specialized Crate modules with clear separation of responsibilities, supporting on-demand integration and custom extensions.',
    'features.modular.badge': '13 Modules',
    'features.compatible.title': 'API Compatible',
    'features.compatible.description': '100% Mem0 API compatible, seamlessly migrate existing applications while providing more powerful extension features.',
    'features.compatible.badge': '100% Compatible',
    
    // Search
    'search.placeholder': 'Search docs, API and examples...',
    'search.noResults': 'No results found',
    'search.docs': 'Docs',
    'search.api': 'API',
    'search.examples': 'Examples',
    
    // Theme
    'theme.light': 'Light Mode',
    'theme.dark': 'Dark Mode',
    'theme.toggle': 'Toggle Theme',
    
    // FAQ
    'faq.title': 'Frequently Asked Questions',
    'faq.subtitle': 'Find answers to common questions about AgentMem. If you have other questions, please feel free to contact our support team.',
    'faq.noAnswer': 'Didn\'t find what you\'re looking for?',
    'faq.supportDescription': 'Our technical support team is here to help you',
    'faq.contactSupport': 'Contact Support',
    'faq.joinCommunity': 'Join Community',
  }
};

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [language, setLanguage] = useState<Language>('zh');

  // 从 localStorage 读取语言设置
  useEffect(() => {
    const savedLanguage = localStorage.getItem('agentmem-language') as Language;
    if (savedLanguage && (savedLanguage === 'zh' || savedLanguage === 'en')) {
      setLanguage(savedLanguage);
    } else {
      // 检测浏览器语言
      const browserLanguage = navigator.language.toLowerCase();
      if (browserLanguage.startsWith('zh')) {
        setLanguage('zh');
      } else {
        setLanguage('en');
      }
    }
  }, []);

  // 保存语言设置到 localStorage
  const handleSetLanguage = (lang: Language) => {
    setLanguage(lang);
    localStorage.setItem('agentmem-language', lang);
  };

  // 翻译函数
  const t = (key: string): string => {
    return translations[language][key as keyof typeof translations[typeof language]] || key;
  };

  return (
    <LanguageContext.Provider value={{ language, setLanguage: handleSetLanguage, t }}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (context === undefined) {
    throw new Error('useLanguage must be used within a LanguageProvider');
  }
  return context;
}

// 语言切换组件
export function LanguageToggle() {
  const { language, setLanguage } = useLanguage();

  return (
    <button
      onClick={() => setLanguage(language === 'zh' ? 'en' : 'zh')}
      className="flex items-center space-x-2 px-3 py-2 rounded-md text-sm font-medium text-slate-300 hover:text-white hover:bg-slate-800 transition-colors"
      title="切换语言 / Switch Language"
    >
      <span className="text-xs font-mono">
        {language === 'zh' ? '中' : 'EN'}
      </span>
    </button>
  );
}