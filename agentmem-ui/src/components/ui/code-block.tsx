"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Copy, Check, Download, Maximize2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface CodeBlockProps {
  code: string;
  language?: string;
  className?: string;
  showCopy?: boolean;
  showLineNumbers?: boolean;
  showLanguage?: boolean;
  title?: string;
  maxHeight?: string;
  filename?: string;
}

/**
 * 获取语言显示名称
 */
const getLanguageDisplayName = (language: string) => {
  const languageMap: Record<string, string> = {
    'javascript': 'JavaScript',
    'typescript': 'TypeScript',
    'python': 'Python',
    'rust': 'Rust',
    'bash': 'Bash',
    'json': 'JSON',
    'yaml': 'YAML',
    'toml': 'TOML',
    'sql': 'SQL',
    'html': 'HTML',
    'css': 'CSS',
    'jsx': 'JSX',
    'tsx': 'TSX',
    'go': 'Go',
    'java': 'Java',
    'cpp': 'C++',
    'c': 'C',
    'php': 'PHP',
    'ruby': 'Ruby',
    'swift': 'Swift',
    'kotlin': 'Kotlin',
    'dart': 'Dart',
    'xml': 'XML',
    'markdown': 'Markdown'
  };
  return languageMap[language.toLowerCase()] || language.toUpperCase();
};

/**
 * 获取语言颜色
 */
const getLanguageColor = (language: string) => {
  const colorMap: Record<string, string> = {
    'javascript': 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
    'typescript': 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    'python': 'bg-green-500/20 text-green-400 border-green-500/30',
    'rust': 'bg-orange-500/20 text-orange-400 border-orange-500/30',
    'bash': 'bg-gray-500/20 text-gray-400 border-gray-500/30',
    'json': 'bg-purple-500/20 text-purple-400 border-purple-500/30',
    'yaml': 'bg-red-500/20 text-red-400 border-red-500/30',
    'sql': 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30',
    'html': 'bg-pink-500/20 text-pink-400 border-pink-500/30',
    'css': 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30'
  };
  return colorMap[language.toLowerCase()] || 'bg-slate-500/20 text-slate-400 border-slate-500/30';
};

/**
 * 简单的语法高亮函数
 */
