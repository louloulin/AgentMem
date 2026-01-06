"use client";

import { useState, useEffect, useRef } from "react";
import { Search, X, FileText, Code, Users, Building } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import Link from "next/link";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";

/**
 * 搜索结果项接口
 */
interface SearchResult {
  id: string;
  title: string;
  content: string;
  url: string;
  type: 'page' | 'docs' | 'api' | 'team';
  category: string;
}

/**
 * 全站搜索组件属性
 */
interface GlobalSearchProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * 模拟搜索数据
 */
const searchData: SearchResult[] = [
  {
    id: '1',
    title: 'AgentMem 首页',
    content: '下一代智能记忆管理系统，为 AI 代理提供强大的记忆能力',
    url: '/',
    type: 'page',
    category: '主页'
  },
  {
    id: '2',
    title: '快速开始',
    content: '了解如何快速集成 AgentMem 到您的项目中',
    url: '/docs',
    type: 'docs',
    category: '文档'
  },
  {
    id: '3',
    title: 'API 参考',
    content: 'AgentMem REST API 完整参考文档',
    url: '/docs#api',
    type: 'api',
    category: 'API'
  },
  {
    id: '4',
    title: '在线演示',
    content: '体验 AgentMem 的强大功能和实时演示',
    url: '/demo',
    type: 'page',
    category: '演示'
  },
  {
    id: '5',
    title: '团队介绍',
    content: '了解 AgentMem 背后的优秀团队',
    url: '/about',
    type: 'team',
    category: '关于'
  },
  {
    id: '6',
    title: '定价方案',
    content: '选择适合您需求的 AgentMem 定价方案',
    url: '/pricing',
    type: 'page',
    category: '定价'
  },
  {
    id: '7',
    title: '技术博客',
    content: '最新的技术分享和行业洞察',
    url: '/blog',
    type: 'page',
    category: '博客'
  },
  {
    id: '8',
    title: '支持中心',
    content: '获取技术支持和帮助文档',
    url: '/support',
    type: 'page',
    category: '支持'
  },
  {
    id: '9',
    title: '记忆管理 API',
    content: 'POST /api/memories - 添加新的记忆到系统中',
    url: '/docs#memory-api',
    type: 'api',
    category: 'API'
  },
  {
    id: '10',
    title: '智能搜索 API',
    content: 'GET /api/search - 使用语义搜索查找相关记忆',
    url: '/docs#search-api',
    type: 'api',
    category: 'API'
  }
];

/**
 * 获取类型图标
 */
const getTypeIcon = (type: string) => {
  switch (type) {
    case 'page':
      return <FileText className="h-4 w-4" />;
    case 'docs':
      return <FileText className="h-4 w-4" />;
    case 'api':
      return <Code className="h-4 w-4" />;
    case 'team':
      return <Users className="h-4 w-4" />;
    default:
      return <Building className="h-4 w-4" />;
  }
};

/**
 * 获取类型颜色
 */
const getTypeColor = (type: string) => {
  switch (type) {
    case 'page':
      return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
    case 'docs':
      return 'bg-green-500/20 text-green-400 border-green-500/30';
    case 'api':
      return 'bg-purple-500/20 text-purple-400 border-purple-500/30';
    case 'team':
      return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    default:
      return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  }
};

/**
 * 全站搜索组件
 */
