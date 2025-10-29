/**
 * 国际化配置和翻译管理
 */

// 支持的语言列表
export const SUPPORTED_LANGUAGES = {
  'zh': {
    name: '中文',
    nativeName: '中文',
    flag: '🇨🇳',
    dir: 'ltr'
  },
  'en': {
    name: 'English',
    nativeName: 'English',
    flag: '🇺🇸',
    dir: 'ltr'
  },
  'ja': {
    name: 'Japanese',
    nativeName: '日本語',
    flag: '🇯🇵',
    dir: 'ltr'
  },
  'ko': {
    name: 'Korean',
    nativeName: '한국어',
    flag: '🇰🇷',
    dir: 'ltr'
  }
} as const;

export type SupportedLanguage = keyof typeof SUPPORTED_LANGUAGES;
export const DEFAULT_LANGUAGE: SupportedLanguage = 'zh';

// 翻译类型定义
export interface TranslationKeys {
  // 通用
  common: {
    loading: string;
    error: string;
    success: string;
    cancel: string;
    confirm: string;
    save: string;
    delete: string;
    edit: string;
    view: string;
    search: string;
    searchPlaceholder: string;
    noResults: string;
    backToTop: string;
    copyCode: string;
    copied: string;
    download: string;
    expand: string;
    collapse: string;
  };
  
  // 导航
  nav: {
    home: string;
    docs: string;
    demo: string;
    about: string;
    pricing: string;
    blog: string;
    support: string;
    github: string;
    language: string;
    theme: string;
  };
  
  // 首页
  home: {
    title: string;
    subtitle: string;
    description: string;
    getStarted: string;
    viewDocs: string;
    learnMore: string;
    features: {
      title: string;
      subtitle: string;
      items: {
        memory: {
          title: string;
          description: string;
        };
        search: {
          title: string;
          description: string;
        };
        reasoning: {
          title: string;
          description: string;
        };
        api: {
          title: string;
          description: string;
        };
        performance: {
          title: string;
          description: string;
        };
        security: {
          title: string;
          description: string;
        };
      };
    };
    stats: {
      users: string;
      stars: string;
      downloads: string;
      uptime: string;
    };
    testimonials: {
      title: string;
      subtitle: string;
    };
    cta: {
      title: string;
      subtitle: string;
      button: string;
    };
  };
  
  // 文档
  docs: {
    title: string;
    subtitle: string;
    quickStart: {
      title: string;
      description: string;
    };
    tutorials: {
      title: string;
      description: string;
    };
    api: {
      title: string;
      description: string;
    };
    examples: {
      title: string;
      description: string;
    };
  };
  
  // 演示
  demo: {
    title: string;
    subtitle: string;
    interactive: {
      title: string;
      description: string;
    };
    examples: {
      title: string;
      description: string;
    };
  };
  
  // 关于
  about: {
    title: string;
    subtitle: string;
    company: {
      title: string;
      description: string;
    };
    mission: {
      title: string;
      description: string;
    };
    team: {
      title: string;
      description: string;
    };
    technology: {
      title: string;
      description: string;
    };
  };
  
  // 定价
  pricing: {
    title: string;
    subtitle: string;
    free: {
      title: string;
      price: string;
      description: string;
      features: string[];
      button: string;
    };
    pro: {
      title: string;
      price: string;
      description: string;
      features: string[];
      button: string;
    };
    enterprise: {
      title: string;
      price: string;
      description: string;
      features: string[];
      button: string;
    };
  };
  
  // 博客
  blog: {
    title: string;
    subtitle: string;
    readMore: string;
    publishedOn: string;
    author: string;
    tags: string;
    categories: string;
  };
  
  // 支持
  support: {
    title: string;
    subtitle: string;
    faq: {
      title: string;
      description: string;
    };
    contact: {
      title: string;
      description: string;
      form: {
        name: string;
        email: string;
        subject: string;
        message: string;
        submit: string;
      };
    };
    community: {
      title: string;
      description: string;
    };
  };
  
  // 页脚
  footer: {
    description: string;
    links: {
      product: string;
      resources: string;
      company: string;
      legal: string;
    };
    copyright: string;
  };
}

// 获取浏览器语言
export function getBrowserLanguage(): SupportedLanguage {
  if (typeof window === 'undefined') return DEFAULT_LANGUAGE;
  
  const browserLang = navigator.language.split('-')[0] as SupportedLanguage;
  return Object.keys(SUPPORTED_LANGUAGES).includes(browserLang) 
    ? browserLang 
    : DEFAULT_LANGUAGE;
}

// 格式化日期
export function formatDate(date: Date, locale: SupportedLanguage): string {
  const localeMap = {
    zh: 'zh-CN',
    en: 'en-US',
    ja: 'ja-JP',
    ko: 'ko-KR'
  };
  
  return new Intl.DateTimeFormat(localeMap[locale], {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  }).format(date);
}

// 格式化数字
export function formatNumber(number: number, locale: SupportedLanguage): string {
  const localeMap = {
    zh: 'zh-CN',
    en: 'en-US',
    ja: 'ja-JP',
    ko: 'ko-KR'
  };
  
  return new Intl.NumberFormat(localeMap[locale]).format(number);
}

// 获取文本方向
export function getTextDirection(locale: SupportedLanguage): 'ltr' | 'rtl' {
  return SUPPORTED_LANGUAGES[locale].dir;
}

// 语言切换工具函数
export function getLanguageFromPath(pathname: string): SupportedLanguage {
  const segments = pathname.split('/');
  const langSegment = segments[1];
  
  if (Object.keys(SUPPORTED_LANGUAGES).includes(langSegment)) {
    return langSegment as SupportedLanguage;
  }
  
  return DEFAULT_LANGUAGE;
}

export function removeLanguageFromPath(pathname: string): string {
  const segments = pathname.split('/');
  const langSegment = segments[1];
  
  if (Object.keys(SUPPORTED_LANGUAGES).includes(langSegment)) {
    return '/' + segments.slice(2).join('/');
  }
  
  return pathname;
}

export function addLanguageToPath(pathname: string, locale: SupportedLanguage): string {
  if (locale === DEFAULT_LANGUAGE) {
    return pathname;
  }
  
  const cleanPath = removeLanguageFromPath(pathname);
  return `/${locale}${cleanPath}`;
}