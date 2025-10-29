/**
 * API Client for AgentMem Backend
 * 
 * Provides type-safe methods to interact with the AgentMem API.
 */

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

/**
 * API Response wrapper
 */
interface ApiResponse<T> {
  data: T;
  message?: string;
  error?: string;
}

/**
 * Agent types
 */
export interface Agent {
  id: string;
  organization_id: string;
  name: string | null;
  description: string | null;
  state: string | null;
  last_active_at: string | null;
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateAgentRequest {
  name?: string;
  description?: string;
}

export interface UpdateAgentStateRequest {
  state: 'idle' | 'thinking' | 'executing' | 'waiting' | 'error';
  error_message?: string;
}

export interface AgentStateResponse {
  agent_id: string;
  state: string;
  last_active_at: string | null;
  error_message: string | null;
}

/**
 * Chat types
 */
export interface ChatMessageRequest {
  message: string;
  user_id?: string;
  stream?: boolean;
  metadata?: Record<string, unknown>;
}

export interface ToolCallInfo {
  tool_name: string;
  arguments: Record<string, unknown>;
  result?: string;
}

export interface ChatMessageResponse {
  message_id: string;
  content: string;
  memories_updated: boolean;
  memories_count: number;
  tool_calls?: ToolCallInfo[];
  processing_time_ms: number;
}

export interface ChatHistoryMessage {
  id: string;
  role: string;
  content: string | null;
  created_at: string;
}

/**
 * Memory types
 */
export interface Memory {
  id: string;
  agent_id: string;
  memory_type: string;
  content: string;
  metadata: Record<string, unknown> | null;
  importance: number;
  created_at: string;
  updated_at: string;
}

export interface CreateMemoryRequest {
  agent_id: string;
  memory_type: string;
  content: string;
  metadata?: Record<string, unknown>;
  importance?: number;
}

/**
 * User types
 */
export interface User {
  id: string;
  email: string;
  name: string | null;
  created_at: string;
}

/**
 * API Client class
 */
class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  /**
   * Set authentication token
   */
  setToken(token: string) {
    this.token = token;
  }

  /**
   * Retry helper with exponential backoff
   */
  private async withRetry<T>(
    fn: () => Promise<T>,
    options: {
      retries?: number;
      delay?: number;
      backoff?: number;
    } = {}
  ): Promise<T> {
    const { retries = 3, delay = 1000, backoff = 2 } = options;

    for (let i = 0; i < retries; i++) {
      try {
        return await fn();
      } catch (error) {
        const isLastAttempt = i === retries - 1;
        if (isLastAttempt) {
          throw error;
        }

        // 指数退避
        const waitTime = delay * Math.pow(backoff, i);
        console.log(`API request failed, retrying ${i + 1}/${retries} after ${waitTime}ms...`);
        await new Promise((resolve) => setTimeout(resolve, waitTime));
      }
    }

    throw new Error('Unreachable');
  }