export function GlobalSearch({ isOpen, onClose }: GlobalSearchProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  /**
   * 执行搜索
   */
  const performSearch = (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([]);
      return;
    }

    setIsSearching(true);

    // 模拟搜索延迟
    setTimeout(() => {
      const filteredResults = searchData.filter(item => 
        item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.category.toLowerCase().includes(searchQuery.toLowerCase())
      );
      
      setResults(filteredResults);
      setIsSearching(false);
    }, 300);
  };

  /**
   * 处理搜索输入
   */
  const handleSearch = (value: string) => {
    setQuery(value);
    performSearch(value);
  };

  /**
   * 处理键盘事件
   */
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  };

  /**
   * 处理结果点击
   */
  const handleResultClick = () => {
    onClose();
    setQuery('');
    setResults([]);
  };

  // 当搜索框打开时自动聚焦
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={onClose}>
      <div className="flex items-start justify-center pt-20 px-4">
        <Card 
          className="w-full max-w-2xl bg-slate-900 border-slate-700"
          onClick={(e) => e.stopPropagation()}
        >
          <CardContent className="p-0">
            {/* 搜索输入框 */}
            <div className="flex items-center p-4 border-b border-slate-700">
              <Search className="h-5 w-5 text-slate-400 mr-3" />
              <Input
                ref={inputRef}
                placeholder="搜索文档、API、页面..."
                value={query}
                onChange={(e) => handleSearch(e.target.value)}
                onKeyDown={handleKeyDown}
                className="border-0 bg-transparent text-white placeholder:text-slate-400 focus-visible:ring-0 focus-visible:ring-offset-0"
              />
              <Button
                size="sm"
                variant="ghost"
                onClick={onClose}
                className="ml-2 h-8 w-8 p-0 text-slate-400 hover:text-white"
              >
                <X className="h-4 w-4" />
              </Button>
            </div>

            {/* 搜索结果 */}
            <div className="max-h-96 overflow-y-auto">
              {isSearching ? (
                <div className="flex items-center justify-center py-8">
                  <div className="animate-spin h-6 w-6 border-2 border-purple-400 border-t-transparent rounded-full"></div>
                  <span className="ml-2 text-slate-400">搜索中...</span>
                </div>
              ) : results.length > 0 ? (
                <div className="py-2">
                  {results.map((result) => (
                    <Link
                      key={result.id}
                      href={result.url}
                      onClick={handleResultClick}
                      className="block px-4 py-3 hover:bg-slate-800/50 transition-colors"
                    >
                      <div className="flex items-start gap-3">
                        <div className={`p-2 rounded-lg ${getTypeColor(result.type)}`}>
                          {getTypeIcon(result.type)}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <h3 className="text-white font-medium truncate">
                              {result.title}
                            </h3>
                            <Badge 
                              variant="outline" 
                              className={`text-xs ${getTypeColor(result.type)}`}
                            >
                              {result.category}
                            </Badge>
                          </div>
                          <p className="text-slate-400 text-sm line-clamp-2">
                            {result.content}
                          </p>
                        </div>
                      </div>
                    </Link>
                  ))}
                </div>
              ) : query.trim() ? (
                <div className="flex flex-col items-center justify-center py-8 text-slate-400">
                  <Search className="h-12 w-12 mb-4 opacity-50" />
                  <p>未找到相关结果</p>
                  <p className="text-sm mt-1">尝试使用不同的关键词</p>
                </div>
              ) : (
                <div className="py-4 px-4">
                  <h3 className="text-white font-medium mb-3">快速导航</h3>
                  <div className="grid grid-cols-2 gap-2">
                    {[
                      { name: '文档', url: '/docs', icon: <FileText className="h-4 w-4" /> },
                      { name: '演示', url: '/demo', icon: <Code className="h-4 w-4" /> },
                      { name: '关于', url: '/about', icon: <Users className="h-4 w-4" /> },
                      { name: '定价', url: '/pricing', icon: <Building className="h-4 w-4" /> }
                    ].map((item) => (
                      <Link
                        key={item.name}
                        href={item.url}
                        onClick={handleResultClick}
                        className="flex items-center gap-2 p-2 rounded-lg text-slate-300 hover:bg-slate-800/50 hover:text-white transition-colors"
                      >
                        {item.icon}
                        <span className="text-sm">{item.name}</span>
                      </Link>
                    ))}
                  </div>
                </div>
              )}
            </div>

            {/* 搜索提示 */}
            {!query.trim() && (
              <div className="px-4 py-3 border-t border-slate-700 bg-slate-800/30">
                <div className="flex items-center justify-between text-xs text-slate-400">
                  <span>输入关键词开始搜索</span>
                  <div className="flex items-center gap-1">
                    <kbd className="px-2 py-1 bg-slate-700 rounded text-xs">ESC</kbd>
                    <span>关闭</span>
                  </div>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

/**
 * 搜索对话框组件 - 提供全站搜索功能的对话框
 */
export const SearchDialog = ({ isOpen, onClose }: GlobalSearchProps) => {
  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>站内搜索</DialogTitle>
        </DialogHeader>
        <div className="py-4">
          <GlobalSearch isOpen={isOpen} onClose={onClose} />
        </div>
      </DialogContent>
    </Dialog>
  );
};

/**
 * 搜索触发按钮组件
 */
export const SearchTrigger = () => {
  const [isSearchOpen, setIsSearchOpen] = useState(false);

  return (
    <>
      <Button
        variant="outline"
        className="w-full justify-start text-sm text-muted-foreground sm:pr-12 md:w-40 lg:w-64"
        onClick={() => setIsSearchOpen(true)}
      >
        <Search className="mr-2 h-4 w-4" />
        搜索...
        <kbd className="pointer-events-none absolute right-1.5 top-1.5 hidden h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 sm:flex">
          <span className="text-xs">⌘</span>K
        </kbd>
      </Button>
      <GlobalSearch isOpen={isSearchOpen} onClose={() => setIsSearchOpen(false)} />
    </>
  );
}