/**
 * Memory Quality Card Component
 * 
 * Displays comprehensive memory quality metrics including:
 * - Average importance
 * - High-quality memory ratio
 * - Importance distribution chart
 * - Memory type distribution
 */

'use client';

import React, { useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell, Legend } from 'recharts';
import { Brain, RefreshCw, TrendingUp, Award, PieChart as PieChartIcon } from 'lucide-react';
import { apiClient, type MemoryQualityResponse } from '@/lib/api-client';

interface MemoryQualityCardProps {
  autoRefresh?: boolean;
  refreshInterval?: number; // in milliseconds
}

// Color palette for charts
const COLORS = {
  purple: '#A855F7',
  blue: '#3B82F6',
  green: '#10B981',
  orange: '#F59E0B',
  red: '#EF4444',
  indigo: '#6366F1',
};

const PIE_COLORS = [COLORS.purple, COLORS.blue, COLORS.green, COLORS.orange, COLORS.indigo, COLORS.red];

export function MemoryQualityCard({ 
  autoRefresh = true,
  refreshInterval = 30000 // 30 seconds
}: MemoryQualityCardProps) {
  const [qualityData, setQualityData] = useState<MemoryQualityResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // ✅ Load data from API
  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await apiClient.getMemoryQuality();
      setQualityData(response);
    } catch (err) {
      console.error('Failed to load memory quality data:', err);
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

  // Prepare chart data
  const importanceChartData = qualityData ? [
    { range: '0.0-0.3 (Low)', count: qualityData.importance_distribution['0.0-0.3'] || 0, fill: COLORS.red },
    { range: '0.3-0.7 (Medium)', count: qualityData.importance_distribution['0.3-0.7'] || 0, fill: COLORS.orange },
    { range: '0.7-1.0 (High)', count: qualityData.importance_distribution['0.7-1.0'] || 0, fill: COLORS.green },
  ] : [];

  const typeChartData = qualityData?.type_distribution.map(type => ({
    name: type.type_name,
    value: type.count,
    percentage: type.percentage,
    avg_importance: type.avg_importance,
  })) || [];

  if (error) {
    return (
      <Card className="shadow-sm bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-lg font-semibold flex items-center gap-2 text-red-400">
            <Brain className="w-5 h-5" />
            Memory Quality Metrics
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-center text-slate-400 py-8">{error}</p>
          <button
            onClick={loadData}
            className="mx-auto block px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 transition-colors"
          >
            Retry
          </button>
        </CardContent>
      </Card>
    );
  }

  if (loading && !qualityData) {
    return (
      <Card className="shadow-sm bg-slate-800/50 border-slate-700">
        <CardHeader>
          <CardTitle className="text-lg font-semibold flex items-center gap-2 text-white">
            <Brain className="w-5 h-5 text-purple-400" />
            Memory Quality Metrics
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-[400px] flex items-center justify-center">
            <RefreshCw className="w-8 h-8 animate-spin text-purple-400" />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="shadow-sm hover:shadow-md transition-shadow duration-200 bg-slate-800/50 border-slate-700">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2 text-white">
          <Brain className="w-5 h-5 text-purple-400" />
          Memory Quality Metrics
          <span className="text-xs text-green-400 font-normal">✅ Live Data</span>
        </CardTitle>
        <button
          onClick={loadData}
          disabled={loading}
          className="p-1 hover:bg-slate-700 rounded transition-colors"
          title="Refresh data"
        >
          <RefreshCw className={`w-4 h-4 text-slate-400 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </CardHeader>
      <CardContent>
        {!qualityData || qualityData.total_memories === 0 ? (
          <div className="h-[400px] flex items-center justify-center text-slate-400">
            <div className="text-center">
              <Brain className="w-12 h-12 mx-auto mb-2 opacity-50" />
              <p>No memories available yet</p>
              <p className="text-sm mt-1">Create some memories to see quality metrics</p>
            </div>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Key Metrics */}
            <div className="grid grid-cols-3 gap-4">
              <div className="text-center p-4 bg-slate-700/50 rounded-lg border border-slate-600">
                <p className="text-sm text-slate-400 mb-1">Avg Importance</p>
                <p className="text-2xl font-bold text-purple-400 flex items-center justify-center gap-1">
                  <Award className="w-5 h-5" />
                  {qualityData.avg_importance.toFixed(2)}
                </p>
              </div>
              <div className="text-center p-4 bg-slate-700/50 rounded-lg border border-slate-600">
                <p className="text-sm text-slate-400 mb-1">High Quality</p>
                <p className="text-2xl font-bold text-green-400 flex items-center justify-center gap-1">
                  <TrendingUp className="w-5 h-5" />
                  {qualityData.high_quality_ratio.toFixed(1)}%
                </p>
              </div>
              <div className="text-center p-4 bg-slate-700/50 rounded-lg border border-slate-600">
                <p className="text-sm text-slate-400 mb-1">Total Memories</p>
                <p className="text-2xl font-bold text-blue-400 flex items-center justify-center gap-1">
                  <Brain className="w-5 h-5" />
                  {qualityData.total_memories}
                </p>
              </div>
            </div>

            {/* Importance Distribution Bar Chart */}
            <div>
              <h4 className="text-sm font-semibold text-slate-300 mb-3 flex items-center gap-2">
                <BarChart className="w-4 h-4" />
                Importance Distribution
              </h4>
              <ResponsiveContainer width="100%" height={200}>
                <BarChart data={importanceChartData}>
                  <CartesianGrid strokeDasharray="3 3" className="stroke-slate-700" />
                  <XAxis 
                    dataKey="range" 
                    tick={{ fill: '#94a3b8', fontSize: 12 }}
                  />
                  <YAxis 
                    tick={{ fill: '#94a3b8', fontSize: 12 }}
                  />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#1e293b',
                      border: '1px solid #475569',
                      borderRadius: '8px',
                      padding: '8px 12px',
                    }}
                    labelStyle={{ color: '#f1f5f9' }}
                  />
                  <Bar 
                    dataKey="count" 
                    radius={[8, 8, 0, 0]}
                  >
                    {importanceChartData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.fill} />
                    ))}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            </div>

            {/* Memory Type Distribution Pie Chart */}
            <div>
              <h4 className="text-sm font-semibold text-slate-300 mb-3 flex items-center gap-2">
                <PieChartIcon className="w-4 h-4" />
                Memory Type Distribution
              </h4>
              <div className="flex items-center gap-4">
                <ResponsiveContainer width="50%" height={200}>
                  <PieChart>
                    <Pie
                      data={typeChartData}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      label={(props: any) => `${props.name}: ${props.percent ? (props.percent * 100).toFixed(1) : 0}%`}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="value"
                    >
                      {typeChartData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={PIE_COLORS[index % PIE_COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip
                      contentStyle={{
                        backgroundColor: '#1e293b',
                        border: '1px solid #475569',
                        borderRadius: '8px',
                      }}
                      formatter={(value: number, name: string, props: any) => [
                        `${value} (${props.payload.percentage.toFixed(1)}%)`,
                        name
                      ]}
                    />
                  </PieChart>
                </ResponsiveContainer>
                <div className="flex-1 space-y-2">
                  {typeChartData.map((type, index) => (
                    <div 
                      key={type.name} 
                      className="flex items-center justify-between p-2 bg-slate-700/30 rounded text-sm"
                    >
                      <div className="flex items-center gap-2">
                        <div 
                          className="w-3 h-3 rounded" 
                          style={{ backgroundColor: PIE_COLORS[index % PIE_COLORS.length] }}
                        />
                        <span className="text-slate-300">{type.name}</span>
                      </div>
                      <div className="text-right">
                        <div className="text-slate-200 font-semibold">{type.value}</div>
                        <div className="text-xs text-slate-400">
                          Imp: {type.avg_importance.toFixed(2)}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

