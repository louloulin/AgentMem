/**
 * Admin Dashboard Page
 * 
 * Main dashboard showing system overview and statistics.
 * Enhanced with real-time data from backend API.
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
      
      // Fetch agents count
      const agents = await apiClient.getAgents();
      
      // Fetch system health
      const healthResponse = await fetch('http://localhost:8080/health');
      const health = await healthResponse.json();
      
      setStats({
        totalAgents: agents.length,
        totalMemories: 0, // Will be updated when memories API is ready
        activeUsers: 1, // Placeholder
        systemStatus: health.status === 'healthy' ? 'Healthy' : 'Issues',
      });
    } catch (err) {
      toast({
        title: 'Error loading dashboard',
        description: err instanceof Error ? err.message : 'Failed to load dashboard data',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div>
          <Skeleton className="h-9 w-48 mb-2" />
          <Skeleton className="h-5 w-96" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {[1, 2, 3, 4].map((i) => (
            <Card key={i} className="p-6">
              <Skeleton className="h-12 w-12 rounded-lg mb-4" />
              <Skeleton className="h-6 w-24 mb-2" />
              <Skeleton className="h-8 w-16" />
            </Card>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
          Dashboard
        </h2>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Welcome to AgentMem Admin Dashboard
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Total Agents"
          value={stats.totalAgents.toString()}
          icon={<Bot className="w-6 h-6" />}
          color="blue"
          trend="+12%"
        />
        <StatCard
          title="Total Memories"
          value={stats.totalMemories > 0 ? stats.totalMemories.toLocaleString() : 'N/A'}
          icon={<Brain className="w-6 h-6" />}
          color="purple"
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
          color={stats.systemStatus === 'Healthy' ? 'emerald' : 'red'}
        />
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-8">
        <MemoryGrowthChart />
        <AgentActivityChart />
      </div>

      {/* Recent Activity */}
      <Card className="p-6 mt-8">
        <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Recent Activity
        </h3>
        <div className="space-y-4">
          <ActivityItem
            title="New agent created"
            description="Agent 'Customer Support Bot' was created"
            time="2 minutes ago"
          />
          <ActivityItem
            title="Memory added"
            description="New episodic memory added to Agent 'Research Assistant'"
            time="15 minutes ago"
          />
          <ActivityItem
            title="User registered"
            description="New user 'john@example.com' registered"
            time="1 hour ago"
          />
        </div>
      </Card>
    </div>
  );
}

/**
 * Stat Card Component
 * Enhanced with trend indicators
 */
interface StatCardProps {
  title: string;
  value: string;
  icon: React.ReactNode;
  color: 'blue' | 'purple' | 'green' | 'emerald' | 'red';
  trend?: string;
}

function StatCard({ title, value, icon, color, trend }: StatCardProps) {
  const colorClasses = {
    blue: 'bg-blue-100 text-blue-600 dark:bg-blue-900 dark:text-blue-300',
    purple: 'bg-purple-100 text-purple-600 dark:bg-purple-900 dark:text-purple-300',
    green: 'bg-green-100 text-green-600 dark:bg-green-900 dark:text-green-300',
    emerald: 'bg-emerald-100 text-emerald-600 dark:bg-emerald-900 dark:text-emerald-300',
    red: 'bg-red-100 text-red-600 dark:bg-red-900 dark:text-red-300',
  };

  return (
    <Card className="p-6 hover:shadow-lg transition-all duration-300">
      <div className="flex items-center justify-between mb-2">
        <p className="text-sm font-medium text-gray-600 dark:text-gray-400">{title}</p>
        {trend && (
          <div className="flex items-center text-xs text-green-600 dark:text-green-400">
            <TrendingUp className="w-3 h-3 mr-1" />
            {trend}
          </div>
        )}
      </div>
      <div className="flex items-center justify-between">
        <p className="text-3xl font-bold text-gray-900 dark:text-white">
          {value}
        </p>
        <div className={`p-3 rounded-lg ${colorClasses[color]}`}>
          {icon}
        </div>
      </div>
    </Card>
  );
}

/**
 * Activity Item Component
 */
interface ActivityItemProps {
  title: string;
  description: string;
  time: string;
}

function ActivityItem({ title, description, time }: ActivityItemProps) {
  return (
    <div className="flex items-start space-x-3 pb-4 border-b border-gray-200 dark:border-gray-700 last:border-0 last:pb-0">
      <div className="flex-shrink-0 w-2 h-2 mt-2 bg-blue-600 rounded-full" />
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-gray-900 dark:text-white">
          {title}
        </p>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          {description}
        </p>
        <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
          {time}
        </p>
      </div>
    </div>
  );
}

