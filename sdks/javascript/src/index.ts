/**
 * AgentMem JavaScript SDK
 * 
 * Official JavaScript/TypeScript client library for AgentMem - 
 * Enterprise-grade memory management for AI agents.
 * 
 * @example
 * ```typescript
 * import { AgentMemClient, MemoryType } from '@agentmem/client';
 * 
 * const client = new AgentMemClient({
 *   apiKey: 'your-api-key'
 * });
 * 
 * // Add a memory
 * const memoryId = await client.addMemory({
 *   content: 'User prefers dark mode',
 *   agent_id: 'assistant_1',
 *   memory_type: MemoryType.SEMANTIC,
 *   importance: 0.8
 * });
 * 
 * // Search memories
 * const results = await client.searchMemories({
 *   agent_id: 'assistant_1',
 *   text_query: 'user preferences',
 *   limit: 5
 * });
 * ```
 */

// Export main client
export { AgentMemClient } from './client';

// Export configuration utilities
export { createConfig, validateConfig, getApiBaseUrl, getDefaultHeaders, DEFAULT_CONFIG } from './config';

// Export all types and interfaces
export {
  // Core types
  Memory,
  MemoryType,
  ImportanceLevel,
  MatchType,
  SearchQuery,
  SearchResult,
  MemoryStats,
  Config,
  
  // Request/Response types
  CreateMemoryParams,
  UpdateMemoryParams,
  BatchCreateMemoryParams,
  RequestOptions,
  ApiResponse,
  HealthStatus,
  SystemMetrics,
  
  // Error types
  AgentMemError,
  AuthenticationError,
  ValidationError,
  NetworkError,
  NotFoundError,
  RateLimitError,
  ServerError,
} from './types';

// Package metadata
export const VERSION = '6.0.0';
export const AUTHOR = 'AgentMem Team';
export const HOMEPAGE = 'https://agentmem.dev';

/**
 * Create a new AgentMem client with environment-based configuration
 * 
 * @example
 * ```typescript
 * import { createClient } from '@agentmem/client';
 * 
 * // Uses AGENTMEM_API_KEY from environment
 * const client = createClient();
 * 
 * // Or with overrides
 * const client = createClient({
 *   apiKey: 'custom-key',
 *   timeout: 60000
 * });
 * ```
 */
export function createClient(config: Partial<Config> = {}): AgentMemClient {
  return new AgentMemClient(config);
}

/**
 * Utility function to validate memory content
 */
export function validateMemoryContent(content: string): boolean {
  return typeof content === 'string' && content.trim().length > 0;
}

/**
 * Utility function to validate agent ID
 */
export function validateAgentId(agentId: string): boolean {
  return typeof agentId === 'string' && agentId.trim().length > 0;
}

/**
 * Utility function to validate importance score
 */
export function validateImportance(importance: number): boolean {
  return typeof importance === 'number' && importance >= 0 && importance <= 1;
}

/**
 * Utility function to create a basic search query
 */
export function createSearchQuery(
  agentId: string,
  textQuery: string,
  options: Partial<SearchQuery> = {}
): SearchQuery {
  return {
    agent_id: agentId,
    text_query: textQuery,
    limit: 10,
    ...options,
  };
}

/**
 * Utility function to create memory creation parameters
 */
export function createMemoryParams(
  content: string,
  agentId: string,
  options: Partial<CreateMemoryParams> = {}
): CreateMemoryParams {
  return {
    content,
    agent_id: agentId,
    memory_type: MemoryType.UNTYPED,
    importance: 0.5,
    ...options,
  };
}
