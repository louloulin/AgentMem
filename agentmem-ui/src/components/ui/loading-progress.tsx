"use client";

import { useState, useEffect, Suspense } from "react";
import { usePathname, useSearchParams } from "next/navigation";
import { cn } from "@/lib/utils";
import { Loader2, CheckCircle, AlertCircle } from "lucide-react";

/**
 * 页面加载进度条组件（内部实现）
 */
function PageLoadingProgressInner() {
  const [isLoading, setIsLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const pathname = usePathname();
  const searchParams = useSearchParams();

  useEffect(() => {
    const handleStart = () => {
      setIsLoading(true);
      setProgress(0);
      
      // 模拟进度增长
      const interval = setInterval(() => {
        setProgress(prev => {
          if (prev >= 90) {
            clearInterval(interval);
            return 90;
          }
          return prev + Math.random() * 30;
        });
      }, 200);
      
      return interval;
    };

    const handleComplete = () => {
      setProgress(100);
      setTimeout(() => {
        setIsLoading(false);
        setProgress(0);
      }, 200);
    };

    // 监听路由变化
    // 页面开始加载
    const interval = handleStart();

    // 页面加载完成
    const timer = setTimeout(() => {
      clearInterval(interval);
      handleComplete();
    }, 1000);

    return () => {
      clearInterval(interval);
      clearTimeout(timer);
    };
  }, [pathname, searchParams]);

  if (!isLoading && progress === 0) {
    return null;
  }

  return (
    <div className="fixed top-0 left-0 right-0 z-50 h-1 bg-slate-800/50">
      <div
        className="h-full bg-gradient-to-r from-purple-500 to-pink-500 transition-all duration-300 ease-out"
        style={{ width: `${progress}%` }}
      />
    </div>
  );
}

/**
 * 页面加载进度条组件（带 Suspense 包装）
 */
export function PageLoadingProgress() {
  return (
    <Suspense fallback={null}>
      <PageLoadingProgressInner />
    </Suspense>
  );
}

/**
 * 内容加载骨架屏组件
 */
interface SkeletonProps {
  className?: string;
  variant?: 'text' | 'rectangular' | 'circular';
  width?: string | number;
  height?: string | number;
  animation?: 'pulse' | 'wave' | 'none';
}

export function Skeleton({ 
  className, 
  variant = 'text',
  width,
  height,
  animation = 'pulse'
}: SkeletonProps) {
  const baseClasses = "bg-slate-700";
  
  const variantClasses = {
    text: "rounded",
    rectangular: "rounded-lg",
    circular: "rounded-full"
  };
  
  const animationClasses = {
    pulse: "animate-pulse",
    wave: "animate-pulse", // 可以自定义波浪动画
    none: ""
  };

  const style: React.CSSProperties = {};
  if (width) style.width = typeof width === 'number' ? `${width}px` : width;
  if (height) style.height = typeof height === 'number' ? `${height}px` : height;
  
  // 默认高度
  if (variant === 'text' && !height) {
    style.height = '1rem';
  }

  return (
    <div 
      className={cn(
        baseClasses,
        variantClasses[variant],
        animationClasses[animation],
        className
      )}
      style={style}
    />
  );
}

/**
 * 卡片骨架屏
 */
export function CardSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn("p-6 bg-slate-800/50 border border-slate-700 rounded-lg", className)}>
      <div className="space-y-4">
        <Skeleton variant="rectangular" height={200} />
        <div className="space-y-2">
          <Skeleton width="60%" />
          <Skeleton width="80%" />
          <Skeleton width="40%" />
        </div>
      </div>
    </div>
  );
}

/**
 * 列表骨架屏
 */
export function ListSkeleton({ 
  items = 5, 
  className 
}: { 
  items?: number;
  className?: string;
}) {
  return (
    <div className={cn("space-y-4", className)}>
      {Array.from({ length: items }).map((_, index) => (
        <div key={index} className="flex items-center space-x-4">
          <Skeleton variant="circular" width={40} height={40} />
          <div className="flex-1 space-y-2">
            <Skeleton width="70%" />
            <Skeleton width="50%" />
          </div>
        </div>
      ))}
    </div>
  );
}

/**
 * 表格骨架屏
 */
