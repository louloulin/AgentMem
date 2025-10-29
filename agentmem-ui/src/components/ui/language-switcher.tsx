"use client";

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Badge } from '@/components/ui/badge';
import { Globe, Check } from 'lucide-react';
import { SUPPORTED_LANGUAGES, SupportedLanguage } from '@/lib/i18n';
import { useLanguage } from '@/contexts/language-context';

/**
 * 语言切换器组件
 */
export function LanguageSwitcher() {
  const { currentLanguage, setLanguage, t } = useLanguage();
  const [isOpen, setIsOpen] = useState(false);

  const handleLanguageChange = (locale: SupportedLanguage) => {
    setLanguage(locale);
    setIsOpen(false);
  };

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="h-9 px-3 text-slate-300 hover:text-white hover:bg-slate-800"
        >
          <Globe className="h-4 w-4 mr-2" />
          <span className="hidden sm:inline">
            {SUPPORTED_LANGUAGES[currentLanguage].name}
          </span>
          <span className="sm:hidden">
            {SUPPORTED_LANGUAGES[currentLanguage].flag}
          </span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-48">
        {Object.entries(SUPPORTED_LANGUAGES).map(([locale, info]) => (
          <DropdownMenuItem
            key={locale}
            onClick={() => handleLanguageChange(locale as SupportedLanguage)}
            className="flex items-center justify-between cursor-pointer"
          >
            <div className="flex items-center space-x-2">
              <span className="text-lg">{info.flag}</span>
              <div className="flex flex-col">
                <span className="text-sm font-medium">{info.name}</span>
                <span className="text-xs text-slate-500">{info.nativeName}</span>
              </div>
            </div>
            {currentLanguage === locale && (
              <Check className="h-4 w-4 text-green-500" />
            )}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/**
 * 简化版语言切换器（仅显示标志）
 */
export function CompactLanguageSwitcher() {
  const { currentLanguage, setLanguage } = useLanguage();
  const [isOpen, setIsOpen] = useState(false);

  const handleLanguageChange = (locale: SupportedLanguage) => {
    setLanguage(locale);
    setIsOpen(false);
  };

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0 text-slate-300 hover:text-white hover:bg-slate-800"
        >
          <span className="text-lg">
            {SUPPORTED_LANGUAGES[currentLanguage].flag}
          </span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-40">
        {Object.entries(SUPPORTED_LANGUAGES).map(([locale, info]) => (
          <DropdownMenuItem
            key={locale}
            onClick={() => handleLanguageChange(locale as SupportedLanguage)}
            className="flex items-center justify-between cursor-pointer"
          >
            <div className="flex items-center space-x-2">
              <span className="text-base">{info.flag}</span>
              <span className="text-sm">{info.nativeName}</span>
            </div>
            {currentLanguage === locale && (
              <Check className="h-3 w-3 text-green-500" />
            )}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/**
 * 语言状态指示器
 */
export function LanguageIndicator() {
  const { currentLanguage } = useLanguage();
  const languageInfo = SUPPORTED_LANGUAGES[currentLanguage];

  return (
    <Badge variant="secondary" className="text-xs">
      <span className="mr-1">{languageInfo.flag}</span>
      {languageInfo.name}
    </Badge>
  );
}

/**
 * 移动端语言切换器
 */
export function MobileLanguageSwitcher() {
  const { currentLanguage, setLanguage, t } = useLanguage();

  return (
    <div className="space-y-2">
      <h3 className="text-sm font-medium text-slate-300 mb-3">
        {t('nav.language')}
      </h3>
      <div className="grid grid-cols-2 gap-2">
        {Object.entries(SUPPORTED_LANGUAGES).map(([locale, info]) => (
          <Button
            key={locale}
            variant={currentLanguage === locale ? "default" : "ghost"}
            size="sm"
            onClick={() => setLanguage(locale as SupportedLanguage)}
            className="justify-start h-auto p-3"
          >
            <div className="flex items-center space-x-2">
              <span className="text-lg">{info.flag}</span>
              <div className="text-left">
                <div className="text-xs font-medium">{info.name}</div>
                <div className="text-xs opacity-70">{info.nativeName}</div>
              </div>
            </div>
          </Button>
        ))}
      </div>
    </div>
  );
}

/**
 * 语言切换按钮组
 */
export function LanguageButtonGroup() {
  const { currentLanguage, setLanguage } = useLanguage();

  return (
    <div className="flex items-center space-x-1 bg-slate-800 rounded-lg p-1">
      {Object.entries(SUPPORTED_LANGUAGES).map(([locale, info]) => (
        <Button
          key={locale}
          variant={currentLanguage === locale ? "default" : "ghost"}
          size="sm"
          onClick={() => setLanguage(locale as SupportedLanguage)}
          className="h-8 px-3 text-xs"
        >
          <span className="mr-1">{info.flag}</span>
          {info.name}
        </Button>
      ))}
    </div>
  );
}