const highlightCode = (code: string, language: string) => {
  // 这里可以集成更复杂的语法高亮库，如 Prism.js 或 highlight.js
  // 目前提供基本的高亮
  let highlightedCode = code;
  
  if (language === 'javascript' || language === 'typescript' || language === 'jsx' || language === 'tsx') {
    // JavaScript/TypeScript 关键字
    highlightedCode = highlightedCode
      .replace(/\b(const|let|var|function|class|import|export|from|default|if|else|for|while|return|async|await|try|catch|finally)\b/g, 
        '<span class="text-purple-400 font-semibold">$1</span>')
      .replace(/\b(true|false|null|undefined)\b/g, 
        '<span class="text-orange-400">$1</span>')
      .replace(/("[^"]*"|'[^']*'|`[^`]*`)/g, 
        '<span class="text-green-400">$1</span>')
      .replace(/\/\/.*$/gm, 
        '<span class="text-slate-500 italic">$&</span>')
      .replace(/\/\*[\s\S]*?\*\//g, 
        '<span class="text-slate-500 italic">$&</span>');
  } else if (language === 'python') {
    // Python 关键字
    highlightedCode = highlightedCode
      .replace(/\b(def|class|import|from|if|elif|else|for|while|return|try|except|finally|with|as|pass|break|continue|and|or|not|in|is|lambda)\b/g, 
        '<span class="text-purple-400 font-semibold">$1</span>')
      .replace(/\b(True|False|None)\b/g, 
        '<span class="text-orange-400">$1</span>')
      .replace(/("[^"]*"|'[^']*'|"""[\s\S]*?"""|'''[\s\S]*?''')/g, 
        '<span class="text-green-400">$1</span>')
      .replace(/#.*$/gm, 
        '<span class="text-slate-500 italic">$&</span>');
  } else if (language === 'rust') {
    // Rust 关键字
    highlightedCode = highlightedCode
      .replace(/\b(fn|let|mut|const|struct|enum|impl|trait|use|mod|pub|if|else|match|for|while|loop|return|break|continue|async|await)\b/g, 
        '<span class="text-purple-400 font-semibold">$1</span>')
      .replace(/\b(true|false|None|Some)\b/g, 
        '<span class="text-orange-400">$1</span>')
      .replace(/("[^"]*")/g, 
        '<span class="text-green-400">$1</span>')
      .replace(/\/\/.*$/gm, 
        '<span class="text-slate-500 italic">$&</span>');
  } else if (language === 'json') {
    // JSON 高亮
    highlightedCode = highlightedCode
      .replace(/("[^"]*")\s*:/g, 
        '<span class="text-blue-400">$1</span>:')
      .replace(/:\s*("[^"]*")/g, 
        ': <span class="text-green-400">$1</span>')
      .replace(/\b(true|false|null)\b/g, 
        '<span class="text-orange-400">$1</span>')
      .replace(/\b\d+(\.\d+)?\b/g, 
        '<span class="text-yellow-400">$&</span>');
  }
  
  return highlightedCode;
};

/**
 * 代码块组件 - 支持语法高亮和复制功能
 */
export function CodeBlock({ 
  code, 
  language = "rust", 
  className, 
  showCopy = true,
  showLineNumbers = false,
  showLanguage = true,
  title,
  maxHeight = "400px",
  filename
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [highlightedCode, setHighlightedCode] = useState('');

  /**
   * 复制代码到剪贴板
   */
  const copyCode = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  };

  /**
   * 下载代码文件
   */
  const downloadCode = () => {
    const blob = new Blob([code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename || `code.${language}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // 高亮代码
  useEffect(() => {
    setHighlightedCode(highlightCode(code, language));
  }, [code, language]);

  const lines = code.split('\n');
  const shouldShowExpand = lines.length > 20;
  const displayLines = isExpanded ? lines : lines.slice(0, 20);

  return (
    <div className={cn("relative group", className)}>
      {/* 头部 */}
      {(title || filename || showLanguage) && (
        <div className="flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700 rounded-t-lg">
          <div className="flex items-center gap-3">
            {filename && (
              <span className="text-slate-300 text-sm font-mono">{filename}</span>
            )}
            {title && (
              <span className="text-white text-sm font-medium">{title}</span>
            )}
            {showLanguage && (
              <Badge className={`text-xs ${getLanguageColor(language)}`}>
                {getLanguageDisplayName(language)}
              </Badge>
            )}
          </div>
          <div className="flex items-center gap-2">
            {filename && (
              <Button
                size="sm"
                variant="ghost"
                onClick={downloadCode}
                className="h-7 w-7 p-0 text-slate-400 hover:text-white"
                title="下载文件"
              >
                <Download className="h-3 w-3" />
              </Button>
            )}
            {shouldShowExpand && (
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setIsExpanded(!isExpanded)}
                className="h-7 w-7 p-0 text-slate-400 hover:text-white"
                title={isExpanded ? "收起" : "展开"}
              >
                <Maximize2 className="h-3 w-3" />
              </Button>
            )}
          </div>
        </div>
      )}
      
      {/* 代码内容 */}
      <div 
        className="relative"
        style={{ maxHeight: isExpanded ? 'none' : maxHeight }}
      >
        <pre className={cn(
          "bg-slate-900 text-slate-100 p-4 overflow-x-auto text-sm font-mono leading-relaxed",
          (title || filename || showLanguage) ? "rounded-t-none rounded-b-lg" : "rounded-lg"
        )}>
          <code className={`language-${language}`}>
            {showLineNumbers ? (
              <div className="flex">
                <div className="select-none text-slate-500 text-right pr-4 border-r border-slate-700 mr-4">
                  {displayLines.map((_, index) => (
                    <div key={index} className="leading-relaxed">
                      {index + 1}
                    </div>
                  ))}
                </div>
                <div className="flex-1">
                  {displayLines.map((line, index) => (
                    <div 
                      key={index} 
                      className="leading-relaxed"
                      dangerouslySetInnerHTML={{ 
                        __html: highlightCode(line, language) || '&nbsp;' 
                      }}
                    />
                  ))}
                </div>
              </div>
            ) : (
              <div 
                dangerouslySetInnerHTML={{ 
                  __html: highlightCode(displayLines.join('\n'), language) 
                }}
              />
            )}
          </code>
        </pre>
        
        {/* 操作按钮 */}
        {showCopy && (
          <div className="absolute top-2 right-2 flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button
              size="sm"
              variant="outline"
              className="border-slate-600 bg-slate-800/90 hover:bg-slate-700 backdrop-blur-sm"
              onClick={copyCode}
            >
              {copied ? (
                <>
                  <Check className="h-4 w-4 text-green-400 mr-1" />
                  <span className="text-green-400 text-xs">已复制</span>
                </>
              ) : (
                <>
                  <Copy className="h-4 w-4 mr-1" />
                  <span className="text-xs">复制</span>
                </>
              )}
            </Button>
          </div>
        )}
        
        {/* 展开/收起提示 */}
        {shouldShowExpand && !isExpanded && (
          <div className="absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-slate-900 to-transparent flex items-end justify-center pb-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => setIsExpanded(true)}
              className="border-slate-600 bg-slate-800/90 hover:bg-slate-700 backdrop-blur-sm text-xs"
            >
              <Maximize2 className="h-3 w-3 mr-1" />
              显示全部 {lines.length} 行
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * 内联代码组件
 */
export function InlineCode({ 
  children, 
  className 
}: { 
  children: React.ReactNode; 
  className?: string; 
}) {
  return (
    <code className={cn(
      "bg-slate-800 text-purple-300 px-2 py-1 rounded text-sm font-mono",
      className
    )}>
      {children}
    </code>
  );
}