export function TableSkeleton({ 
  rows = 5, 
  columns = 4,
  className 
}: { 
  rows?: number;
  columns?: number;
  className?: string;
}) {
  return (
    <div className={cn("space-y-4", className)}>
      {/* 表头 */}
      <div className="grid gap-4" style={{ gridTemplateColumns: `repeat(${columns}, 1fr)` }}>
        {Array.from({ length: columns }).map((_, index) => (
          <Skeleton key={index} height={20} />
        ))}
      </div>
      
      {/* 表格行 */}
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <div key={rowIndex} className="grid gap-4" style={{ gridTemplateColumns: `repeat(${columns}, 1fr)` }}>
          {Array.from({ length: columns }).map((_, colIndex) => (
            <Skeleton key={colIndex} height={16} />
          ))}
        </div>
      ))}
    </div>
  );
}

/**
 * 加载状态组件
 */
interface LoadingStateProps {
  loading: boolean;
  error?: string | null;
  children: React.ReactNode;
  skeleton?: React.ReactNode;
  className?: string;
}

export function LoadingState({ 
  loading, 
  error, 
  children, 
  skeleton,
  className 
}: LoadingStateProps) {
  if (loading) {
    return (
      <div className={cn("animate-pulse", className)}>
        {skeleton || (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-purple-400" />
            <span className="ml-2 text-slate-400">加载中...</span>
          </div>
        )}
      </div>
    );
  }

  if (error) {
    return (
      <div className={cn("flex flex-col items-center justify-center py-12 text-red-400", className)}>
        <AlertCircle className="h-12 w-12 mb-4" />
        <p className="text-lg font-semibold mb-2">加载失败</p>
        <p className="text-sm text-slate-400">{error}</p>
      </div>
    );
  }

  return <div className={className}>{children}</div>;
}

/**
 * 懒加载容器组件
 */
interface LazyLoadProps {
  children: React.ReactNode;
  placeholder?: React.ReactNode;
  rootMargin?: string;
  threshold?: number;
  className?: string;
  onIntersect?: () => void;
}

export function LazyLoad({ 
  children, 
  placeholder,
  rootMargin = '100px',
  threshold = 0.1,
  className,
  onIntersect
}: LazyLoadProps) {
  const [isInView, setIsInView] = useState(false);
  const [ref, setRef] = useState<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!ref) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setIsInView(true);
            onIntersect?.();
            observer.disconnect();
          }
        });
      },
      {
        rootMargin,
        threshold
      }
    );

    observer.observe(ref);

    return () => observer.disconnect();
  }, [ref, rootMargin, threshold, onIntersect]);

  return (
    <div ref={setRef} className={className}>
      {isInView ? children : (placeholder || <Skeleton height={200} />)}
    </div>
  );
}

/**
 * 无限滚动组件
 */
interface InfiniteScrollProps {
  children: React.ReactNode;
  hasMore: boolean;
  loading: boolean;
  onLoadMore: () => void;
  threshold?: number;
  className?: string;
}

export function InfiniteScroll({ 
  children, 
  hasMore, 
  loading, 
  onLoadMore,
  threshold = 100,
  className 
}: InfiniteScrollProps) {
  const [ref, setRef] = useState<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!ref || loading || !hasMore) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            onLoadMore();
          }
        });
      },
      {
        rootMargin: `${threshold}px`
      }
    );

    observer.observe(ref);

    return () => observer.disconnect();
  }, [ref, loading, hasMore, onLoadMore, threshold]);

  return (
    <div className={className}>
      {children}
      
      {hasMore && (
        <div ref={setRef} className="flex items-center justify-center py-8">
          {loading ? (
            <div className="flex items-center">
              <Loader2 className="h-6 w-6 animate-spin text-purple-400 mr-2" />
              <span className="text-slate-400">加载更多...</span>
            </div>
          ) : (
            <div className="h-1" /> // 触发区域
          )}
        </div>
      )}
      
      {!hasMore && (
        <div className="flex items-center justify-center py-8 text-slate-400">
          <CheckCircle className="h-5 w-5 mr-2" />
          <span>已加载全部内容</span>
        </div>
      )}
    </div>
  );
}

/**
 * 页面加载包装器
 */
interface PageLoaderProps {
  loading?: boolean;
  children: React.ReactNode;
}

export function PageLoader({ loading = false, children }: PageLoaderProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted || loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="h-12 w-12 animate-spin text-purple-400 mx-auto mb-4" />
          <p className="text-white text-lg">加载中...</p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}