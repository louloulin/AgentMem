/**
 * API Client for AgentMem Backend
 *
 * Provides type-safe methods to interact with the AgentMem API.
 *
 * Features:
 * - Type-safe API methods
 * - Automatic retries with exponential backoff
 * - Client-side caching with TTL
 * - Request deduplication
 */

import { DEFAULT_USER_ID, DEFAULT_ORG_ID, normalizeUserId } from './constants';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

/**
 * Cache entry with TTL
 */
interface CacheEntry<T> {
  data: T;
  expiry: number;
  timestamp: number;
}

/**
 * Cache statistics for monitoring
 */
interface CacheStats {
  hits: number;
  misses: number;
  size: number;
  hitRate: number;
}

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
  llm_config?: {
    provider: string;
    model: string;
    api_key?: string;
  };
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
 * Working Memory types
 */
export interface WorkingMemoryItem {
  id: string;
  user_id: string;
  agent_id: string;
  session_id: string;
  content: string;
  priority: number;
  expires_at?: string | null;
  metadata: Record<string, unknown> | null;
  created_at: string;
}

export interface AddWorkingMemoryRequest {
  session_id: string;
  content: string;
  priority?: number;
  expires_in_seconds?: number;
  metadata?: Record<string, unknown>;
}

export interface ClearWorkingMemoryResponse {
  deleted_count: number;
  session_id: string;
}

export interface CleanupWorkingMemoryResponse {
  cleaned_count: number;
}

/**
 * User types
 */
export interface User {
  id: string;
  email: string;
  name: string | null;
  organization_id?: string;
  roles?: string[];
  created_at: string;
}

export interface UsersListResponse {
  users: User[];
  total: number;
  page: number;
  page_size: number;
}

/**
 * Statistics types
 */
export interface DashboardStats {
  total_agents: number;
  total_users: number;
  total_memories: number;
  total_messages: number;
  active_agents: number;
  active_users: number;
  avg_response_time_ms: number;
  recent_activities: ActivityLog[];
  memories_by_type: Record<string, number>;
  timestamp: string;
}

export interface ActivityLog {
  id: string;
  activity_type: string;
  agent_id?: string;
  user_id?: string;
  description: string;
  timestamp: string;
}

export interface MemoryGrowthResponse {
  data: MemoryGrowthPoint[];
  total_memories: number;
  growth_rate: number;
  timestamp: string;
}

export interface MemoryGrowthPoint {
  date: string;
  total: number;
  new: number;
  by_type: Record<string, number>;
}

export interface AgentActivityResponse {
  agents: AgentActivityStats[];
  total_agents: number;
  timestamp: string;
}

export interface MemoryTypeStats {
  type_name: string;
  count: number;
  percentage: number;
  avg_importance: number;
}

export interface MemoryQualityResponse {
  avg_importance: number;
  high_quality_ratio: number;
  importance_distribution: Record<string, number>;
  type_distribution: MemoryTypeStats[];
  total_memories: number;
  access_stats: Record<string, number>;
  timestamp: string;
}

export interface AgentActivityStats {
  agent_id: string;
  agent_name: string;
  total_memories: number;
  total_interactions: number;
  last_active?: string;
  avg_importance: number;
}

/**
 * API Client class
 */
class ApiClient {
  private baseUrl: string;
  private token: string | null = null;
  
  // ==================== Cache System ====================
  private cache: Map<string, CacheEntry<unknown>> = new Map();
  private readonly DEFAULT_TTL = 30000; // 30 seconds
  private cacheStats = {
    hits: 0,
    misses: 0
  };

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
    
