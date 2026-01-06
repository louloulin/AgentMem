/**
 * Application Constants
 * 
 * This file contains all application-wide constants to ensure consistency
 * between frontend and backend.
 */

/**
 * Default user and organization IDs
 * 
 * IMPORTANT: These values MUST match the backend defaults in:
 * - crates/agent-mem-server/src/middleware/auth.rs (default_auth_middleware)
 * 
 * Used in development/testing environments when authentication is disabled.
 * In production, these should be replaced with actual authenticated user IDs.
 */
export const DEFAULT_USER_ID = 'default';
export const DEFAULT_ORG_ID = 'default-org';
export const DEFAULT_ROLES = ['admin', 'user'];

/**
 * Default authenticated user for development
 * 
 * This matches the AuthUser struct in the backend:
 * ```rust
 * pub struct AuthUser {
 *     pub user_id: String,
 *     pub org_id: String,
 *     pub roles: Vec<String>,
 * }
 * ```
 */
export interface AuthUser {
  user_id: string;
  org_id: string;
  roles: string[];
}

export const DEFAULT_AUTH_USER: AuthUser = {
  user_id: DEFAULT_USER_ID,
  org_id: DEFAULT_ORG_ID,
  roles: DEFAULT_ROLES,
};

/**
 * API Configuration
 */
export const API_CONFIG = {
  BASE_URL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080',
  TIMEOUT: 30000, // 30 seconds
  RETRY_ATTEMPTS: 3,
  RETRY_DELAY: 1000, // 1 second
} as const;

/**
 * Cache Configuration
 */
export const CACHE_CONFIG = {
  DEFAULT_TTL: 30000, // 30 seconds
  AGENTS_TTL: 60000, // 1 minute
  MEMORIES_TTL: 30000, // 30 seconds
  STATS_TTL: 10000, // 10 seconds
} as const;

/**
 * Memory Types
 * 
 * Must match backend MemoryType enum in:
 * - crates/agent-mem-traits/src/memory.rs
 */
export const MEMORY_TYPES = {
  WORKING: 'working',
  SEMANTIC: 'Semantic',
  EPISODIC: 'Episodic',
  PROCEDURAL: 'Procedural',
  CORE: 'Core',
  RESOURCE: 'Resource',
  KNOWLEDGE: 'Knowledge',
  CONTEXTUAL: 'Contextual',
  FACTUAL: 'Factual',
} as const;

export type MemoryType = typeof MEMORY_TYPES[keyof typeof MEMORY_TYPES];

/**
 * Pagination Defaults
 */
export const PAGINATION = {
  DEFAULT_PAGE: 0,
  DEFAULT_LIMIT: 20,
  MAX_LIMIT: 100,
} as const;

/**
 * Validation Rules
 */
export const VALIDATION = {
  USER_ID_MAX_LENGTH: 255,
  ORG_ID_MAX_LENGTH: 255,
  MESSAGE_MAX_LENGTH: 10000,
  AGENT_NAME_MAX_LENGTH: 255,
  MEMORY_CONTENT_MAX_LENGTH: 50000,
} as const;

/**
 * Utility Functions
 */

/**
 * Validate user_id format
 */
export function validateUserId(userId: string): boolean {
  return userId.length > 0 && userId.length <= VALIDATION.USER_ID_MAX_LENGTH;
}

/**
 * Normalize user_id (for backward compatibility)
 * 
 * Converts legacy 'default-user' to 'default'
 */
export function normalizeUserId(userId?: string): string {
  if (!userId || userId === 'default-user') {
    return DEFAULT_USER_ID;
  }
  return userId;
}

/**
 * Validate organization_id format
 */
export function validateOrgId(orgId: string): boolean {
  return orgId.length > 0 && orgId.length <= VALIDATION.ORG_ID_MAX_LENGTH;
}

/**
 * Normalize organization_id (for backward compatibility)
 */
export function normalizeOrgId(orgId?: string): string {
  if (!orgId || orgId === 'default') {
    return DEFAULT_ORG_ID;
  }
  return orgId;
}

/**
 * Check if a memory type is valid
 */
export function isValidMemoryType(type: string): type is MemoryType {
  return Object.values(MEMORY_TYPES).includes(type as MemoryType);
}

/**
 * Get display name for memory type
 */
export function getMemoryTypeDisplayName(type: MemoryType): string {
  const displayNames: Record<MemoryType, string> = {
    [MEMORY_TYPES.WORKING]: 'Working Memory',
    [MEMORY_TYPES.SEMANTIC]: 'Semantic Memory',
    [MEMORY_TYPES.EPISODIC]: 'Episodic Memory',
    [MEMORY_TYPES.PROCEDURAL]: 'Procedural Memory',
    [MEMORY_TYPES.CORE]: 'Core Memory',
    [MEMORY_TYPES.RESOURCE]: 'Resource Memory',
    [MEMORY_TYPES.KNOWLEDGE]: 'Knowledge Memory',
    [MEMORY_TYPES.CONTEXTUAL]: 'Contextual Memory',
    [MEMORY_TYPES.FACTUAL]: 'Factual Memory',
  };
  return displayNames[type] || type;
}

