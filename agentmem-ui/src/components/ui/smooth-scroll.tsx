"use client";

import { useEffect, useState } from "react";
import { ChevronUp, Menu } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";

/**
 * 锚点项接口
 */
interface AnchorItem {
  id: string;
  label: string;
  offset?: number;
}

/**
 * 平滑滚动到指定元素
 */
export const smoothScrollTo = (elementId: string, offset: number = 80) => {
  const element = document.getElementById(elementId);
  if (element) {
    const elementPosition = element.getBoundingClientRect().top;
    const offsetPosition = elementPosition + window.pageYOffset - offset;

    window.scrollTo({
      top: offsetPosition,
      behavior: 'smooth'
    });
  }
};

/**
 * 平滑滚动到顶部
 */
export const scrollToTop = () => {
  window.scrollTo({
    top: 0,
    behavior: 'smooth'
  });
};

/**
 * 回到顶部按钮组件
 */
export function ScrollToTopButton({ className }: { className?: string }) {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const toggleVisibility = () => {
      if (window.pageYOffset > 300) {
        setIsVisible(true);
      } else {
        setIsVisible(false);
      }
    };

    window.addEventListener('scroll', toggleVisibility);
    return () => window.removeEventListener('scroll', toggleVisibility);
  }, []);

  if (!isVisible) {
    return null;
  }

  return (
    <Button
      onClick={scrollToTop}
      className={cn(
        "fixed bottom-8 right-8 z-50 h-12 w-12 rounded-full bg-purple-600 hover:bg-purple-700 shadow-lg transition-all duration-300 hover:scale-110",
        className
      )}
      size="sm"
      aria-label="回到顶部"
    >
      <ChevronUp className="h-5 w-5" />
    </Button>
  );
}

/**
 * 页面导航组件属性
 */
interface PageNavigationProps {
  anchors: AnchorItem[];
  className?: string;
  activeColor?: string;
}

/**
 * 页面导航组件（侧边栏锚点导航）
 */
export function PageNavigation({ 
  anchors, 
  className,
  activeColor = "border-purple-500 bg-purple-500/20"
}: PageNavigationProps) {
  const [activeAnchor, setActiveAnchor] = useState<string>('');
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      // 显示/隐藏导航
      setIsVisible(window.pageYOffset > 200);

      // 确定当前活跃的锚点
      const scrollPosition = window.pageYOffset + 100;
      
      for (let i = anchors.length - 1; i >= 0; i--) {
        const anchor = anchors[i];
        const element = document.getElementById(anchor.id);
        
        if (element) {
          const elementTop = element.offsetTop;
          if (scrollPosition >= elementTop) {
            setActiveAnchor(anchor.id);
            break;
          }
        }
      }
    };

    window.addEventListener('scroll', handleScroll);
    handleScroll(); // 初始检查
    
    return () => window.removeEventListener('scroll', handleScroll);
  }, [anchors]);

  const handleAnchorClick = (anchorId: string, offset?: number) => {
    smoothScrollTo(anchorId, offset);
  };

  if (!isVisible || anchors.length === 0) {
    return null;
  }

  return (
    <Card className={cn(
      "fixed left-8 top-1/2 transform -translate-y-1/2 z-40 bg-slate-800/90 backdrop-blur-sm border-slate-700 shadow-xl",
      className
    )}>
      <CardContent className="p-2">
        <nav aria-label="页面导航">
          <ul className="space-y-1">
            {anchors.map((anchor) => (
              <li key={anchor.id}>
                <button
                  onClick={() => handleAnchorClick(anchor.id, anchor.offset)}
                  className={cn(
                    "w-full text-left px-3 py-2 text-sm rounded-md transition-all duration-200 hover:bg-slate-700",
                    activeAnchor === anchor.id
                      ? `text-white ${activeColor}`
                      : "text-slate-400 hover:text-white"
                  )}
                  aria-current={activeAnchor === anchor.id ? "true" : undefined}
                >
                  {anchor.label}
                </button>
              </li>
            ))}
          </ul>
        </nav>
      </CardContent>
    </Card>
  );
}

/**
 * 移动端页面导航组件
 */
