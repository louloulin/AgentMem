/**
 * Chat Interface Page
 * 
 * Provides a chat interface to interact with agents.
 */

'use client';

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Send, Bot, User, Loader2, Zap } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { apiClient, Agent } from '@/lib/api-client';
import { useSSE } from '@/hooks/use-sse';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || 'http://localhost:8080';

interface Message {
  id: string;
  role: 'user' | 'agent';
  content: string;
  timestamp: Date;
  isStreaming?: boolean; // 标识是否正在流式接收
}

export default function ChatPage() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState<string>('');
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [useStreaming, setUseStreaming] = useState(true); // 是否启用流式响应
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Initialize SSE connection with token
  const token = typeof window !== 'undefined' ? localStorage.getItem('auth_token') : null;
  const { isConnected: sseConnected } = useSSE(`${API_BASE_URL}/api/v1/sse`, {
    token: token || undefined,
    debug: true,
  });

  // Load agents on mount
  useEffect(() => {
    loadAgents();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Load chat history when agent changes
  useEffect(() => {
    if (selectedAgentId) {
      loadChatHistory();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedAgentId]);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const loadAgents = async () => {
    try {
      const data = await apiClient.getAgents();
      setAgents(data);
      if (data.length > 0 && !selectedAgentId) {
        setSelectedAgentId(data[0].id);
      }
    } catch (err) {
      console.error('Failed to load agents:', err);
    }
  };

  const loadChatHistory = async () => {
    if (!selectedAgentId) return;

    try {
      const history = await apiClient.getChatHistory(selectedAgentId);
      const loadedMessages: Message[] = history.map((msg) => ({
        id: msg.id,
        role: msg.role === 'user' ? 'user' : 'agent',
        content: msg.content || '',
        timestamp: new Date(msg.created_at),
      }));
      
      // ✅ 按时间戳排序（从旧到新）
      loadedMessages.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
      
      setMessages(loadedMessages);
    } catch (err) {
      console.error('Failed to load chat history:', err);
    }
  };

  // Handle streaming chat message via SSE
  const handleStreamingMessage = useCallback(async (messageContent: string) => {
    if (!selectedAgentId) return;

    const agentMessageId = `agent-${Date.now()}`;
    
    // Add empty agent message with streaming flag
    const agentMessage: Message = {
      id: agentMessageId,
      role: 'agent',
      content: '',
      timestamp: new Date(),
      isStreaming: true,
    };
    setMessages((prev) => [...prev, agentMessage]);

    try {
      const url = `${API_BASE_URL}/api/v1/agents/${selectedAgentId}/chat/stream`;
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({
          message: messageContent,
          user_id: 'default', // Add user_id for tenant isolation
          stream: true,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      if (!reader) {
        throw new Error('No response body');
      }

      let accumulatedContent = '';

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split('\n');

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6).trim();
            if (!data || data === 'keep-alive') continue;

            try {
              const parsed = JSON.parse(data);
              
              if (parsed.chunk_type === 'content' && parsed.content) {
                accumulatedContent += parsed.content;
                
                // Update message content
                setMessages((prev) =>
                  prev.map((msg) =>
                    msg.id === agentMessageId
                      ? { ...msg, content: accumulatedContent }
                      : msg
                  )
                );
              } else if (parsed.chunk_type === 'done') {
                // Mark streaming as complete
                setMessages((prev) =>
                  prev.map((msg) =>
                    msg.id === agentMessageId
                      ? { ...msg, isStreaming: false }
                      : msg
                  )
                );
              } else if (parsed.chunk_type === 'error') {
                throw new Error(parsed.content || 'Unknown error');
              }
            } catch (parseErr) {
              console.error('Failed to parse SSE data:', parseErr);
            }
          }
        }
      }
    } catch (err) {
      console.error('Streaming error:', err);
      
      // Update with error message
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === agentMessageId
            ? {
                ...msg,
                content: `Error: ${err instanceof Error ? err.message : 'Failed to stream response'}`,
                isStreaming: false,
              }
            : msg
        )
      );
    }
  }, [selectedAgentId, token]);

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || !selectedAgentId || loading) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input,
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    const messageContent = input;
    setInput('');
    setLoading(true);

    try {
      if (useStreaming) {
        // Use SSE streaming
        await handleStreamingMessage(messageContent);
      } else {
        // Use regular API call
        const response = await apiClient.sendChatMessage(selectedAgentId, {
          message: messageContent,
          user_id: 'default', // Add user_id for tenant isolation
        });

        const agentMessage: Message = {
          id: response.message_id,
          role: 'agent',
          content: response.content,
          timestamp: new Date(),
        };

        setMessages((prev) => [...prev, agentMessage]);
      }
    } catch (err) {
      console.error('Failed to send message:', err);

      // Show error message to user
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'agent',
        content: `Error: ${err instanceof Error ? err.message : 'Failed to send message'}. Please try again.`,
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setLoading(false);
    }
  };

  const selectedAgent = agents.find((a) => a.id === selectedAgentId);

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
            Chat
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Interact with your agents
          </p>
        </div>
        <div className="flex items-center space-x-4">
          {/* SSE Connection Status */}
          <Badge
            variant={sseConnected ? 'default' : 'secondary'}
            className="flex items-center space-x-1"
          >
            <Zap className="w-3 h-3" />
            <span>{sseConnected ? 'SSE Connected' : 'SSE Disconnected'}</span>
          </Badge>

          {/* Streaming Toggle */}
          <label className="flex items-center space-x-2 cursor-pointer">
            <input
              type="checkbox"
              checked={useStreaming}
              onChange={(e) => setUseStreaming(e.target.checked)}
              className="w-4 h-4"
            />
            <span className="text-sm text-gray-700 dark:text-gray-300">
              Stream responses
            </span>
          </label>

          {/* Agent Selector */}
          <div className="w-64">
            <select
              value={selectedAgentId}
              onChange={(e) => setSelectedAgentId(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            >
              <option value="">Select an agent...</option>
              {agents.map((agent) => (
                <option key={agent.id} value={agent.id}>
                  {agent.name || `Agent ${agent.id.slice(0, 8)}`}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Chat Container */}
      <Card className="flex-1 flex flex-col overflow-hidden">
        {!selectedAgentId ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <Bot className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                Select an agent to start chatting
              </h3>
              <p className="text-gray-600 dark:text-gray-400">
                Choose an agent from the dropdown above
              </p>
            </div>
          </div>
        ) : (
          <>
            {/* Agent Info */}
            {selectedAgent && (
              <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                <div className="flex items-center space-x-3">
                  <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
                    <Bot className="w-5 h-5 text-blue-600 dark:text-blue-300" />
                  </div>
                  <div>
                    <h3 className="font-semibold text-gray-900 dark:text-white">
                      {selectedAgent.name || 'Unnamed Agent'}
                    </h3>
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      {selectedAgent.state || 'idle'}
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {messages.length === 0 && (
                <div className="text-center text-gray-500 dark:text-gray-400 py-8">
                  No messages yet. Start a conversation!
                </div>
              )}
              {messages.map((message) => (
                <MessageBubble key={message.id} message={message} />
              ))}
              {loading && (
                <div className="flex items-center space-x-2 text-gray-500">
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span className="text-sm">Agent is thinking...</span>
                </div>
              )}
              <div ref={messagesEndRef} />
            </div>

            {/* Input */}
            <div className="p-4 border-t border-gray-200 dark:border-gray-700">
              <form onSubmit={handleSendMessage} className="flex space-x-2">
                <Input
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  placeholder="Type your message..."
                  disabled={loading}
                  className="flex-1"
                />
                <Button type="submit" disabled={loading || !input.trim()}>
                  <Send className="w-4 h-4" />
                </Button>
              </form>
            </div>
          </>
        )}
      </Card>
    </div>
  );
}

/**
 * Message Bubble Component
 */
interface MessageBubbleProps {
  message: Message;
}

function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === 'user';

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`flex items-start space-x-2 max-w-[70%] ${
          isUser ? 'flex-row-reverse space-x-reverse' : ''
        }`}
      >
        <div
          className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
            isUser
              ? 'bg-blue-600 text-white'
              : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300'
          }`}
        >
          {isUser ? (
            <User className="w-4 h-4" />
          ) : (
            <Bot className="w-4 h-4" />
          )}
        </div>
        <div>
          <div
            className={`rounded-lg px-4 py-2 ${
              isUser
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white'
            }`}
          >
            <p className="text-sm whitespace-pre-wrap">
              {message.content}
              {message.isStreaming && (
                <span className="inline-flex items-center ml-2">
                  <Loader2 className="w-3 h-3 animate-spin" />
                </span>
              )}
            </p>
            {!message.content && message.isStreaming && (
              <div className="flex items-center space-x-1 text-gray-500">
                <Loader2 className="w-3 h-3 animate-spin" />
                <span className="text-xs">Streaming...</span>
              </div>
            )}
          </div>
          <div className="flex items-center space-x-2 mt-1 px-1">
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {message.timestamp.toLocaleTimeString()}
            </p>
            {message.isStreaming && (
              <Badge variant="secondary" className="text-xs px-1 py-0">
                <Zap className="w-2 h-2 mr-1" />
                Live
              </Badge>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

