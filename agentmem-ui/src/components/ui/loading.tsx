"use client";

import { cn } from "@/lib/utils";

/**
 * 加载旋转器组件
 */
export function LoadingSpinner({ 
  className, 
  size = "md" 
}: { 
  className?: string; 
  size?: "sm" | "md" | "lg"; 
}) {
  const sizeClasses = {
    sm: "h-4 w-4",
    md: "h-6 w-6",
    lg: "h-8 w-8"
  };

  return (
    <div className={cn(
      "animate-spin rounded-full border-2 border-slate-600 border-t-purple-400",
      sizeClasses[size],
      className
    )} />
  );
}

/**
 * 骨架屏组件
 */
export function Skeleton({ 
  className 
}: { 
  className?: string; 
}) {
  return (
    <div className={cn(
      "animate-pulse rounded-md bg-slate-700",
      className
    )} />
  );
}

/**
 * 卡片骨架屏
 */
export function CardSkeleton() {
  return (
    <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-6 space-y-4">
      <div className="flex items-center space-x-4">
        <Skeleton className="h-12 w-12 rounded-full" />
        <div className="space-y-2 flex-1">
          <Skeleton className="h-4 w-3/4" />
          <Skeleton className="h-3 w-1/2" />
        </div>
      </div>
      <div className="space-y-2">
        <Skeleton className="h-3 w-full" />
        <Skeleton className="h-3 w-5/6" />
        <Skeleton className="h-3 w-4/6" />
      </div>
    </div>
  );
}

/**
 * 页面加载组件
 */
export function PageLoading() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 flex items-center justify-center">
      <div className="text-center space-y-4">
        <LoadingSpinner size="lg" />
        <p className="text-slate-300">加载中...</p>
      </div>
    </div>
  );
}

/**
 * 内容加载组件
 */
export function ContentLoading({ 
  lines = 3 
}: { 
  lines?: number; 
}) {
  return (
    <div className="space-y-3">
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton 
          key={i} 
          className={cn(
            "h-4",
            i === lines - 1 ? "w-3/4" : "w-full"
          )} 
        />
      ))}
    </div>
  );
}

/**
 * 按钮加载状态
 */
export function ButtonLoading({ 
  children, 
  isLoading, 
  ...props 
}: React.ButtonHTMLAttributes<HTMLButtonElement> & { 
  children?: React.ReactNode; 
  isLoading: boolean; 
}) {
  return (
    <button {...props} disabled={isLoading || props.disabled}>
      {isLoading && <LoadingSpinner size="sm" className="mr-2" />}
      {children}
    </button>
  );
}