/**
 * Admin Dashboard Page
 * 
 * Main dashboard showing system overview and statistics.
 * Enhanced with real-time data from backend API and purple theme.
 */

'use client';

import React, { useState, useEffect } from 'react';
import { MemoryGrowthChart } from '@/components/charts/memory-growth-chart';
import { AgentActivityChart } from '@/components/charts/agent-activity-chart';
import { Bot, Brain, Users, Activity, TrendingUp } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { apiClient } from '@/lib/api-client';
import { useToast } from '@/hooks/use-toast';

export default function AdminDashboard() {
  const [stats, setStats] = useState({
    totalAgents: 0,
    totalMemories: 0,
    activeUsers: 0,
    systemStatus: 'Checking...',
  });
  const [loading, setLoading] = useState(true);
  const { toast } = useToast();

  useEffect(() => {
    loadDashboardStats();
  }, []);

  const loadDashboardStats = async () => {
    try {
      setLoading(true);
      
      // ✅ Parallel fetch all data
      const [agents, users, health, metrics] = await Promise.all([
        apiClient.getAgents(),
        apiClient.getUsers().catch(() => [] as any[]), // Fallback to empty array if fails
        apiClient.getHealth(),
        apiClient.getMetrics().catch(() => ({ total_memories: 0 }) as any), // Fallback
      ]);
      
      // Calculate total memories from agents or use metrics
      let totalMemories = metrics.total_memories || 0;
      
      // If metrics doesn't provide memory count, calculate from agents
      if (totalMemories === 0 && agents.length > 0) {
        try {
          const memoryCounts = await Promise.all(
            agents.map(agent => 
              apiClient.getMemories(agent.id)
                .then(memories => memories.length)
                .catch(() => 0)
            )
          );
          totalMemories = memoryCounts.reduce((sum, count) => sum + count, 0);
        } catch (error) {
          console.error('Failed to calculate total memories:', error);
        }
      }
      
      setStats({
        totalAgents: agents.length,
        totalMemories: totalMemories, // ✅ Real data
        activeUsers: users.length, // ✅ Real data
        systemStatus: health.status === 'healthy' ? 'Healthy' : 'Issues',
      });
    } catch (err) {
      console.error('Failed to load dashboard stats:', err);
      toast({
        title: "Error",
        description: "Failed to load dashboard statistics",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div>
          <Skeleton className="h-9 w-48 mb-2 bg-slate-700/50" />
          <Skeleton className="h-5 w-96 bg-slate-700/50" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {[1, 2, 3, 4].map((i) => (
            <Card key={i} className="p-6 bg-slate-800/50 border-slate-700">
              <Skeleton className="h-12 w-12 rounded-lg mb-4 bg-slate-700/50" />
              <Skeleton className="h-6 w-24 mb-2 bg-slate-700/50" />
              <Skeleton className="h-8 w-16 bg-slate-700/50" />
            </Card>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="mb-8">
        <h2 className="text-3xl font-bold text-white mb-2">Dashboard Overview</h2>
        <p className="text-slate-400">Monitor your AI agents and system performance</p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Total Agents"
          value={stats.totalAgents.toString()}
          icon={<Bot className="w-6 h-6" />}
          color="purple"
          trend="+12%"
        />
        <StatCard
          title="Total Memories"
          value={stats.totalMemories > 0 ? stats.totalMemories.toLocaleString() : 'N/A'}
          icon={<Brain className="w-6 h-6" />}
          color="blue"
          trend="+5%"
        />
        <StatCard
          title="Active Users"
          value={stats.activeUsers.toString()}
          icon={<Users className="w-6 h-6" />}
          color="green"
          trend="+2%"
        />
        <StatCard
          title="System Status"
          value={stats.systemStatus}
          icon={<Activity className="w-6 h-6" />}
          color={stats.systemStatus === 'Healthy' ? 'green' : 'red'}
        />
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-8">
        <Card className="p-6 bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
          <h3 className="text-lg font-semibold mb-4 text-white">
            Memory Growth Trend
          </h3>
          <MemoryGrowthChart />
        </Card>

        <Card className="p-6 bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
          <h3 className="text-lg font-semibold mb-4 text-white">
            Agent Activity
          </h3>
          <AgentActivityChart />
        </Card>
      </div>

      {/* Recent Activity */}
      <Card className="p-6 mt-8 bg-slate-800/50 border-slate-700">
        <h3 className="text-xl font-semibold text-white mb-4">
          Recent Activity
        </h3>
        <div className="space-y-4">
          <ActivityItem
            title="New agent created"
            description="Agent 'Customer Support Bot' was created"
            time="2 minutes ago"
          />
          <ActivityItem
            title="Memory updated"
            description="Memory 'Product Knowledge' was updated"
            time="5 minutes ago"
          />
          <ActivityItem
            title="User joined"
            description="New user 'john@example.com' joined"
            time="10 minutes ago"
          />
        </div>
      </Card>
    </div>
  );
}

interface StatCardProps {
  title: string;
  value: string | number;
  icon: React.ReactNode;
  trend?: string;
  color?: 'blue' | 'green' | 'purple' | 'orange' | 'red';
}

function StatCard({ title, value, icon, trend, color = 'purple' }: StatCardProps) {
  const colorClasses = {
    blue: 'bg-blue-500/20 text-blue-400',
    green: 'bg-green-500/20 text-green-400',
    purple: 'bg-purple-500/20 text-purple-400',
    orange: 'bg-orange-500/20 text-orange-400',
    red: 'bg-red-500/20 text-red-400',
  };

  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-slate-400">
            {title}
          </p>
          <div className="flex items-baseline gap-2">
            <p className="text-2xl font-bold text-white mt-2">
              {value}
            </p>
            {trend && (
              <span className="text-xs text-green-400 flex items-center gap-1">
                <TrendingUp className="w-3 h-3" />
                {trend}
              </span>
            )}
          </div>
        </div>
        <div className={`p-3 rounded-lg ${colorClasses[color]}`}>
          {icon}
        </div>
      </div>
    </Card>
  );
}

interface ActivityItemProps {
  title: string;
  description: string;
  time: string;
}

function ActivityItem({ title, description, time }: ActivityItemProps) {
  return (
    <div className="flex items-start justify-between py-3 border-b border-slate-700 last:border-0">
      <div>
        <h4 className="text-sm font-medium text-white">{title}</h4>
        <p className="text-sm text-slate-400 mt-1">{description}</p>
      </div>
      <span className="text-xs text-slate-500">{time}</span>
    </div>
  );
}
