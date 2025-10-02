/**
 * Agents Management Page
 * 
 * Allows viewing, creating, and managing agents.
 */

'use client';

import React, { useState, useEffect } from 'react';
import { Bot, Plus, Trash2, Edit, Activity } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Dialog } from '@/components/ui/dialog';
import { apiClient, Agent } from '@/lib/api-client';

export default function AgentsPage() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  // Load agents on mount
  useEffect(() => {
    loadAgents();
  }, []);

  const loadAgents = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.getAgents();
      setAgents(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load agents');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateAgent = async (name: string, description: string) => {
    try {
      await apiClient.createAgent({ name, description });
      setShowCreateDialog(false);
      await loadAgents();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create agent');
    }
  };

  const handleDeleteAgent = async (agentId: string) => {
    if (!confirm('Are you sure you want to delete this agent?')) {
      return;
    }

    try {
      await apiClient.deleteAgent(agentId);
      await loadAgents();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete agent');
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
            Agents
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Manage your AI agents
          </p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="w-4 h-4 mr-2" />
          Create Agent
        </Button>
      </div>

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

      {/* Agents Grid */}
      {!loading && agents.length === 0 && (
        <Card className="p-12 text-center">
          <Bot className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
            No agents yet
          </h3>
          <p className="text-gray-600 dark:text-gray-400 mb-4">
            Create your first agent to get started
          </p>
          <Button onClick={() => setShowCreateDialog(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Create Agent
          </Button>
        </Card>
      )}

      {!loading && agents.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {agents.map((agent) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              onDelete={() => handleDeleteAgent(agent.id)}
            />
          ))}
        </div>
      )}

      {/* Create Agent Dialog */}
      {showCreateDialog && (
        <CreateAgentDialog
          onClose={() => setShowCreateDialog(false)}
          onCreate={handleCreateAgent}
        />
      )}
    </div>
  );
}

/**
 * Agent Card Component
 */
interface AgentCardProps {
  agent: Agent;
  onDelete: () => void;
}

function AgentCard({ agent, onDelete }: AgentCardProps) {
  const getStateColor = (state: string | null) => {
    switch (state) {
      case 'idle':
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
      case 'thinking':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'executing':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'waiting':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'error':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
    }
  };

  return (
    <Card className="p-6 hover:shadow-lg transition-shadow">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
            <Bot className="w-6 h-6 text-blue-600 dark:text-blue-300" />
          </div>
          <div>
            <h3 className="font-semibold text-gray-900 dark:text-white">
              {agent.name || 'Unnamed Agent'}
            </h3>
            <span
              className={`inline-block px-2 py-1 text-xs font-medium rounded-full mt-1 ${getStateColor(
                agent.state
              )}`}
            >
              {agent.state || 'idle'}
            </span>
          </div>
        </div>
        <button
          onClick={onDelete}
          className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>

      <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
        {agent.description || 'No description'}
      </p>

      <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
        <span>ID: {agent.id.slice(0, 8)}...</span>
        {agent.last_active_at && (
          <span>
            Active: {new Date(agent.last_active_at).toLocaleDateString()}
          </span>
        )}
      </div>
    </Card>
  );
}

/**
 * Create Agent Dialog Component
 */
interface CreateAgentDialogProps {
  onClose: () => void;
  onCreate: (name: string, description: string) => void;
}

function CreateAgentDialog({ onClose, onCreate }: CreateAgentDialogProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onCreate(name, description);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-md p-6">
        <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Create New Agent
        </h3>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Agent"
              required
            />
          </div>
          <div>
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe what this agent does..."
              rows={3}
            />
          </div>
          <div className="flex justify-end space-x-3">
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit">Create</Button>
          </div>
        </form>
      </Card>
    </div>
  );
}

