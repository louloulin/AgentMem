/**
 * Memory Panel Component
 * 
 * Displays relevant memories in a side panel, inspired by Kimi's UI design.
 */

'use client';

import React, { useState } from 'react';
import { Brain, Loader2, ChevronRight, ChevronLeft } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

// Memory Display Item Interface
export interface MemoryDisplayItem {
  id: string;
  title: string;
  content: string;
  memory_type: string;
  relevance_score: number;
  created_at: string;
  scope: string;
  metadata?: {
    source?: string;
    tags?: string[];
  };
}

interface MemoryPanelProps {
  visible: boolean;
  memories: MemoryDisplayItem[];
  loading: boolean;
  onToggle: () => void;
}

export function MemoryPanel({ visible, memories, loading, onToggle }: MemoryPanelProps) {
  if (!visible) {
    // Collapsed state: only show toggle button
    return (
      <div className="fixed right-0 top-1/2 -translate-y-1/2 z-50">
        <Button
          onClick={onToggle}
          className="rounded-l-lg rounded-r-none shadow-lg hover:shadow-xl transition-shadow"
          variant="outline"
          size="sm"
        >
          <ChevronLeft className="w-4 h-4 mr-1" />
          <Brain className="w-4 h-4 mr-1" />
          <span className="text-sm">相关记忆</span>
          {memories.length > 0 && (
            <Badge variant="secondary" className="ml-2 text-xs">
              {memories.length}
            </Badge>
          )}
        </Button>
      </div>
    );
  }
  
  return (
    <div className="w-96 border-l bg-white dark:bg-gray-900 flex flex-col h-full shadow-lg">
      {/* Header */}
      <div className="border-b dark:border-gray-700 p-4 flex items-center justify-between bg-gradient-to-r from-blue-50 to-purple-50 dark:from-gray-800 dark:to-gray-800">
        <div className="flex items-center gap-2">
          <Brain className="w-5 h-5 text-blue-600 dark:text-blue-400" />
          <h3 className="font-semibold text-gray-900 dark:text-white">相关记忆</h3>
          {memories.length > 0 && (
            <Badge variant="secondary" className="ml-1">
              {memories.length}
            </Badge>
          )}
        </div>
        <Button
          onClick={onToggle}
          size="sm"
          variant="ghost"
          className="hover:bg-white/50 dark:hover:bg-gray-700"
        >
          <ChevronRight className="w-4 h-4" />
        </Button>
      </div>
      
      {/* Memory List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {loading ? (
          <div className="flex flex-col items-center justify-center py-12">
            <Loader2 className="w-8 h-8 animate-spin text-blue-600 dark:text-blue-400 mb-3" />
            <span className="text-sm text-gray-600 dark:text-gray-400">搜索相关记忆中...</span>
          </div>
        ) : memories.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <div className="w-16 h-16 rounded-full bg-gray-100 dark:bg-gray-800 flex items-center justify-center mb-4">
              <Brain className="w-8 h-8 text-gray-400" />
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">暂无相关记忆</p>
            <p className="text-xs text-gray-500 dark:text-gray-500">
              发送消息后自动搜索相关记忆
            </p>
          </div>
        ) : (
          memories.map((memory) => (
            <MemoryCard key={memory.id} memory={memory} />
          ))
        )}
      </div>
    </div>
  );
}

// Memory Card Component
interface MemoryCardProps {
  memory: MemoryDisplayItem;
}

function MemoryCard({ memory }: MemoryCardProps) {
  const [expanded, setExpanded] = useState(false);
  
  // Format relative time
  const relativeTime = formatRelativeTime(memory.created_at);
  
  // Memory type icon mapping
  const typeConfig = {
    'Semantic': { icon: '📚', color: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' },
    'Episodic': { icon: '📝', color: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200' },
    'Procedural': { icon: '⚙️', color: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200' },
    'Working': { icon: '💭', color: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200' },
    'Core': { icon: '⭐', color: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200' },
  };
  
  const config = typeConfig[memory.memory_type as keyof typeof typeConfig] || {
    icon: '📄',
    color: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200',
  };
  
  // Calculate relevance color
  const relevanceColor = memory.relevance_score >= 0.8 
    ? 'text-green-600 dark:text-green-400' 
    : memory.relevance_score >= 0.5 
    ? 'text-blue-600 dark:text-blue-400' 
    : 'text-gray-600 dark:text-gray-400';
  
  return (
    <Card 
      className="p-3 hover:shadow-md dark:hover:shadow-gray-800 transition-all cursor-pointer border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800"
      onClick={() => setExpanded(!expanded)}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2 flex-1 min-w-0">
          <span className="text-lg flex-shrink-0">{config.icon}</span>
          <div className="flex-1 min-w-0">
            <p className="font-medium text-sm truncate text-gray-900 dark:text-white">
              {memory.title}
            </p>
            <div className="flex items-center gap-2 mt-1 flex-wrap">
              <Badge variant="outline" className={`text-xs ${config.color}`}>
                {memory.memory_type}
              </Badge>
              <span className="text-xs text-gray-500 dark:text-gray-400">{relativeTime}</span>
            </div>
          </div>
        </div>
        
        {/* Relevance Score */}
        <div className="ml-2 flex-shrink-0">
          <div className={`text-xs font-semibold ${relevanceColor}`}>
            {(memory.relevance_score * 100).toFixed(0)}%
          </div>
        </div>
      </div>
      
      {/* Expanded Content */}
      {expanded && (
        <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700 animate-fadeIn">
          <p className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap leading-relaxed">
            {memory.content}
          </p>
          
          {/* Metadata */}
          {memory.metadata && (
            <div className="mt-3 pt-3 border-t border-gray-100 dark:border-gray-800">
              <div className="text-xs text-gray-500 dark:text-gray-400 space-y-1">
                {memory.metadata.source && (
                  <div className="flex items-center gap-1">
                    <span className="font-medium">来源:</span>
                    <span>{memory.metadata.source}</span>
                  </div>
                )}
                {memory.metadata.tags && memory.metadata.tags.length > 0 && (
                  <div className="flex gap-1 flex-wrap mt-2">
                    {memory.metadata.tags.map((tag, i) => (
                      <Badge 
                        key={i} 
                        variant="secondary" 
                        className="text-xs px-2 py-0.5"
                      >
                        #{tag}
                      </Badge>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </Card>
  );
}

// Format relative time helper
function formatRelativeTime(timestamp: string): string {
  const now = Date.now();
  const then = new Date(timestamp).getTime();
  const diffMs = now - then;
  
  const seconds = Math.floor(diffMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  
  if (days > 7) return new Date(timestamp).toLocaleDateString('zh-CN');
  if (days > 0) return `${days}天前`;
  if (hours > 0) return `${hours}小时前`;
  if (minutes > 0) return `${minutes}分钟前`;
  if (seconds > 30) return `${seconds}秒前`;
  return '刚刚';
}

