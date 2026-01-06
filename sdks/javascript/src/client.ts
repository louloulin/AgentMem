/**
 * AgentMem JavaScript SDK - Main Client
 * 
 * Official JavaScript/TypeScript client for AgentMem API.
 */

import axios, { AxiosInstance, AxiosResponse, AxiosError } from 'axios';
import axiosRetry from 'axios-retry';
import {
  Config,
  Memory,
  MemoryType,
  SearchQuery,
  SearchResult,
  MemoryStats,
  CreateMemoryParams,
  UpdateMemoryParams,
  BatchCreateMemoryParams,
  HealthStatus,
  SystemMetrics,
  RequestOptions,
  AgentMemError,
  AuthenticationError,
  ValidationError,
  NetworkError,
  NotFoundError,
  RateLimitError,
  ServerError,
} from './types';
import { createConfig, getApiBaseUrl, getDefaultHeaders } from './config';

/**
 * Cache entry interface
 */
interface CacheEntry {
  data: any;
  timestamp: number;
}

/**
 * AgentMem JavaScript client for interacting with AgentMem API
 * 
 * @example
 * ```typescript
 * import { AgentMemClient, MemoryType } from '@agentmem/client';
 * 
 * const client = new AgentMemClient({
 *   apiKey: 'your-api-key',
 *   baseUrl: 'https://api.agentmem.dev'
 * });
 * 
 * // Add a memory
 * const memoryId = await client.addMemory({
 *   content: 'Important project information',
 *   agent_id: 'agent_1',
 *   memory_type: MemoryType.SEMANTIC,
 *   importance: 0.8
 * });
 * 
 * // Search memories
 * const results = await client.searchMemories({
 *   agent_id: 'agent_1',
 *   text_query: 'project information',
 *   limit: 5
 * });
 * ```
 */
export class AgentMemClient {
  private config: Config;
  private httpClient: AxiosInstance;
  private cache: Map<string, CacheEntry> = new Map();

  /**
   * Initialize AgentMem client
   */
  constructor(config: Partial<Config>) {
    this.config = createConfig(config);
    this.httpClient = this.createHttpClient();
    this.setupRetryLogic();
  }

  /**
   * Create HTTP client instance
   */
  private createHttpClient(): AxiosInstance {
    const client = axios.create({
      baseURL: getApiBaseUrl(this.config),
      timeout: this.config.timeout,
      headers: getDefaultHeaders(this.config),
    });

    // Request interceptor for logging
    if (this.config.enableLogging) {
      client.interceptors.request.use(
        (config) => {
          console.log(`[AgentMem] ${config.method?.toUpperCase()} ${config.url}`);
          return config;
        },
        (error) => {
          console.error('[AgentMem] Request error:', error);
          return Promise.reject(error);
        }
      );
    }

    // Response interceptor for error handling
    client.interceptors.response.use(
      (response) => {
        if (this.config.enableLogging) {
          console.log(`[AgentMem] Response ${response.status} ${response.config.url}`);
        }
        return response;
      },
      (error) => {
        return Promise.reject(this.handleError(error));
      }
    );

    return client;
  }

  /**
   * Setup retry logic
   */
  private setupRetryLogic(): void {
    axiosRetry(this.httpClient, {
      retries: this.config.maxRetries,
      retryDelay: (retryCount) => {
        return this.config.retryDelay * Math.pow(2, retryCount - 1); // Exponential backoff
      },
      retryCondition: (error) => {
        // Retry on network errors and 5xx status codes
        return axiosRetry.isNetworkOrIdempotentRequestError(error) ||
               (error.response?.status >= 500 && error.response?.status < 600);
      },
    });
  }

  /**
   * Handle HTTP errors and convert to AgentMem errors
   */
  private handleError(error: AxiosError): AgentMemError {
    if (error.response) {
      const status = error.response.status;
      const message = (error.response.data as any)?.message || error.message;

      switch (status) {
        case 401:
          return new AuthenticationError(message);
        case 400:
          return new ValidationError(message);
        case 404:
          return new NotFoundError(message);
        case 429:
          return new RateLimitError(message);
        case 500:
        case 502:
        case 503:
        case 504:
          return new ServerError(message);
        default:
          return new AgentMemError(message, status);
      }
    } else if (error.request) {
      return new NetworkError('Network error: ' + error.message);
    } else {
      return new AgentMemError('Request error: ' + error.message);
    }
  }

