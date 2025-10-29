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

  if (error) {
    return (
      <Card className="shadow-sm">
        <CardHeader>
          <CardTitle className="text-lg font-semibold flex items-center gap-2 text-red-600">
            <Brain className="w-5 h-5" />
            Memory Growth Trend
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-center text-gray-500 py-8">{error}</p>
          <button
            onClick={loadData}
            className="mx-auto block px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            Retry
          </button>
        </CardContent>
      </Card>
    );
  }

  if (loading && !growthData) {
    return (
      <Card className="shadow-sm">
        <CardHeader>
          <CardTitle className="text-lg font-semibold flex items-center gap-2">
            <Brain className="w-5 h-5 text-blue-600" />
            Memory Growth Trend
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-[300px] flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-blue-600" />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="shadow-sm hover:shadow-md transition-shadow duration-200">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2">
          <Brain className="w-5 h-5 text-blue-600" />
          Memory Growth Trend
          <span className="text-xs text-green-600 font-normal">✅ Live Data</span>
        </CardTitle>
        <button
          onClick={loadData}
          disabled={loading}
          className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
          title="Refresh data"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </CardHeader>
      <CardContent>
        {chartData.length === 0 ? (
          <div className="h-[300px] flex items-center justify-center text-gray-500">
            <div className="text-center">
              <Brain className="w-12 h-12 mx-auto mb-2 opacity-50" />
              <p>No growth data available yet</p>
              <p className="text-sm mt-1">Create some memories to see trends</p>
            </div>
          </div>
        ) : (
          <>
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
                  formatter={(value: number, name: string) => {
                    if (name === 'total') return [value, 'Total Memories'];
                    if (name === 'new') return [value, 'New Memories'];
                    return [value, name];
                  }}
                />
                <Legend 
                  wrapperStyle={{ 
                    paddingTop: '10px',
                    fontSize: '14px',
                  }}
                />
                <Line 
                  type="monotone" 
                  dataKey="total" 
                  stroke="#3ECF8E" 
                  strokeWidth={2}
                  dot={{ fill: '#3ECF8E', strokeWidth: 2 }}
                  activeDot={{ r: 6 }}
                  name="Total Memories"
                />
                <Line 
                  type="monotone" 
                  dataKey="new" 
                  stroke="#60A5FA" 
                  strokeWidth={2}
                  dot={{ fill: '#60A5FA', strokeWidth: 2 }}
                  activeDot={{ r: 6 }}
                  name="New Memories"
                  strokeDasharray="5 5"
                />
              </LineChart>
            </ResponsiveContainer>
            <div className="mt-4 grid grid-cols-3 gap-4 text-sm">
              <div className="text-center">
                <p className="text-gray-600 dark:text-gray-400">Total Memories</p>
                <p className="font-semibold text-blue-600 dark:text-blue-400 text-lg">
                  {totalMemories.toLocaleString()}
                </p>
              </div>
              <div className="text-center">
                <p className="text-gray-600 dark:text-gray-400">30-Day Growth</p>
                <p className="font-semibold text-green-600 dark:text-green-400 text-lg flex items-center justify-center gap-1">
                  <TrendingUp className="w-4 h-4" />
                  +{periodGrowth.toLocaleString()}
                </p>
              </div>
              <div className="text-center">
                <p className="text-gray-600 dark:text-gray-400">Daily Avg</p>
                <p className="font-semibold text-purple-600 dark:text-purple-400 text-lg">
                  {growthRate.toFixed(1)}/day
                </p>
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
