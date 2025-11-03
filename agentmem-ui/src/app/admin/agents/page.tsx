/**
 * Agents Management Page
 * 
 * Allows viewing, creating, and managing agents.
 * Enhanced with Supabase-style modern UI and Toast notifications.
 */

'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import { Bot, Plus, Trash2, Edit, Activity, AlertCircle, Wifi, WifiOff, MessageSquare } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Dialog } from '@/components/ui/dialog';
import { Skeleton } from '@/components/ui/skeleton';
import { Alert } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/hooks/use-toast';
import { apiClient, Agent } from '@/lib/api-client';
import { useWebSocket, WsMessage } from '@/hooks/use-websocket';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || 'http://localhost:8080';
const WS_URL = API_BASE_URL.replace('http', 'ws') + '/api/v1/ws';

export default function AgentsPage() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const { toast } = useToast();

  // Initialize WebSocket connection with token
  const token = typeof window !== 'undefined' ? localStorage.getItem('auth_token') : null;
  const { isConnected: wsConnected, subscribe, lastMessage } = useWebSocket(WS_URL, {
    token: token || undefined,
    autoReconnect: true,
    debug: true,
  });

  // Load agents on mount
  useEffect(() => {
    loadAgents();
  }, []);

  // Handle WebSocket messages for real-time updates
  useEffect(() => {
    const unsubscribe = subscribe('agent_update', async (message: WsMessage) => {
      console.log('[Agents] Received agent_update:', message);
      
      // Show toast notification
      const agentData = message.data as { agent_id?: string; name?: string; action?: string };
      const action = agentData?.action || 'updated';
      const agentName = agentData?.name || 'Agent';
      
      toast({
        title: `Agent ${action}`,
        description: `${agentName} was ${action}`,
      });
      
      // Refresh agent list
      await loadAgents();
    });

    return unsubscribe;
  }, [subscribe, toast]);

  const loadAgents = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.getAgents();
      setAgents(data);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load agents';
      setError(message);
      toast({
        title: 'Error loading agents',
        description: message,
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
    }
  }, [toast]);

  const handleCreateAgent = async (
    name: string,
    description: string,
    llmProvider?: string,
    llmModel?: string
  ) => {
    try {
      const llm_config = llmProvider && llmModel ? {
        provider: llmProvider,
        model: llmModel,
      } : undefined;

      await apiClient.createAgent({ name, description, llm_config });
      setShowCreateDialog(false);
      toast({
        title: 'Agent created',
        description: `Successfully created agent "${name}"${llm_config ? ` with ${llmProvider}/${llmModel}` : ''}`,
      });
      await loadAgents();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to create agent';
      toast({
        title: 'Error creating agent',
        description: message,
        variant: 'destructive',
      });
    }
  };

  const handleDeleteAgent = async (agentId: string, agentName: string) => {
    if (!confirm('Are you sure you want to delete this agent?')) {
      return;
    }

    try {
      await apiClient.deleteAgent(agentId);
      toast({
        title: 'Agent deleted',
        description: `Successfully deleted agent "${agentName}"`,
      });
      await loadAgents();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to delete agent';
      toast({
        title: 'Error deleting agent',
        description: message,
        variant: 'destructive',
      });
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
        <div className="flex items-center space-x-4">
          {/* WebSocket Connection Status */}
          <Badge
            variant={wsConnected ? 'default' : 'secondary'}
            className="flex items-center space-x-1"
          >
            {wsConnected ? <Wifi className="w-3 h-3" /> : <WifiOff className="w-3 h-3" />}
            <span>{wsConnected ? 'Live' : 'Disconnected'}</span>
          </Badge>

          <Button onClick={() => setShowCreateDialog(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Create Agent
          </Button>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <Alert variant="destructive" className="mb-6">
          <AlertCircle className="h-4 w-4" />
          <div>
            <h4 className="font-semibold">Error</h4>
            <p className="text-sm mt-1">{error}</p>
          </div>
        </Alert>
      )}

      {/* Loading State */}
      {loading && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {[1, 2, 3].map((i) => (
            <Card key={i} className="p-6">
              <Skeleton className="h-12 w-12 rounded-lg mb-4" />
              <Skeleton className="h-6 w-3/4 mb-2" />
              <Skeleton className="h-4 w-full mb-4" />
              <Skeleton className="h-9 w-full" />
            </Card>
          ))}
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
              onDelete={() => handleDeleteAgent(agent.id, agent.name || 'Unnamed Agent')}
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
 * Enhanced with click-to-chat functionality
 */
interface AgentCardProps {
  agent: Agent;
  onDelete: () => void;
}

function AgentCard({ agent, onDelete }: AgentCardProps) {
  const router = useRouter();

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

  const handleCardClick = () => {
    router.push(`/admin/chat?agent_id=${agent.id}`);
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent card click when clicking delete
    onDelete();
  };

  return (
    <Card 
      className="p-6 hover:shadow-lg hover:border-purple-500/50 transition-all duration-300 cursor-pointer group relative"
      onClick={handleCardClick}
    >
      {/* Hover overlay with chat icon */}
      <div className="absolute inset-0 bg-purple-500/5 opacity-0 group-hover:opacity-100 transition-opacity rounded-lg pointer-events-none flex items-center justify-center">
        <div className="bg-purple-500 text-white p-3 rounded-full opacity-0 group-hover:opacity-100 transition-all transform scale-75 group-hover:scale-100">
          <MessageSquare className="w-6 h-6" />
        </div>
      </div>

      <div className="flex items-start justify-between mb-4 relative z-10">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-purple-100 dark:bg-purple-900 rounded-lg group-hover:bg-purple-200 dark:group-hover:bg-purple-800 transition-colors">
            <Bot className="w-6 h-6 text-purple-600 dark:text-purple-300" />
          </div>
          <div>
            <h3 className="font-semibold text-gray-900 dark:text-white group-hover:text-purple-600 dark:group-hover:text-purple-400 transition-colors">
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
          onClick={handleDeleteClick}
          className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors relative z-20"
          title="Delete agent"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>

      <p className="text-sm text-gray-600 dark:text-gray-400 mb-4 relative z-10">
        {agent.description || 'No description'}
      </p>

      <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 relative z-10">
        <span>ID: {agent.id.slice(0, 8)}...</span>
        {agent.last_active_at && (
          <span>
            Active: {new Date(agent.last_active_at).toLocaleDateString()}
          </span>
        )}
      </div>

      {/* Click to chat hint */}
      <div className="absolute bottom-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
        <span className="text-xs text-purple-600 dark:text-purple-400 font-medium flex items-center gap-1">
          <MessageSquare className="w-3 h-3" />
          Click to chat
        </span>
      </div>
    </Card>
  );
}

/**
 * Create Agent Dialog Component
 * Enhanced with LLM configuration
 */
interface CreateAgentDialogProps {
  onClose: () => void;
  onCreate: (name: string, description: string, llmProvider?: string, llmModel?: string) => void;
}

function CreateAgentDialog({ onClose, onCreate }: CreateAgentDialogProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [llmProvider, setLlmProvider] = useState('zhipu');
  const [llmModel, setLlmModel] = useState('glm-4-plus');
  const [showAdvanced, setShowAdvanced] = useState(true);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onCreate(name, description, showAdvanced ? llmProvider : undefined, showAdvanced ? llmModel : undefined);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-md p-6 max-h-[90vh] overflow-y-auto">
        <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Create New Agent
        </h3>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Label htmlFor="name">Name *</Label>
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
          
          {/* LLM Configuration Section */}
          <div className="border-t border-gray-200 dark:border-gray-700 pt-4">
            <div className="flex items-center justify-between mb-3">
              <Label className="text-base font-semibold">LLM Configuration</Label>
              <button
                type="button"
                onClick={() => setShowAdvanced(!showAdvanced)}
                className="text-sm text-blue-600 dark:text-blue-400 hover:underline"
              >
                {showAdvanced ? 'Hide' : 'Show'}
              </button>
            </div>
            
            {showAdvanced && (
              <div className="space-y-3 bg-gray-50 dark:bg-gray-800/50 p-4 rounded-lg">
                <div>
                  <Label htmlFor="llmProvider" className="text-sm">
                    Provider
                  </Label>
                  <select
                    id="llmProvider"
                    value={llmProvider}
                    onChange={(e) => setLlmProvider(e.target.value)}
                    className="w-full mt-1 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="zhipu">Zhipu AI (智谱)</option>
                    <option value="openai">OpenAI</option>
                    <option value="anthropic">Anthropic</option>
                    <option value="deepseek">DeepSeek</option>
                  </select>
                </div>
                <div>
                  <Label htmlFor="llmModel" className="text-sm">
                    Model
                  </Label>
                  <Input
                    id="llmModel"
                    value={llmModel}
                    onChange={(e) => setLlmModel(e.target.value)}
                    placeholder="e.g., glm-4-plus"
                    className="text-sm"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    {llmProvider === 'zhipu' && 'Recommended: glm-4-plus, glm-4'}
                    {llmProvider === 'openai' && 'Recommended: gpt-4, gpt-3.5-turbo'}
                    {llmProvider === 'anthropic' && 'Recommended: claude-3-opus, claude-3-sonnet'}
                    {llmProvider === 'deepseek' && 'Recommended: deepseek-chat'}
                  </p>
                </div>
                <div className="text-xs text-gray-600 dark:text-gray-400 bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
                  <strong>Note:</strong> API keys are configured via environment variables (e.g., ZHIPU_API_KEY)
                </div>
              </div>
            )}
          </div>
          
          <div className="flex justify-end space-x-3 pt-2">
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit">Create Agent</Button>
          </div>
        </form>
      </Card>
    </div>
  );
}