export function MobilePageNavigation({ 
  anchors, 
  className 
}: { 
  anchors: AnchorItem[];
  className?: string;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const [activeAnchor, setActiveAnchor] = useState<string>('');

  useEffect(() => {
    const handleScroll = () => {
      const scrollPosition = window.pageYOffset + 100;
      
      for (let i = anchors.length - 1; i >= 0; i--) {
        const anchor = anchors[i];
        const element = document.getElementById(anchor.id);
        
        if (element) {
          const elementTop = element.offsetTop;
          if (scrollPosition >= elementTop) {
            setActiveAnchor(anchor.id);
            break;
          }
        }
      }
    };

    window.addEventListener('scroll', handleScroll);
    handleScroll();
    
    return () => window.removeEventListener('scroll', handleScroll);
  }, [anchors]);

  const handleAnchorClick = (anchorId: string, offset?: number) => {
    smoothScrollTo(anchorId, offset);
    setIsOpen(false);
  };

  if (anchors.length === 0) {
    return null;
  }

  return (
    <div className={cn("md:hidden", className)}>
      {/* 触发按钮 */}
      <Button
        onClick={() => setIsOpen(!isOpen)}
        className="fixed bottom-20 right-8 z-50 h-12 w-12 rounded-full bg-slate-800/90 hover:bg-slate-700 shadow-lg backdrop-blur-sm"
        size="sm"
        aria-label="页面导航"
      >
        <Menu className="h-5 w-5" />
      </Button>

      {/* 导航菜单 */}
      {isOpen && (
        <>
          {/* 背景遮罩 */}
          <div 
            className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
            onClick={() => setIsOpen(false)}
          />
          
          {/* 导航内容 */}
          <Card className="fixed bottom-36 right-8 z-50 bg-slate-800/95 backdrop-blur-sm border-slate-700 shadow-xl max-w-xs">
            <CardContent className="p-4">
              <h3 className="text-white font-semibold mb-3">页面导航</h3>
              <nav>
                <ul className="space-y-2">
                  {anchors.map((anchor) => (
                    <li key={anchor.id}>
                      <button
                        onClick={() => handleAnchorClick(anchor.id, anchor.offset)}
                        className={cn(
                          "w-full text-left px-3 py-2 text-sm rounded-md transition-all duration-200",
                          activeAnchor === anchor.id
                            ? "text-white bg-purple-500/20 border-l-2 border-purple-500"
                            : "text-slate-400 hover:text-white hover:bg-slate-700"
                        )}
                      >
                        {anchor.label}
                      </button>
                    </li>
                  ))}
                </ul>
              </nav>
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}

/**
 * 进度指示器组件
 */
export function ScrollProgressIndicator({ className }: { className?: string }) {
  const [scrollProgress, setScrollProgress] = useState(0);

  useEffect(() => {
    const handleScroll = () => {
      const totalHeight = document.documentElement.scrollHeight - window.innerHeight;
      const progress = (window.pageYOffset / totalHeight) * 100;
      setScrollProgress(Math.min(progress, 100));
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <div className={cn(
      "fixed top-0 left-0 right-0 z-50 h-1 bg-slate-800/50",
      className
    )}>
      <div 
        className="h-full bg-gradient-to-r from-purple-500 to-pink-500 transition-all duration-150 ease-out"
        style={{ width: `${scrollProgress}%` }}
      />
    </div>
  );
}

/**
 * 平滑滚动链接组件
 */
interface SmoothScrollLinkProps {
  href: string;
  children: React.ReactNode;
  className?: string;
  offset?: number;
  onClick?: () => void;
}

export function SmoothScrollLink({ 
  href, 
  children, 
  className, 
  offset = 80,
  onClick
}: SmoothScrollLinkProps) {
  const handleClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    // 如果是锚点链接（以 # 开头）
    if (href.startsWith('#')) {
      e.preventDefault();
      const elementId = href.substring(1);
      smoothScrollTo(elementId, offset);
      onClick?.();
    }
  };

  return (
    <a 
      href={href}
      className={className}
      onClick={handleClick}
    >
      {children}
    </a>
  );
}