"use client";

import React, { createContext, useContext, useState, useEffect } from 'react';
import { SupportedLanguage, DEFAULT_LANGUAGE, getBrowserLanguage } from '@/lib/i18n';
import { createTranslateFunction, TranslateFunction } from '@/locales';

interface LanguageContextType {
  currentLanguage: SupportedLanguage;
  setLanguage: (lang: SupportedLanguage) => void;
  t: TranslateFunction;
  // 向后兼容
  language: SupportedLanguage;
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

/**
 * 语言上下文提供者
 */
export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [currentLanguage, setCurrentLanguage] = useState<SupportedLanguage>(DEFAULT_LANGUAGE);

  // 初始化语言设置
  useEffect(() => {
    const savedLanguage = localStorage.getItem('agentmem-language') as SupportedLanguage;
    const browserLanguage = getBrowserLanguage();
    
    // 优先级：保存的语言 > 浏览器语言 > 默认语言
    const initialLanguage = savedLanguage || browserLanguage;
    setCurrentLanguage(initialLanguage);
  }, []);

  // 保存语言设置并更新文档属性
  useEffect(() => {
    localStorage.setItem('agentmem-language', currentLanguage);
    document.documentElement.lang = currentLanguage;
    document.documentElement.dir = 'ltr'; // 所有支持的语言都是从左到右的
  }, [currentLanguage]);

  // 创建翻译函数
  const t = createTranslateFunction(currentLanguage);

  const setLanguage = (lang: SupportedLanguage) => {
    setCurrentLanguage(lang);
    
    // 触发自定义事件，通知其他组件语言已更改
    window.dispatchEvent(new CustomEvent('languageChange', {
      detail: { language: lang }
    }));
  };

  const value: LanguageContextType = {
    currentLanguage,
    setLanguage,
    t,
    // 向后兼容
    language: currentLanguage
  };

  return (
    <LanguageContext.Provider value={value}>
      {children}
    </LanguageContext.Provider>
  );
}

/**
 * 使用语言上下文的 Hook
 */
export function useLanguage() {
  const context = useContext(LanguageContext);
  if (context === undefined) {
    throw new Error('useLanguage must be used within a LanguageProvider');
  }
  return context;
}

/**
 * 语言切换 Hook
 */
export function useLanguageSwitcher() {
  const { currentLanguage, setLanguage } = useLanguage();
  
  const switchLanguage = (targetLanguage?: SupportedLanguage) => {
    if (targetLanguage) {
      setLanguage(targetLanguage);
    } else {
      // 如果没有指定目标语言，则在中英文之间切换
      const nextLanguage = currentLanguage === 'zh' ? 'en' : 'zh';
      setLanguage(nextLanguage);
    }
  };
  
  return {
    currentLanguage,
    switchLanguage,
    setLanguage
  };
}

/**
 * 翻译 Hook
 */
export function useTranslation() {
  const { t, currentLanguage } = useLanguage();
  
  return {
    t,
    language: currentLanguage,
    // 便捷的翻译函数
    translate: t
  };
}