  /**
   * Generate cache key
   */
  private getCacheKey(method: string, url: string, params?: any): string {
    const keyParts = [method, url];
    if (params) {
      keyParts.push(JSON.stringify(params));
    }
    return keyParts.join('|');
  }

  /**
   * Check if cache entry is valid
   */
  private isCacheValid(entry: CacheEntry): boolean {
    const age = Date.now() - entry.timestamp;
    return age < this.config.cacheTtl;
  }

  /**
   * Get from cache
   */
  private getFromCache(key: string): any | null {
    if (!this.config.enableCaching) {
      return null;
    }

    const entry = this.cache.get(key);
    if (entry && this.isCacheValid(entry)) {
      return entry.data;
    }

    // Clean up expired entry
    if (entry) {
      this.cache.delete(key);
    }

    return null;
  }

  /**
   * Set cache entry
   */
  private setCache(key: string, data: any): void {
    if (this.config.enableCaching) {
      this.cache.set(key, {
        data,
        timestamp: Date.now(),
      });
    }
  }

  /**
   * Make HTTP request with caching support
   */
  private async makeRequest<T>(
    method: string,
    endpoint: string,
    data?: any,
    options: RequestOptions = {}
  ): Promise<T> {
    const url = endpoint;
    
    // Check cache for GET requests
    if (method === 'GET' && options.useCache !== false) {
      const cacheKey = this.getCacheKey(method, url, data);
      const cachedResult = this.getFromCache(cacheKey);
      if (cachedResult !== null) {
        return cachedResult;
      }
    }

    // Prepare request config
    const requestConfig: any = {
      method,
      url,
      timeout: options.timeout || this.config.timeout,
      headers: { ...getDefaultHeaders(this.config), ...options.headers },
    };

    if (data) {
      if (method === 'GET') {
        requestConfig.params = data;
      } else {
        requestConfig.data = data;
      }
    }

    // Make request
    const response: AxiosResponse<T> = await this.httpClient.request(requestConfig);
    
    // Cache successful GET requests
    if (method === 'GET' && options.useCache !== false) {
      const cacheKey = this.getCacheKey(method, url, data);
      this.setCache(cacheKey, response.data);
    }

    return response.data;
  }

  /**
   * Add a new memory
   */
  async addMemory(params: CreateMemoryParams): Promise<string> {
    const response = await this.makeRequest<{ id: string }>('POST', '/memories', params);
    return response.id;
  }

  /**
   * Get a memory by ID
   */
  async getMemory(memoryId: string): Promise<Memory> {
    return this.makeRequest<Memory>('GET', `/memories/${memoryId}`, undefined, { useCache: true });
  }

  /**
   * Update an existing memory
   */
  async updateMemory(memoryId: string, params: UpdateMemoryParams): Promise<Memory> {
    return this.makeRequest<Memory>('PUT', `/memories/${memoryId}`, params);
  }

  /**
   * Delete a memory
   */
  async deleteMemory(memoryId: string): Promise<void> {
    await this.makeRequest<void>('DELETE', `/memories/${memoryId}`);
  }

  /**
   * Search memories
   */
  async searchMemories(query: SearchQuery): Promise<SearchResult[]> {
    const response = await this.makeRequest<{ results: SearchResult[] }>('POST', '/memories/search', query);
    return response.results;
  }

  /**
   * Add multiple memories in batch
   */
  async batchAddMemories(params: BatchCreateMemoryParams): Promise<string[]> {
    const response = await this.makeRequest<{ ids: string[] }>('POST', '/memories/batch', params);
    return response.ids;
  }

  /**
   * Get memory statistics for an agent
   */
  async getMemoryStats(agentId: string): Promise<MemoryStats> {
    return this.makeRequest<MemoryStats>('GET', '/memories/stats', { agent_id: agentId }, { useCache: true });
  }

  /**
   * Check API health status
   */
  async healthCheck(): Promise<HealthStatus> {
    return this.makeRequest<HealthStatus>('GET', '/health', undefined, { useCache: true });
  }

  /**
   * Get system metrics
   */
  async getMetrics(): Promise<SystemMetrics> {
    return this.makeRequest<SystemMetrics>('GET', '/metrics', undefined, { useCache: true });
  }

  /**
   * Clear cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Get current configuration (with masked API key)
   */
  getConfig(): Partial<Config> {
    return {
      ...this.config,
      apiKey: '***', // Mask API key for security
    };
  }
}