    // Clean expired cache entries every minute
    if (typeof window !== 'undefined') {
      setInterval(() => this.cleanExpiredCache(), 60000);
    }
  }

  /**
   * Set authentication token
   */
  setToken(token: string) {
    this.token = token;
  }

  /**
   * Get cached data if available and not expired
   */
  private getCached<T>(key: string): T | null {
    const cached = this.cache.get(key);
    if (!cached) {
      this.cacheStats.misses++;
      return null;
    }

    if (cached.expiry < Date.now()) {
      this.cache.delete(key);
      this.cacheStats.misses++;
      return null;
    }

    this.cacheStats.hits++;
    return cached.data as T;
  }

  /**
   * Set data in cache with TTL
   */
  private setCache<T>(key: string, data: T, ttl: number = this.DEFAULT_TTL): void {
    this.cache.set(key, {
      data,
      expiry: Date.now() + ttl,
      timestamp: Date.now()
    });
  }

  /**
   * Clear cache entries matching a pattern
   */
  private clearCache(pattern?: string): void {
    if (!pattern) {
      this.cache.clear();
      return;
    }

    for (const key of Array.from(this.cache.keys())) {
      if (key.startsWith(pattern)) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Clean expired cache entries
   */
  private cleanExpiredCache(): void {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (entry.expiry < now) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): CacheStats {
    const total = this.cacheStats.hits + this.cacheStats.misses;
    return {
      hits: this.cacheStats.hits,
      misses: this.cacheStats.misses,
      size: this.cache.size,
      hitRate: total > 0 ? (this.cacheStats.hits / total) * 100 : 0
    };
  }

  /**
   * Manually invalidate cache
   */
  invalidateCache(pattern?: string): void {
    this.clearCache(pattern);
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
        'X-User-ID': DEFAULT_USER_ID,  // 🔧 Added: Required for backend auth
        'X-Organization-ID': DEFAULT_ORG_ID,  // 🔧 Added: Required for backend auth
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
   * Get all agents (cached for 30s)
   */
  async getAgents(): Promise<Agent[]> {
    const cacheKey = 'agents:list';
    const cached = this.getCached<Agent[]>(cacheKey);
    if (cached) {
      console.log('✅ Cache hit: agents:list');
      return cached;
    }

    console.log('🔄 Cache miss: agents:list');
    const response = await this.request<ApiResponse<Agent[]>>('/api/v1/agents');
    this.setCache(cacheKey, response.data, 30000); // 30s TTL
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
   * Create new agent (invalidates agent cache)
   */
  async createAgent(data: CreateAgentRequest): Promise<Agent> {
    const response = await this.request<ApiResponse<Agent>>('/api/v1/agents', {
      method: 'POST',
      body: JSON.stringify(data),
    });
    
    // Invalidate agents cache
    this.clearCache('agents:');
    console.log('🗑️  Cache cleared: agents:*');
    
    return response.data;
  }

  /**
   * Update agent (invalidates agent cache)
   */
  async updateAgent(agentId: string, data: Partial<Agent>): Promise<Agent> {
    const response = await this.request<ApiResponse<Agent>>(
      `/api/v1/agents/${agentId}`,
      {
        method: 'PUT',
        body: JSON.stringify(data),
      }
    );
    
    // Invalidate agents cache
    this.clearCache('agents:');
    console.log('🗑️  Cache cleared: agents:*');
    
    return response.data;
  }

  /**
   * Delete agent (invalidates agent cache)
   */
  async deleteAgent(agentId: string): Promise<void> {
    await this.request(`/api/v1/agents/${agentId}`, {
      method: 'DELETE',
    });
    
    // Invalidate agents cache
    this.clearCache('agents:');
    console.log('🗑️  Cache cleared: agents:*');
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
   * Get all memories with pagination (🆕 Fix 1: Global memories list API)
   */
  async getAllMemories(page: number = 0, limit: number = 20, agentId?: string, memoryType?: string): Promise<{ memories: Memory[], pagination: { page: number, limit: number, total: number, total_pages: number } }> {
    const cacheKey = `memories:all:${page}:${limit}:${agentId || 'all'}:${memoryType || 'all'}`;
    const cached = this.getCached<{ memories: Memory[], pagination: any }>(cacheKey);
    if (cached) {
      console.log(`✅ Cache hit: ${cacheKey}`);
      return cached;
    }

    console.log(`🔄 Cache miss: ${cacheKey}`);
    let url = `/api/v1/memories?page=${page}&limit=${limit}`;
    if (agentId) {
      url += `&agent_id=${agentId}`;
    }
    if (memoryType && memoryType !== 'all') {
      url += `&memory_type=${memoryType}`;
    }
    
    const response = await this.request<ApiResponse<{ memories: Memory[], pagination: any }>>(url);
    this.setCache(cacheKey, response.data, 30000); // 30s TTL
    return response.data;
  }

  /**
   * Create new memory (invalidates memory and stats cache)
   */
  async createMemory(data: CreateMemoryRequest): Promise<Memory> {
    const response = await this.request<ApiResponse<Memory>>(
      '/api/v1/memories',
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    );
    
    // Invalidate related caches
    this.clearCache('memories:');
    this.clearCache('stats:');
    console.log('🗑️  Cache cleared: memories:*, stats:*');
    
    return response.data;
  }

  /**
   * Delete memory (invalidates memory and stats cache)
   */
  async deleteMemory(memoryId: string): Promise<void> {
    await this.request(`/api/v1/memories/${memoryId}`, {
      method: 'DELETE',
    });
    
    // Invalidate related caches
    this.clearCache('memories:');
    this.clearCache('stats:');
    console.log('🗑️  Cache cleared: memories:*, stats:*');
  }

  /**
   * Search memories
   */
  async searchMemories(query: string, agentId?: string, userId?: string): Promise<Memory[]> {
    const response = await this.request<ApiResponse<Memory[]>>(
      `/api/v1/memories/search`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query,
          agent_id: agentId,
          user_id: normalizeUserId(userId), // ✅ 修复: 使用 normalizeUserId 统一处理
        }),
      }
    );
    return response.data;
  }

  // ==================== User APIs ====================

  /**
   * Get all users (cached for 30s, with pagination support)
   */
  async getUsers(page: number = 1, pageSize: number = 50): Promise<User[]> {
    const cacheKey = `users:list:${page}:${pageSize}`;
    const cached = this.getCached<User[]>(cacheKey);
    if (cached) {
      console.log(`✅ Cache hit: users:list:${page}:${pageSize}`);
      return cached;
    }

    console.log(`🔄 Cache miss: users:list:${page}:${pageSize}`);
    const response = await this.request<ApiResponse<UsersListResponse>>(
      `/api/v1/users?page=${page}&page_size=${pageSize}`
    );
    this.setCache(cacheKey, response.data.users, 30000); // 30s TTL
    return response.data.users;
  }

  /**
   * Get users list with full pagination info
   */
  async getUsersWithPagination(page: number = 1, pageSize: number = 50): Promise<UsersListResponse> {
    const response = await this.request<ApiResponse<UsersListResponse>>(
      `/api/v1/users?page=${page}&page_size=${pageSize}`
    );
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

  // ==================== Statistics APIs ====================

  /**
   * Get dashboard statistics
   * 
   * Returns comprehensive statistics for the admin dashboard including:
   * - Total counts for agents, users, memories, messages
   * - Active entity counts (last 24h)
   * - Recent activity logs
   * - Memory distribution by type
   */
  async getDashboardStats(): Promise<DashboardStats> {
    const response = await this.request<DashboardStats>('/api/v1/stats/dashboard');
    return response;
  }

  /**
   * Get memory growth statistics
   * 
   * Returns time series data showing memory growth over the last 30 days.
   * Data points include total memories, new memories, and breakdown by type.
   */
  async getMemoryGrowth(): Promise<MemoryGrowthResponse> {
    const response = await this.request<MemoryGrowthResponse>('/api/v1/stats/memories/growth');
    return response;
  }

  /**
   * Get agent activity statistics
   * 
   * Returns activity statistics for all agents including:
   * - Memory counts
   * - Interaction counts (messages)
   * - Last active timestamps
   * - Average memory importance
   */
  async getAgentActivity(): Promise<AgentActivityResponse> {
    const response = await this.request<AgentActivityResponse>('/api/v1/stats/agents/activity');
    return response;
  }

  /**
   * Get memory quality statistics
   * 
   * Returns comprehensive memory quality metrics including:
   * - Average importance
   * - High-quality memory ratio
   * - Importance distribution
   * - Memory type distribution
   */
  async getMemoryQuality(): Promise<MemoryQualityResponse> {
    const response = await this.request<MemoryQualityResponse>('/api/v1/stats/memory/quality');
    return response;
  }

  // ==================== Working Memory APIs ====================

  /**
   * Add a working memory item
   */
  async addWorkingMemory(data: AddWorkingMemoryRequest): Promise<WorkingMemoryItem> {
    const response = await this.request<ApiResponse<{ id: string; session_id: string; content: string; created_at: string }>>(
      '/api/v1/working-memory',
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    );
    
    // Invalidate working memory cache
    this.clearCache('working-memory:');
    console.log('🗑️  Cache cleared: working-memory:*');
    
    // Return full item (API returns partial, we'll construct it)
    return {
      id: response.data.id,
      user_id: DEFAULT_USER_ID,  // ✅ 使用常量
      agent_id: 'default-agent',
      session_id: response.data.session_id,
      content: response.data.content,
      priority: data.priority || 1,
      expires_at: data.expires_in_seconds ? new Date(Date.now() + data.expires_in_seconds * 1000).toISOString() : null,
      metadata: data.metadata || {},
      created_at: response.data.created_at,
    };
  }

  /**
   * Get working memory items for a session
   */
  async getWorkingMemory(sessionId: string, minPriority?: number): Promise<WorkingMemoryItem[]> {
    const cacheKey = `working-memory:session:${sessionId}:${minPriority || 'all'}`;
    const cached = this.getCached<WorkingMemoryItem[]>(cacheKey);
    if (cached) {
      console.log(`✅ Cache hit: ${cacheKey}`);
      return cached;
    }

    console.log(`🔄 Cache miss: ${cacheKey}`);
    let url = `/api/v1/working-memory?session_id=${sessionId}`;
    if (minPriority !== undefined) {
      url += `&min_priority=${minPriority}`;
    }
    
    const response = await this.request<ApiResponse<WorkingMemoryItem[]>>(url);
    this.setCache(cacheKey, response.data, 10000); // 10s TTL (working memory changes frequently)
    return response.data;
  }

  /**
   * Delete a working memory item
   */
  async deleteWorkingMemoryItem(itemId: string): Promise<void> {
    await this.request(`/api/v1/working-memory/${itemId}`, {
      method: 'DELETE',
    });
    
    // Invalidate working memory cache
    this.clearCache('working-memory:');
    console.log('🗑️  Cache cleared: working-memory:*');
  }

  /**
   * Clear all working memory items for a session
   */
  async clearWorkingMemory(sessionId: string): Promise<ClearWorkingMemoryResponse> {
    const response = await this.request<ApiResponse<ClearWorkingMemoryResponse>>(
      `/api/v1/working-memory/sessions/${sessionId}`,
      {
        method: 'DELETE',
      }
    );
    
    // Invalidate working memory cache
    this.clearCache('working-memory:');
    console.log('🗑️  Cache cleared: working-memory:*');
    
    return response.data;
  }

  /**
   * Cleanup expired working memory items
   */
  async cleanupExpiredWorkingMemory(): Promise<CleanupWorkingMemoryResponse> {
    const response = await this.request<ApiResponse<CleanupWorkingMemoryResponse>>(
      '/api/v1/working-memory/cleanup',
      {
        method: 'POST',
      }
    );
    
    // Invalidate working memory cache
    this.clearCache('working-memory:');
    console.log('🗑️  Cache cleared: working-memory:*');
    
    return response.data;
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

