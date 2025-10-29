/**
 * Agent Activity Chart Component
 * 
 * Displays agent activity statistics using Recharts
 * ✅ Now supports real-time data from metrics API
 */

'use client';

import React, { useEffect, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Bot, RefreshCw } from 'lucide-react';
import { apiClient } from '@/lib/api-client';

interface AgentActivityChartProps {
  data?: Array<{
    agent: string;
    memories: number;
    interactions: number;
  }>;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

// Fallback data for when API is not available
const fallbackData = [
  { agent: 'Core', memories: 156, interactions: 89 },
  { agent: 'Episodic', memories: 234, interactions: 145 },
  { agent: 'Semantic', memories: 189, interactions: 98 },
  { agent: 'Procedural', memories: 145, interactions: 67 },
  { agent: 'Working', memories: 298, interactions: 234 },
];

export function AgentActivityChart({ 
  data: propData,
  autoRefresh = true,
  refreshInterval = 30000
}: AgentActivityChartProps) {
  const [chartData, setChartData] = useState(propData || fallbackData);
  const [loading, setLoading] = useState(false);
  const [isUsingRealData, setIsUsingRealData] = useState(!!propData);

  // Load real data from metrics API and agents
  const loadData = async () => {
    try {
      setLoading(true);
      
      // Try to get data from metrics API first
      const metrics = await apiClient.getMetrics();
      
      if (metrics.agent_activity && metrics.agent_activity.length > 0) {
        setChartData(metrics.agent_activity);
        setIsUsingRealData(true);
      } else {
        // Fallback: get real agents and compute stats
        const agents = await apiClient.getAgents();
        
        if (agents.length > 0) {
          const activityData = await Promise.all(
            agents.map(async (agent) => {
              try {
                const memories = await apiClient.getMemories(agent.id);
                const messages = await apiClient.getChatHistory(agent.id);
                
                return {
                  agent: agent.name || agent.id.slice(0, 8),
                  memories: memories.length,
                  interactions: messages.length
                };
              } catch (error) {
                console.error(`Failed to get data for agent ${agent.id}:`, error);
                return {
                  agent: agent.name || agent.id.slice(0, 8),
                  memories: 0,
                  interactions: 0
                };
              }
            })
          );
          
          setChartData(activityData.filter(d => d.memories > 0 || d.interactions > 0));
          setIsUsingRealData(true);
        } else {
          setIsUsingRealData(false);
        }
      }
    } catch (error) {
      console.error('Failed to load agent activity data:', error);
      setIsUsingRealData(false);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!propData) {
      loadData();
    }

    if (autoRefresh && !propData) {
      const interval = setInterval(loadData, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [propData, autoRefresh, refreshInterval]);

  const totalMemories = chartData.reduce((sum, item) => sum + item.memories, 0);
  const totalInteractions = chartData.reduce((sum, item) => sum + item.interactions, 0);

  return (
    <Card className="shadow-sm hover:shadow-md transition-shadow duration-200">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2">
          <Bot className="w-5 h-5 text-green-600" />
          Agent 活动统计
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
        {chartData.length === 0 ? (
          <div className="h-[300px] flex items-center justify-center text-gray-500">
            <div className="text-center">
              <Bot className="w-12 h-12 mx-auto mb-2 opacity-50" />
              <p>暂无Agent数据</p>
              <p className="text-sm mt-1">创建Agent后数据将显示在这里</p>
            </div>
          </div>
        ) : (
          <>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={chartData}>
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
                <p className="font-semibold text-green-600 dark:text-green-400">
                  {totalMemories}
                </p>
                <p>总记忆数</p>
              </div>
              <div>
                <p className="font-semibold text-green-600 dark:text-green-400">
                  {totalInteractions}
                </p>
                <p>总交互次数</p>
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
