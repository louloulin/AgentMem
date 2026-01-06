"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Moon, Sun } from "lucide-react";

/**
 * 主题切换组件 - 支持深色和浅色模式切换
 */
export function ThemeToggle() {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [mounted, setMounted] = useState(false);

  /**
   * 组件挂载后初始化主题
   */
  useEffect(() => {
    setMounted(true);
    const savedTheme = localStorage.getItem('theme') as 'dark' | 'light' || 'dark';
    setTheme(savedTheme);
    document.documentElement.classList.toggle('dark', savedTheme === 'dark');
  }, []);

  /**
   * 切换主题
   */
  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
    document.documentElement.classList.toggle('dark', newTheme === 'dark');
  };

  // 避免服务端渲染不匹配
  if (!mounted) {
    return (
      <Button
        variant="outline"
        size="sm"
        className="border-slate-600 bg-transparent"
        disabled
      >
        <Sun className="h-4 w-4" />
      </Button>
    );
  }

  return (
    <Button
      variant="outline"
      size="sm"
      className="border-slate-600 bg-transparent hover:bg-slate-800 transition-colors"
      onClick={toggleTheme}
      title={theme === 'dark' ? '切换到浅色模式' : '切换到深色模式'}
    >
      {theme === 'dark' ? (
        <Sun className="h-4 w-4 text-yellow-400" />
      ) : (
        <Moon className="h-4 w-4 text-slate-600" />
      )}
    </Button>
  );
}