  /**
   * Make HTTP request with retry
   */
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    return this.withRetry(async () => {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...(options.headers as Record<string, string>),
      };

      if (this.token) {
        headers['Authorization'] = `Bearer ${this.token}`;
      }

      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        ...options,
        headers,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({
          error: response.statusText,
        }));
        throw new Error(error.error || `HTTP ${response.status}`);
      }

      return response.json();
    });
  }

  // ==================== Agent APIs ====================

  /**
   * Get all agents
   */
  async getAgents(): Promise<Agent[]> {
    const response = await this.request<ApiResponse<Agent[]>>('/api/v1/agents');
    return response.data;
  }

  /**
   * Get agent by ID
   */
  async getAgent(agentId: string): Promise<Agent> {
    const response = await this.request<ApiResponse<Agent>>(
      `/api/v1/agents/${agentId}`
    );
    return response.data;
  }

  /**
   * Create new agent
   */
  async createAgent(data: CreateAgentRequest): Promise<Agent> {
    const response = await this.request<ApiResponse<Agent>>('/api/v1/agents', {
      method: 'POST',
      body: JSON.stringify(data),
    });
    return response.data;
  }

  /**
   * Update agent
   */
  async updateAgent(agentId: string, data: Partial<Agent>): Promise<Agent> {
    const response = await this.request<ApiResponse<Agent>>(
      `/api/v1/agents/${agentId}`,
      {
        method: 'PUT',
        body: JSON.stringify(data),
      }
    );
    return response.data;
  }

  /**
   * Delete agent
   */
  async deleteAgent(agentId: string): Promise<void> {
    await this.request(`/api/v1/agents/${agentId}`, {
      method: 'DELETE',
    });
  }

  /**
   * Get agent state
   */
  async getAgentState(agentId: string): Promise<AgentStateResponse> {
    const response = await this.request<ApiResponse<AgentStateResponse>>(
      `/api/v1/agents/${agentId}/state`
    );
    return response.data;
  }

  /**
   * Update agent state
   */
  async updateAgentState(
    agentId: string,
    data: UpdateAgentStateRequest
  ): Promise<AgentStateResponse> {
    const response = await this.request<ApiResponse<AgentStateResponse>>(
      `/api/v1/agents/${agentId}/state`,
      {
        method: 'PUT',
        body: JSON.stringify(data),
      }
    );
    return response.data;
  }

  // ==================== Chat APIs ====================

  /**
   * Send chat message to agent
   */
  async sendChatMessage(
    agentId: string,
    data: ChatMessageRequest
  ): Promise<ChatMessageResponse> {
    const response = await this.request<ApiResponse<ChatMessageResponse>>(
      `/api/v1/agents/${agentId}/chat`,
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    );
    return response.data;
  }

  /**
   * Get chat history for agent
   */
  async getChatHistory(agentId: string): Promise<ChatHistoryMessage[]> {
    const response = await this.request<ApiResponse<ChatHistoryMessage[]>>(
      `/api/v1/agents/${agentId}/chat/history`
    );
    return response.data;
  }

  // ==================== Memory APIs ====================

  /**
   * Get memories for an agent
   */
  async getMemories(agentId: string): Promise<Memory[]> {
    const response = await this.request<ApiResponse<Memory[]>>(
      `/api/v1/agents/${agentId}/memories`
    );
    return response.data;
  }

  /**
   * Create new memory
   */
  async createMemory(data: CreateMemoryRequest): Promise<Memory> {
    const response = await this.request<ApiResponse<Memory>>(
      '/api/v1/memories',
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    );
    return response.data;
  }

  /**
   * Delete memory
   */
  async deleteMemory(memoryId: string): Promise<void> {
    await this.request(`/api/v1/memories/${memoryId}`, {
      method: 'DELETE',
    });
  }

  /**
   * Search memories
   */
  async searchMemories(query: string, agentId?: string): Promise<Memory[]> {
    const params = new URLSearchParams({ query });
    if (agentId) {
      params.append('agent_id', agentId);
    }
    const response = await this.request<ApiResponse<Memory[]>>(
      `/api/v1/memories/search?${params}`
    );
    return response.data;
  }

  // ==================== User APIs ====================

  /**
   * Get all users
   */
  async getUsers(): Promise<User[]> {
    const response = await this.request<ApiResponse<User[]>>('/api/v1/users');
    return response.data;
  }

  /**
   * Get current user
   */
  async getCurrentUser(): Promise<User> {
    const response = await this.request<ApiResponse<User>>('/api/v1/users/me');
    return response.data;
  }

  // ==================== Extended Memory APIs ====================

  /**
   * Update memory
   */
  async updateMemory(memoryId: string, data: Partial<Memory>): Promise<Memory> {
    const response = await this.request<ApiResponse<Memory>>(
      `/api/v1/memories/${memoryId}`,
      {
        method: 'PUT',
        body: JSON.stringify(data),
      }
    );
    return response.data;
  }

  /**
   * Get single memory
   */
  async getMemory(memoryId: string): Promise<Memory> {
    const response = await this.request<ApiResponse<Memory>>(
      `/api/v1/memories/${memoryId}`
    );
    return response.data;
  }

  // ==================== Health & Metrics APIs ====================

  /**
   * Get system health
   */
  async getHealth(): Promise<HealthResponse> {
    const response = await this.request<HealthResponse>('/health');
    return response;
  }

  /**
   * Get system metrics
   */
  async getMetrics(): Promise<MetricsResponse> {
    const response = await this.request<MetricsResponse>('/metrics');
    return response;
  }
}

// ==================== Additional Types ====================

export interface HealthResponse {
  status: string;
  timestamp: string;
  components?: Record<string, ComponentStatus>;
}

export interface ComponentStatus {
  status: string;
  message?: string;
}

export interface MetricsResponse {
  total_memories?: number;
  total_agents?: number;
  total_users?: number;
  avg_response_time_ms?: number;
  active_connections?: number;
  // Memory growth data for charts
  memory_growth?: Array<{
    date: string;
    count: number;
  }>;
  // Agent activity data for charts
  agent_activity?: Array<{
    agent: string;
    memories: number;
    interactions: number;
  }>;
}

// Export singleton instance
export const apiClient = new ApiClient();

// Export class for testing
export default ApiClient;

