/**
 * AgentMem JavaScript SDK - Type Definitions
 * 
 * Core data types and interfaces for the AgentMem JavaScript SDK.
 */

/**
 * Memory type enumeration
 */
export enum MemoryType {
  EPISODIC = 'episodic',
  SEMANTIC = 'semantic',
  PROCEDURAL = 'procedural',
  UNTYPED = 'untyped',
}

/**
 * Memory importance level
 */
export enum ImportanceLevel {
  LOW = 1,
  MEDIUM = 2,
  HIGH = 3,
  CRITICAL = 4,
}

/**
 * Search match type
 */
export enum MatchType {
  EXACT_TEXT = 'exact_text',
  PARTIAL_TEXT = 'partial_text',
  SEMANTIC = 'semantic',
  METADATA = 'metadata',
}

/**
 * Memory data structure
 */
export interface Memory {
  id: string;
  content: string;
  memory_type: MemoryType;
  agent_id: string;
  user_id?: string;
  session_id?: string;
  importance: number;
  metadata?: Record<string, any>;
  created_at?: string;
  updated_at?: string;
  access_count: number;
  last_accessed?: string;
  embedding?: number[];
}

/**
 * Search query parameters
 */
export interface SearchQuery {
  agent_id: string;
  text_query?: string;
  vector_query?: number[];
  memory_type?: MemoryType;
  user_id?: string;
  min_importance?: number;
  max_age_seconds?: number;
  limit?: number;
  metadata_filters?: Record<string, any>;
}

/**
 * Search result with score and match type
 */
export interface SearchResult {
  memory: Memory;
  score: number;
  match_type: MatchType;
}

/**
 * Memory statistics
 */
export interface MemoryStats {
  total_memories: number;
  memories_by_type: Record<string, number>;
  memories_by_agent: Record<string, number>;
  average_importance: number;
  oldest_memory_age_days: number;
  most_accessed_memory_id?: string;
  total_access_count: number;
}

/**
 * Client configuration options
 */
export interface Config {
  /** API key for authentication */
  apiKey: string;
  
  /** Base URL for the AgentMem API */
  baseUrl?: string;
  
  /** API version */
  apiVersion?: string;
  
  /** Request timeout in milliseconds */
  timeout?: number;
  
  /** Maximum number of retry attempts */
  maxRetries?: number;
  
  /** Delay between retries in milliseconds */
  retryDelay?: number;
  
  /** Enable request/response compression */
  enableCompression?: boolean;
  
  /** Enable response caching */
  enableCaching?: boolean;
  
  /** Cache TTL in milliseconds */
  cacheTtl?: number;
  
  /** Enable debug logging */
  enableLogging?: boolean;
  
  /** Custom headers to include in requests */
  customHeaders?: Record<string, string>;
}

/**
 * Memory creation parameters
 */
export interface CreateMemoryParams {
  content: string;
  agent_id: string;
  memory_type?: MemoryType;
  user_id?: string;
  session_id?: string;
  importance?: number;
  metadata?: Record<string, any>;
}

/**
 * Memory update parameters
 */
export interface UpdateMemoryParams {
  content?: string;
  importance?: number;
  metadata?: Record<string, any>;
}

/**
 * Batch memory creation parameters
 */
export interface BatchCreateMemoryParams {
  memories: CreateMemoryParams[];
}

/**
 * API response wrapper
 */
export interface ApiResponse<T = any> {
  data?: T;
  error?: string;
  message?: string;
  status: number;
}

/**
 * Health check response
 */
export interface HealthStatus {
  status: 'healthy' | 'unhealthy';
  version: string;
  uptime: number;
  timestamp: string;
  services: Record<string, 'up' | 'down'>;
}

/**
 * System metrics
 */
export interface SystemMetrics {
  requests_per_second: number;
  average_response_time: number;
  active_connections: number;
  memory_usage: number;
  cpu_usage: number;
  cache_hit_rate: number;
}

/**
 * Error types
 */
export class AgentMemError extends Error {
  constructor(message: string, public statusCode?: number, public code?: string) {
    super(message);
    this.name = 'AgentMemError';
  }
}

export class AuthenticationError extends AgentMemError {
  constructor(message: string = 'Authentication failed') {
    super(message, 401, 'AUTHENTICATION_ERROR');
    this.name = 'AuthenticationError';
  }
}

export class ValidationError extends AgentMemError {
  constructor(message: string = 'Request validation failed') {
    super(message, 400, 'VALIDATION_ERROR');
    this.name = 'ValidationError';
  }
}

export class NetworkError extends AgentMemError {
  constructor(message: string = 'Network error') {
    super(message, 0, 'NETWORK_ERROR');
    this.name = 'NetworkError';
  }
}

export class NotFoundError extends AgentMemError {
  constructor(message: string = 'Resource not found') {
    super(message, 404, 'NOT_FOUND_ERROR');
    this.name = 'NotFoundError';
  }
}

export class RateLimitError extends AgentMemError {
  constructor(message: string = 'Rate limit exceeded') {
    super(message, 429, 'RATE_LIMIT_ERROR');
    this.name = 'RateLimitError';
  }
}

export class ServerError extends AgentMemError {
  constructor(message: string = 'Server error') {
    super(message, 500, 'SERVER_ERROR');
    this.name = 'ServerError';
  }
}

/**
 * Request options for API calls
 */
export interface RequestOptions {
  timeout?: number;
  retries?: number;
  useCache?: boolean;
  headers?: Record<string, string>;
}
