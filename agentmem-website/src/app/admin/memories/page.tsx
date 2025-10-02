/**
 * Memories Management Page
 * 
 * Allows viewing, searching, and managing agent memories.
 */

'use client';

import React, { useState, useEffect } from 'react';
import { Brain, Search, Trash2, Filter } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { apiClient, Memory, Agent } from '@/lib/api-client';

export default function MemoriesPage() {
  const [memories, setMemories] = useState<Memory[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedAgentId, setSelectedAgentId] = useState<string>('');
  const [selectedType, setSelectedType] = useState<string>('');

  // Load data on mount
  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const agentsData = await apiClient.getAgents();
      setAgents(agentsData);
      
      // Load memories for first agent if available
      if (agentsData.length > 0) {
        const memoriesData = await apiClient.getMemories(agentsData[0].id);
        setMemories(memoriesData);
        setSelectedAgentId(agentsData[0].id);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handleAgentChange = async (agentId: string) => {
    setSelectedAgentId(agentId);
    if (!agentId) {
      setMemories([]);
      return;
    }

    try {
      setLoading(true);
      const data = await apiClient.getMemories(agentId);
      setMemories(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load memories');
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      return;
    }

    try {
      setLoading(true);
      const data = await apiClient.searchMemories(
        searchQuery,
        selectedAgentId || undefined
      );
      setMemories(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to search memories');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteMemory = async (memoryId: string) => {
    if (!confirm('Are you sure you want to delete this memory?')) {
      return;
    }

    try {
      await apiClient.deleteMemory(memoryId);
      setMemories((prev) => prev.filter((m) => m.id !== memoryId));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete memory');
    }
  };

  // Filter memories by type
  const filteredMemories = selectedType
    ? memories.filter((m) => m.memory_type === selectedType)
    : memories;

  // Get unique memory types
  const memoryTypes = Array.from(new Set(memories.map((m) => m.memory_type)));

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
          Memories
        </h2>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          View and manage agent memories
        </p>
      </div>

      {/* Filters */}
      <Card className="p-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Agent Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Agent
            </label>
            <select
              value={selectedAgentId}
              onChange={(e) => handleAgentChange(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            >
              <option value="">All Agents</option>
              {agents.map((agent) => (
                <option key={agent.id} value={agent.id}>
                  {agent.name || `Agent ${agent.id.slice(0, 8)}`}
                </option>
              ))}
            </select>
          </div>

          {/* Type Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Memory Type
            </label>
            <select
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            >
              <option value="">All Types</option>
              {memoryTypes.map((type) => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </select>
          </div>

          {/* Search */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Search
            </label>
            <div className="flex space-x-2">
              <Input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search memories..."
                onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
              />
              <Button onClick={handleSearch}>
                <Search className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </Card>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-red-800 dark:text-red-200">{error}</p>
        </div>
      )}

      {/* Loading State */}
      {loading && (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        </div>
      )}

      {/* Memories List */}
      {!loading && filteredMemories.length === 0 && (
        <Card className="p-12 text-center">
          <Brain className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
            No memories found
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            {selectedAgentId
              ? 'This agent has no memories yet'
              : 'Select an agent to view memories'}
          </p>
        </Card>
      )}

      {!loading && filteredMemories.length > 0 && (
        <div className="space-y-4">
          {filteredMemories.map((memory) => (
            <MemoryCard
              key={memory.id}
              memory={memory}
              onDelete={() => handleDeleteMemory(memory.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Memory Card Component
 */
interface MemoryCardProps {
  memory: Memory;
  onDelete: () => void;
}

function MemoryCard({ memory, onDelete }: MemoryCardProps) {
  const getTypeColor = (type: string) => {
    const colors: Record<string, string> = {
      episodic: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
      semantic: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
      procedural: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      working: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      core: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    };
    return colors[type] || 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
  };

  return (
    <Card className="p-6">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center space-x-3">
          <Brain className="w-5 h-5 text-gray-600 dark:text-gray-400" />
          <span
            className={`px-2 py-1 text-xs font-medium rounded-full ${getTypeColor(
              memory.memory_type
            )}`}
          >
            {memory.memory_type}
          </span>
          <span className="text-xs text-gray-500 dark:text-gray-400">
            Importance: {memory.importance}/10
          </span>
        </div>
        <button
          onClick={onDelete}
          className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>

      <p className="text-gray-900 dark:text-white mb-4">{memory.content}</p>

      <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
        <span>ID: {memory.id.slice(0, 8)}...</span>
        <span>Created: {new Date(memory.created_at).toLocaleString()}</span>
      </div>
    </Card>
  );
}

