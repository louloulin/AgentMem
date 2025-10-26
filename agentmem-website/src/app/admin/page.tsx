/**
 * Admin Dashboard Page
 * 
 * Main dashboard showing system overview and statistics.
 */

'use client';

import React from 'react';
import { MemoryGrowthChart } from '@/components/charts/memory-growth-chart';
import { AgentActivityChart } from '@/components/charts/agent-activity-chart';
import { Bot, Brain, Users, Activity } from 'lucide-react';
import { Card } from '@/components/ui/card';

export default function AdminDashboard() {
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
          value="12"
          icon={<Bot className="w-6 h-6" />}
          color="blue"
        />
        <StatCard
          title="Total Memories"
          value="1,234"
          icon={<Brain className="w-6 h-6" />}
          color="purple"
        />
        <StatCard
          title="Active Users"
          value="45"
          icon={<Users className="w-6 h-6" />}
          color="green"
        />
        <StatCard
          title="System Status"
          value="Healthy"
          icon={<Activity className="w-6 h-6" />}
          color="emerald"
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
 */
interface StatCardProps {
  title: string;
  value: string;
  icon: React.ReactNode;
  color: 'blue' | 'purple' | 'green' | 'emerald';
}

function StatCard({ title, value, icon, color }: StatCardProps) {
  const colorClasses = {
    blue: 'bg-blue-100 text-blue-600 dark:bg-blue-900 dark:text-blue-300',
    purple: 'bg-purple-100 text-purple-600 dark:bg-purple-900 dark:text-purple-300',
    green: 'bg-green-100 text-green-600 dark:bg-green-900 dark:text-green-300',
    emerald: 'bg-emerald-100 text-emerald-600 dark:bg-emerald-900 dark:text-emerald-300',
  };

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-600 dark:text-gray-400">{title}</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
            {value}
          </p>
        </div>
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

