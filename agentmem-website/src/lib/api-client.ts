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
 * Memory types
 */
export interface Memory {
  id: string;
  agent_id: string;
  memory_type: string;
  content: string;
  metadata: Record<string, any> | null;
  importance: number;
  created_at: string;
  updated_at: string;
}

export interface CreateMemoryRequest {
  agent_id: string;
  memory_type: string;
  content: string;
  metadata?: Record<string, any>;
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
   * Make HTTP request
   */
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
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
}

// Export singleton instance
export const apiClient = new ApiClient();

// Export class for testing
export default ApiClient;

