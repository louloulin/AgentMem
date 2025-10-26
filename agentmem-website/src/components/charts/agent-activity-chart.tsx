/**
 * Agent Activity Chart Component
 * 
 * Displays agent activity statistics using Recharts
 * Supabase-style modern bar chart design
 */

'use client';

import React from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Bot } from 'lucide-react';

interface AgentActivityChartProps {
  data?: Array<{
    agent: string;
    memories: number;
    interactions: number;
  }>;
}

// 默认数据（用于演示）
const defaultData = [
  { agent: 'Core', memories: 156, interactions: 89 },
  { agent: 'Episodic', memories: 234, interactions: 145 },
  { agent: 'Semantic', memories: 189, interactions: 98 },
  { agent: 'Procedural', memories: 145, interactions: 67 },
  { agent: 'Working', memories: 298, interactions: 234 },
];

export function AgentActivityChart({ data = defaultData }: AgentActivityChartProps) {
  return (
    <Card className="shadow-sm hover:shadow-md transition-shadow duration-200">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2">
          <Bot className="w-5 h-5 text-green-600" />
          Agent 活动统计
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={data}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-gray-200 dark:stroke-gray-700" />
            <XAxis 
              dataKey="agent" 
              tick={{ fill: 'hsl(var(--foreground))', fontSize: 12 }}
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
            <Legend 
              wrapperStyle={{ 
                paddingTop: '20px',
                fontSize: '14px',
              }}
            />
            <Bar dataKey="memories" fill="#3ECF8E" radius={[4, 4, 0, 0]} name="记忆数" />
            <Bar dataKey="interactions" fill="#2CB574" radius={[4, 4, 0, 0]} name="交互次数" />
          </BarChart>
        </ResponsiveContainer>
        <div className="mt-4 flex justify-around text-sm text-gray-600 dark:text-gray-400">
          <div>
            <p className="font-semibold text-supabase-green">
              {data.reduce((sum, item) => sum + item.memories, 0)}
            </p>
            <p>总记忆数</p>
          </div>
          <div>
            <p className="font-semibold text-green-600 dark:text-green-400">
              {data.reduce((sum, item) => sum + item.interactions, 0)}
            </p>
            <p>总交互次数</p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

