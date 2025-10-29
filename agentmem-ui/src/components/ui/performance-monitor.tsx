"use client";

import { useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Activity, Clock, Zap, TrendingUp } from 'lucide-react';

interface PerformanceMetrics {
  loadTime: number;
  firstContentfulPaint: number;
  largestContentfulPaint: number;
  cumulativeLayoutShift: number;
  firstInputDelay: number;
  memoryUsage?: number;
  connectionType?: string;
}

/**
 * 性能监控钩子
 */
export function usePerformanceMonitor() {
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const measurePerformance = () => {
      try {
        const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
        const paint = performance.getEntriesByType('paint');
        
        const fcp = paint.find(entry => entry.name === 'first-contentful-paint')?.startTime || 0;
        
        // 获取 LCP (需要 PerformanceObserver)
        let lcp = 0;
        if ('PerformanceObserver' in window) {
          const observer = new PerformanceObserver((list) => {
            const entries = list.getEntries();
            const lastEntry = entries[entries.length - 1];
            lcp = lastEntry.startTime;
          });
          observer.observe({ entryTypes: ['largest-contentful-paint'] });
        }

        // 获取内存使用情况 (Chrome only)
        const memoryInfo = (performance as unknown as { memory?: { usedJSHeapSize: number } }).memory;
        const memoryUsage = memoryInfo ? memoryInfo.usedJSHeapSize / 1024 / 1024 : undefined;

        // 获取网络连接信息
        const connection = (navigator as unknown as { connection?: { effectiveType: string } }).connection;
        const connectionType = connection ? connection.effectiveType : undefined;

        const performanceMetrics: PerformanceMetrics = {
          loadTime: navigation.loadEventEnd - navigation.startTime,
          firstContentfulPaint: fcp,
          largestContentfulPaint: lcp,
          cumulativeLayoutShift: 0, // 需要 PerformanceObserver 来测量
          firstInputDelay: 0, // 需要 PerformanceObserver 来测量
          memoryUsage,
          connectionType,
        };

        setMetrics(performanceMetrics);
        setIsLoading(false);
      } catch (error) {
        console.error('Performance measurement failed:', error);
        setIsLoading(false);
      }
    };

    // 等待页面完全加载后测量性能
    if (document.readyState === 'complete') {
      measurePerformance();
    } else {
      window.addEventListener('load', measurePerformance);
      return () => window.removeEventListener('load', measurePerformance);
    }
  }, []);

  return { metrics, isLoading };
}

/**
 * 性能评分函数
 */
function getPerformanceScore(value: number, thresholds: { good: number; needs_improvement: number }): {
  score: 'good' | 'needs_improvement' | 'poor';
  color: string;
} {
  if (value <= thresholds.good) {
    return { score: 'good', color: 'text-green-500' };
  } else if (value <= thresholds.needs_improvement) {
    return { score: 'needs_improvement', color: 'text-yellow-500' };
  } else {
    return { score: 'poor', color: 'text-red-500' };
  }
}

/**
 * 性能监控仪表板组件
 */
export function PerformanceDashboard() {
  const { metrics, isLoading } = usePerformanceMonitor();

  if (isLoading) {
    return (
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white flex items-center">
            <Activity className="mr-2 h-5 w-5" />
            性能监控
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="animate-pulse space-y-4">
            <div className="h-4 bg-slate-700 rounded w-3/4"></div>
            <div className="h-4 bg-slate-700 rounded w-1/2"></div>
            <div className="h-4 bg-slate-700 rounded w-2/3"></div>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!metrics) {
    return (
      <Card className="bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white flex items-center">
            <Activity className="mr-2 h-5 w-5" />
            性能监控
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-slate-400">无法获取性能数据</p>
        </CardContent>
      </Card>
    );
  }

  const loadTimeScore = getPerformanceScore(metrics.loadTime, { good: 1000, needs_improvement: 2500 });
  const fcpScore = getPerformanceScore(metrics.firstContentfulPaint, { good: 1800, needs_improvement: 3000 });

  return (
    <Card className="bg-slate-800/50 border-slate-700">
      <CardHeader>
        <CardTitle className="text-white flex items-center">
          <Activity className="mr-2 h-5 w-5" />
          性能监控
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* 页面加载时间 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <Clock className="mr-2 h-4 w-4 text-blue-400" />
            <span className="text-slate-300">页面加载时间</span>
          </div>
          <div className="flex items-center space-x-2">
            <span className={loadTimeScore.color}>
              {Math.round(metrics.loadTime)}ms
            </span>
            <Badge 
              variant={loadTimeScore.score === 'good' ? 'default' : 'secondary'}
              className={loadTimeScore.score === 'good' ? 'bg-green-500' : loadTimeScore.score === 'needs_improvement' ? 'bg-yellow-500' : 'bg-red-500'}
            >
              {loadTimeScore.score === 'good' ? '优秀' : loadTimeScore.score === 'needs_improvement' ? '良好' : '需改进'}
            </Badge>
          </div>
        </div>

        {/* 首次内容绘制 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <Zap className="mr-2 h-4 w-4 text-purple-400" />
            <span className="text-slate-300">首次内容绘制</span>
          </div>
          <div className="flex items-center space-x-2">
            <span className={fcpScore.color}>
              {Math.round(metrics.firstContentfulPaint)}ms
            </span>
            <Badge 
              variant={fcpScore.score === 'good' ? 'default' : 'secondary'}
              className={fcpScore.score === 'good' ? 'bg-green-500' : fcpScore.score === 'needs_improvement' ? 'bg-yellow-500' : 'bg-red-500'}
            >
              {fcpScore.score === 'good' ? '优秀' : fcpScore.score === 'needs_improvement' ? '良好' : '需改进'}
            </Badge>
          </div>
        </div>

        {/* 内存使用 */}
        {metrics.memoryUsage && (
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <TrendingUp className="mr-2 h-4 w-4 text-green-400" />
              <span className="text-slate-300">内存使用</span>
            </div>
            <span className="text-slate-300">
              {metrics.memoryUsage.toFixed(1)} MB
            </span>
          </div>
        )}

        {/* 网络连接 */}
        {metrics.connectionType && (
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <Activity className="mr-2 h-4 w-4 text-cyan-400" />
              <span className="text-slate-300">网络连接</span>
            </div>
            <Badge variant="outline" className="border-cyan-400 text-cyan-400">
              {metrics.connectionType.toUpperCase()}
            </Badge>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

/**
 * 简化的性能指标组件
 */
export function PerformanceMetrics() {
  const { metrics, isLoading } = usePerformanceMonitor();

  if (isLoading || !metrics) {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 z-50">
      <Card className="bg-slate-900/90 border-slate-700 backdrop-blur-sm">
        <CardContent className="p-3">
          <div className="flex items-center space-x-4 text-xs">
            <div className="flex items-center">
              <Clock className="mr-1 h-3 w-3 text-blue-400" />
              <span className="text-slate-300">{Math.round(metrics.loadTime)}ms</span>
            </div>
            <div className="flex items-center">
              <Zap className="mr-1 h-3 w-3 text-purple-400" />
              <span className="text-slate-300">{Math.round(metrics.firstContentfulPaint)}ms</span>
            </div>
            {metrics.memoryUsage && (
              <div className="flex items-center">
                <TrendingUp className="mr-1 h-3 w-3 text-green-400" />
                <span className="text-slate-300">{metrics.memoryUsage.toFixed(1)}MB</span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}