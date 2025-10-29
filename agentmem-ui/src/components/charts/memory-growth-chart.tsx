/**
 * Memory Growth Chart Component
 * 
 * Displays memory growth trends using Recharts
 * ✅ 100% Real Data from Stats API
 */

'use client';

import React, { useEffect, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Brain, RefreshCw, TrendingUp } from 'lucide-react';
import { apiClient, type MemoryGrowthResponse } from '@/lib/api-client';

interface MemoryGrowthChartProps {
  autoRefresh?: boolean;
  refreshInterval?: number; // in milliseconds
}

export function MemoryGrowthChart({ 
  autoRefresh = true,
  refreshInterval = 30000 // 30 seconds
}: MemoryGrowthChartProps) {
  const [growthData, setGrowthData] = useState<MemoryGrowthResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // ✅ Load real data from new Stats API
  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Use new dedicated Memory Growth API
      const response = await apiClient.getMemoryGrowth();
      setGrowthData(response);
    } catch (err) {
      console.error('Failed to load memory growth data:', err);
      setError('Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    // Load data on mount
    loadData();

    // Set up auto-refresh
    if (autoRefresh) {
      const interval = setInterval(loadData, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [autoRefresh, refreshInterval]);

  // Calculate growth statistics
  const chartData = growthData?.data || [];
  const totalMemories = growthData?.total_memories || 0;
  const growthRate = growthData?.growth_rate || 0;
  const periodGrowth = chartData.length > 1 
    ? chartData[chartData.length - 1].total - chartData[0].total 
    : 0;

  return (
    <Card className="shadow-sm hover:shadow-md transition-shadow duration-200">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2">
          <Brain className="w-5 h-5 text-blue-600" />
          记忆增长趋势
          {!isUsingRealData && (
            <span className="text-xs text-gray-500 font-normal">(示例数据)</span>
          )}
        </CardTitle>
        {!propData && (
          <button
            onClick={loadData}
            disabled={loading}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
            title="刷新数据"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>
        )}
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-gray-200 dark:stroke-gray-700" />
            <XAxis 
              dataKey="date" 
              tick={{ fill: 'hsl(var(--foreground))', fontSize: 12 }}
              tickFormatter={(value) => {
                const date = new Date(value);
                return `${date.getMonth() + 1}/${date.getDate()}`;
              }}
            />
            <YAxis 
              tick={{ fill: 'hsl(var(--foreground))', fontSize: 12 }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'hsl(var(--background))',
                border: '1px solid hsl(var(--border))',
                borderRadius: '8px',
                padding: '8px 12px',
              }}
              labelStyle={{ color: 'hsl(var(--foreground))' }}
            />
            <Line 
              type="monotone" 
              dataKey="count" 
              stroke="#3ECF8E" 
              strokeWidth={2}
              dot={{ fill: '#3ECF8E', strokeWidth: 2 }}
              activeDot={{ r: 6 }}
            />
          </LineChart>
        </ResponsiveContainer>
        <div className="mt-4 text-sm text-gray-600 dark:text-gray-400">
          <p>
            过去7天新增 
            <span className="font-semibold text-blue-600 dark:text-blue-400"> {growth} </span> 
            条记忆
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
