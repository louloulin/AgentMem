/**
 * Agent Activity Chart Component
 * 
 * Displays agent activity statistics using Recharts
 * ✅ 100% Real Data from Stats API
 */

'use client';

import React, { useEffect, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Bot, RefreshCw } from 'lucide-react';
import { apiClient, type AgentActivityResponse } from '@/lib/api-client';

interface AgentActivityChartProps {
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export function AgentActivityChart({ 
  autoRefresh = true,
  refreshInterval = 30000
}: AgentActivityChartProps) {
  const [activityData, setActivityData] = useState<AgentActivityResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // ✅ Load real data from new Stats API
  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Use new dedicated Agent Activity API
      const response = await apiClient.getAgentActivity();
      setActivityData(response);
    } catch (err) {
      console.error('Failed to load agent activity data:', err);
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

  // Prepare chart data from API response
  const chartData = activityData?.agents.map(agent => ({
    agent: agent.agent_name,
    memories: agent.total_memories,
    interactions: agent.total_interactions,
  })) || [];

  const totalMemories = chartData.reduce((sum, item) => sum + item.memories, 0);
  const totalInteractions = chartData.reduce((sum, item) => sum + item.interactions, 0);

  if (error) {
    return (
      <Card className="shadow-sm">
        <CardHeader>
          <CardTitle className="text-lg font-semibold flex items-center gap-2 text-red-600">
            <Bot className="w-5 h-5" />
            Agent Activity Statistics
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

  if (loading && !activityData) {
    return (
      <Card className="shadow-sm">
        <CardHeader>
          <CardTitle className="text-lg font-semibold flex items-center gap-2">
            <Bot className="w-5 h-5 text-green-600" />
            Agent Activity Statistics
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-[300px] flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-green-600" />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="shadow-sm hover:shadow-md transition-shadow duration-200">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2">
          <Bot className="w-5 h-5 text-green-600" />
          Agent Activity Statistics
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
              <Bot className="w-12 h-12 mx-auto mb-2 opacity-50" />
              <p>No Agent data available yet</p>
              <p className="text-sm mt-1">Create some agents to see activity</p>
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
                  formatter={(value: number, name: string) => {
                    if (name === 'memories') return [value, 'Memories'];
                    if (name === 'interactions') return [value, 'Interactions'];
                    return [value, name];
                  }}
                />
                <Legend 
                  wrapperStyle={{ 
                    paddingTop: '20px',
                    fontSize: '14px',
                  }}
                />
                <Bar dataKey="memories" fill="#3ECF8E" radius={[4, 4, 0, 0]} name="Memories" />
                <Bar dataKey="interactions" fill="#2CB574" radius={[4, 4, 0, 0]} name="Interactions" />
              </BarChart>
            </ResponsiveContainer>
            <div className="mt-4 grid grid-cols-3 gap-4 text-sm">
              <div className="text-center">
                <p className="text-gray-600 dark:text-gray-400">Total Agents</p>
                <p className="font-semibold text-purple-600 dark:text-purple-400 text-lg">
                  {activityData?.total_agents || 0}
                </p>
              </div>
              <div className="text-center">
                <p className="text-gray-600 dark:text-gray-400">Total Memories</p>
                <p className="font-semibold text-green-600 dark:text-green-400 text-lg">
                  {totalMemories.toLocaleString()}
                </p>
              </div>
              <div className="text-center">
                <p className="text-gray-600 dark:text-gray-400">Total Interactions</p>
                <p className="font-semibold text-blue-600 dark:text-blue-400 text-lg">
                  {totalInteractions.toLocaleString()}
                </p>
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
