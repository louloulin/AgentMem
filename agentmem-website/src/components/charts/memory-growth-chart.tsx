/**
 * Memory Growth Chart Component
 * 
 * Displays memory growth trends using Recharts
 * Supabase-style modern chart design
 */

'use client';

import React from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Brain } from 'lucide-react';

interface MemoryGrowthChartProps {
  data?: Array<{
    date: string;
    count: number;
  }>;
}

// 默认数据（用于演示）
const defaultData = [
  { date: '2024-10-20', count: 120 },
  { date: '2024-10-21', count: 245 },
  { date: '2024-10-22', count: 386 },
  { date: '2024-10-23', count: 524 },
  { date: '2024-10-24', count: 688 },
  { date: '2024-10-25', count: 892 },
  { date: '2024-10-26', count: 1234 },
];

export function MemoryGrowthChart({ data = defaultData }: MemoryGrowthChartProps) {
  return (
    <Card className="shadow-sm hover:shadow-md transition-shadow duration-200">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2">
          <Brain className="w-5 h-5 text-blue-600" />
          记忆增长趋势
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={data}>
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
              stroke="#3b82f6" 
              strokeWidth={2}
              dot={{ fill: '#3b82f6', strokeWidth: 2 }}
              activeDot={{ r: 6 }}
            />
          </LineChart>
        </ResponsiveContainer>
        <div className="mt-4 text-sm text-gray-600 dark:text-gray-400">
          <p>过去7天新增 <span className="font-semibold text-blue-600 dark:text-blue-400">{data[data.length - 1].count - data[0].count}</span> 条记忆</p>
        </div>
      </CardContent>
    </Card>
  );
}

