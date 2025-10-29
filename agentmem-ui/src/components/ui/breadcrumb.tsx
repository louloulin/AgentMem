"use client";

import { ChevronRight, Home } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";

/**
 * 面包屑项接口
 */
interface BreadcrumbItem {
  label: string;
  href?: string;
  icon?: React.ReactNode;
}

/**
 * 面包屑组件属性
 */
interface BreadcrumbProps {
  items?: BreadcrumbItem[];
  className?: string;
  showHome?: boolean;
  separator?: React.ReactNode;
}

/**
 * 路径映射配置
 */
const pathLabels: Record<string, string> = {
  '/': '首页',
  '/docs': '文档',
  '/demo': '演示',
  '/about': '关于',
  '/pricing': '定价',
  '/blog': '博客',
  '/support': '支持',
  '/api': 'API',
  '/tutorials': '教程',
  '/guides': '指南',
  '/examples': '示例',
  '/changelog': '更新日志',
  '/contact': '联系我们'
};

/**
 * 根据路径生成面包屑项
 */
const generateBreadcrumbItems = (pathname: string): BreadcrumbItem[] => {
  const segments = pathname.split('/').filter(Boolean);
  const items: BreadcrumbItem[] = [];
  
  // 添加首页
  items.push({
    label: '首页',
    href: '/',
    icon: <Home className="h-4 w-4" />
  });
  
  // 构建路径段
  let currentPath = '';
  segments.forEach((segment, index) => {
    currentPath += `/${segment}`;
    const isLast = index === segments.length - 1;
    
    // 处理动态路由和特殊情况
    let label = pathLabels[currentPath] || segment;
    
    // 如果是最后一个段且没有找到标签，尝试格式化
    if (isLast && !pathLabels[currentPath]) {
      label = segment
        .split('-')
        .map(word => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
    }
    
    items.push({
      label,
      href: isLast ? undefined : currentPath
    });
  });
  
  return items;
};

/**
 * 面包屑导航组件
 */
export function Breadcrumb({ 
  items, 
  className, 
  showHome = true,
  separator = <ChevronRight className="h-4 w-4 text-slate-400" />
}: BreadcrumbProps) {
  const pathname = usePathname();
  
  // 如果没有提供自定义项，则根据当前路径生成
  const breadcrumbItems = items || generateBreadcrumbItems(pathname);
  
  // 如果不显示首页且只有首页项，则不渲染
  if (!showHome && breadcrumbItems.length === 1 && breadcrumbItems[0].href === '/') {
    return null;
  }
  
  // 过滤首页项（如果不显示首页）
  const displayItems = showHome ? breadcrumbItems : breadcrumbItems.slice(1);
  
  if (displayItems.length === 0) {
    return null;
  }
  
  return (
    <nav 
      className={cn("flex items-center space-x-2 text-sm", className)}
      aria-label="面包屑导航"
    >
      <ol className="flex items-center space-x-2">
        {displayItems.map((item, index) => {
          const isLast = index === displayItems.length - 1;
          
          return (
            <li key={index} className="flex items-center space-x-2">
              {index > 0 && (
                <span className="flex-shrink-0" aria-hidden="true">
                  {separator}
                </span>
              )}
              
              {item.href && !isLast ? (
                <Link
                  href={item.href}
                  className="flex items-center space-x-1 text-slate-400 hover:text-white transition-colors"
                >
                  {item.icon}
                  <span>{item.label}</span>
                </Link>
              ) : (
                <span 
                  className={cn(
                    "flex items-center space-x-1",
                    isLast ? "text-white font-medium" : "text-slate-400"
                  )}
                  aria-current={isLast ? "page" : undefined}
                >
                  {item.icon}
                  <span>{item.label}</span>
                </span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}

/**
 * 简化的面包屑组件（仅显示当前页面）
 */
export function SimpleBreadcrumb({ className }: { className?: string }) {
  const pathname = usePathname();
  const currentLabel = pathLabels[pathname] || '页面';
  
  return (
    <div className={cn("flex items-center space-x-2 text-sm text-slate-400", className)}>
      <Home className="h-4 w-4" />
      <ChevronRight className="h-4 w-4" />
      <span className="text-white font-medium">{currentLabel}</span>
    </div>
  );
}

/**
 * 面包屑容器组件（带背景和样式）
 */
export function BreadcrumbContainer({ 
  children, 
  className 
}: { 
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={cn(
      "bg-slate-800/30 backdrop-blur-sm border-b border-slate-700 py-3",
      className
    )}>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {children}
      </div>
    </div>
  );
}