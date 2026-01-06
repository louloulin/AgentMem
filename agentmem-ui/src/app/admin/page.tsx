/**
 * Admin Dashboard Page
 * 
 * Main dashboard showing system overview and statistics.
 * Enhanced with real-time data from backend API and purple theme.
 */

'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { MemoryGrowthChart } from '@/components/charts/memory-growth-chart';
import { AgentActivityChart } from '@/components/charts/agent-activity-chart';
import { MemoryQualityCard } from '@/components/stats/MemoryQualityCard';
import { Bot, Brain, Users, Activity, TrendingUp, MessageSquare, Wifi, WifiOff } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Badge } from '@/components/ui/badge';
import { apiClient, type DashboardStats, type ActivityLog } from '@/lib/api-client';
import { useToast } from '@/hooks/use-toast';
import { useWebSocket, type WsMessage } from '@/hooks/use-websocket';

export default function AdminDashboard() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const { toast } = useToast();

  // ✅ WebSocket for real-time updates
  const API_BASE_URL = typeof window !== 'undefined' 
    ? (process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080')
    : 'http://localhost:8080';
    
  const WS_URL = API_BASE_URL.replace(/^http/, 'ws') + '/api/v1/ws';
  
  const ws = useWebSocket(WS_URL, {
    token: typeof window !== 'undefined' ? localStorage.getItem('auth_token') || undefined : undefined,
    autoReconnect: true,
    maxReconnectAttempts: 5,
    heartbeatInterval: 30000,
    debug: true, // Enable debug mode for development
  });

  // Handle WebSocket messages
  const handleWebSocketMessage = useCallback((message: WsMessage) => {
    console.log('[Dashboard] WebSocket message:', message);
    
    switch (message.type) {
      case 'agent_update':
        // Agent status changed
        toast({
          title: "Agent Updated",
          description: `Agent ${message.data ? (message.data as { agent_id?: string }).agent_id : 'Unknown'} status changed`,
        });
        // Refresh dashboard stats
        loadDashboardStats();
        break;
        
      case 'memory_update':
        // Memory added/updated/deleted
        toast({
          title: "Memory Updated",
          description: "A memory has been updated",
        });
        // Refresh dashboard stats
        loadDashboardStats();
        break;
        
      case 'message':
        // New chat message
        toast({
          title: "New Message",
          description: "A new message has been received",
        });
        break;
        
      case 'error':
        // Error notification
        toast({
          title: "Error",
          description: message.data ? String(message.data) : "An error occurred",
          variant: "destructive",
        });
        break;
    }
  }, [toast]);

  // Subscribe to WebSocket messages
  useEffect(() => {
    const unsubscribe = ws.subscribe('*', handleWebSocketMessage);
    return unsubscribe;
  }, [ws, handleWebSocketMessage]);

  useEffect(() => {
    loadDashboardStats();
    
    // Auto-refresh every 30 seconds (as fallback if WebSocket is disconnected)
    const interval = setInterval(loadDashboardStats, 30000);
    return () => clearInterval(interval);
  }, []);

  const loadDashboardStats = async () => {
    try {
      setLoading(true);
      
      // ✅ Use new unified Stats API - 100% real data
      const dashboardStats = await apiClient.getDashboardStats();
      
      setStats(dashboardStats);
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
      {/* Page Header with WebSocket Status */}
      <div className="mb-8 flex items-start justify-between">
        <div>
          <h2 className="text-3xl font-bold text-white mb-2">Dashboard Overview</h2>
          <p className="text-slate-400">Monitor your AI agents and system performance</p>
        </div>
        
        {/* ✅ WebSocket Connection Status Indicator */}
        <div className="flex items-center gap-2">
          {ws.isConnected ? (
            <Badge variant="default" className="bg-green-600 hover:bg-green-700">
              <Wifi className="w-3 h-3 mr-1" />
              Live Updates
            </Badge>
          ) : ws.isReconnecting ? (
            <Badge variant="secondary" className="bg-yellow-600 hover:bg-yellow-700">
              <Activity className="w-3 h-3 mr-1 animate-pulse" />
              Reconnecting... ({ws.reconnectAttempts}/{5})
            </Badge>
          ) : (
            <Badge variant="destructive">
              <WifiOff className="w-3 h-3 mr-1" />
              Offline
            </Badge>
          )}
        </div>
      </div>

      {/* Stats Grid - ✅ 100% Real Data from Stats API */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Total Agents"
          value={stats?.total_agents.toString() || '0'}
          icon={<Bot className="w-6 h-6" />}
          color="purple"
          subtitle={`${stats?.active_agents || 0} active (24h)`}
        />
        <StatCard
          title="Total Memories"
          value={stats?.total_memories.toLocaleString() || '0'}
          icon={<Brain className="w-6 h-6" />}
          color="blue"
          subtitle={`${Object.keys(stats?.memories_by_type || {}).length} types`}
        />
        <StatCard
          title="Total Users"
          value={stats?.total_users.toString() || '0'}
          icon={<Users className="w-6 h-6" />}
          color="green"
          subtitle={`${stats?.active_users || 0} active (24h)`}
        />
        <StatCard
          title="Total Messages"
          value={stats?.total_messages.toLocaleString() || '0'}
          icon={<MessageSquare className="w-6 h-6" />}
          color="orange"
          subtitle={`${stats?.avg_response_time_ms.toFixed(0)}ms avg`}
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

      {/* Memory Quality Metrics - 🆕 New Feature */}
      <div className="mt-8">
        <MemoryQualityCard />
      </div>

      {/* Recent Activity - ✅ 100% Real Data from Stats API */}
      <Card className="p-6 mt-8 bg-slate-800/50 border-slate-700">
        <h3 className="text-xl font-semibold text-white mb-4 flex items-center justify-between">
          <span>Recent Activity</span>
          <span className="text-xs text-slate-500">
            Last updated: {stats?.timestamp ? new Date(stats.timestamp).toLocaleTimeString() : 'N/A'}
          </span>
        </h3>
        <div className="space-y-4">
          {stats?.recent_activities && stats.recent_activities.length > 0 ? (
            stats.recent_activities.map((activity) => (
              <ActivityItem
                key={activity.id}
                title={formatActivityTitle(activity.activity_type)}
                description={activity.description}
                time={formatTimeAgo(activity.timestamp)}
                activityType={activity.activity_type}
              />
            ))
          ) : (
            <p className="text-slate-500 text-center py-8">No recent activity</p>
          )}
        </div>
      </Card>
    </div>
  );
}

interface StatCardProps {
  title: string;
  value: string | number;
  icon: React.ReactNode;
  subtitle?: string;
  color?: 'blue' | 'green' | 'purple' | 'orange' | 'red';
}

function StatCard({ title, value, icon, subtitle, color = 'purple' }: StatCardProps) {
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
        <div className="flex-1">
          <p className="text-sm font-medium text-slate-400">
            {title}
          </p>
          <p className="text-2xl font-bold text-white mt-2">
            {value}
          </p>
          {subtitle && (
            <p className="text-xs text-slate-500 mt-1">
              {subtitle}
            </p>
          )}
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
  activityType: string;
}

function ActivityItem({ title, description, time, activityType }: ActivityItemProps) {
  const getActivityIcon = (type: string) => {
    if (type.includes('message')) return '💬';
    if (type.includes('memory')) return '🧠';
    if (type.includes('agent')) return '🤖';
    if (type.includes('user')) return '👤';
    return '📋';
  };

  return (
    <div className="flex items-start justify-between py-3 border-b border-slate-700 last:border-0 hover:bg-slate-700/30 transition-colors px-2 rounded">
      <div className="flex items-start gap-3">
        <span className="text-lg mt-0.5">{getActivityIcon(activityType)}</span>
        <div>
          <h4 className="text-sm font-medium text-white">{title}</h4>
          <p className="text-sm text-slate-400 mt-1">{description}</p>
        </div>
      </div>
      <span className="text-xs text-slate-500 whitespace-nowrap ml-4">{time}</span>
    </div>
  );
}

// ✅ Helper functions for real data formatting
function formatActivityTitle(activityType: string): string {
  const titles: Record<string, string> = {
    'message_sent': 'New Message',
    'memory_created': 'Memory Created',
    'memory_updated': 'Memory Updated',
    'agent_created': 'Agent Created',
    'user_joined': 'User Joined',
  };
  return titles[activityType] || activityType.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
}

function formatTimeAgo(timestamp: string): string {
  const now = new Date();
  const then = new Date(timestamp);
  const diffMs = now.getTime() - then.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return `${diffSec}s ago`;
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  return then.toLocaleDateString();
}
