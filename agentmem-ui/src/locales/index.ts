import { SupportedLanguage, TranslationKeys } from '@/lib/i18n';
import { zh } from './zh';
import { en } from './en';

/**
 * 翻译数据索引
 */
export const translations: Record<SupportedLanguage, TranslationKeys> = {
  zh,
  en,
  // 日语和韩语暂时使用英语翻译作为占位符
  ja: en,
  ko: en
};

/**
 * 获取指定语言的翻译
 */
export function getTranslation(locale: SupportedLanguage): TranslationKeys {
  return translations[locale] || translations.zh;
}

/**
 * 获取嵌套翻译值
 */
export function getNestedTranslation(
  translations: TranslationKeys,
  key: string
): string {
  const keys = key.split('.');
  let result: unknown = translations;
  
  for (const k of keys) {
    if (result && typeof result === 'object' && k in result) {
      result = (result as Record<string, unknown>)[k];
    } else {
      return key; // 如果找不到翻译，返回原始 key
    }
  }
  
  return typeof result === 'string' ? result : key;
}

/**
 * 翻译函数类型
 */
export type TranslateFunction = (key: string, params?: Record<string, string | number>) => string;

/**
 * 创建翻译函数
 */
export function createTranslateFunction(locale: SupportedLanguage): TranslateFunction {
  const t = getTranslation(locale);
  
  return (key: string, params?: Record<string, string | number>) => {
    let translation = getNestedTranslation(t, key);
    
    // 参数替换
    if (params) {
      Object.entries(params).forEach(([paramKey, value]) => {
        translation = translation.replace(
          new RegExp(`{{${paramKey}}}`, 'g'),
          String(value)
        );
      });
    }
    
    return translation;
  };
}