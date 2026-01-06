/**
 * AgentMem JavaScript SDK - Configuration
 * 
 * Configuration management for the AgentMem JavaScript SDK.
 */

import { Config } from './types';

/**
 * Default configuration values
 */
export const DEFAULT_CONFIG: Partial<Config> = {
  baseUrl: 'https://api.agentmem.dev',
  apiVersion: 'v1',
  timeout: 30000, // 30 seconds
  maxRetries: 3,
  retryDelay: 1000, // 1 second
  enableCompression: true,
  enableCaching: true,
  cacheTtl: 300000, // 5 minutes
  enableLogging: false,
  customHeaders: {},
};

/**
 * Create configuration from environment variables and overrides
 */
export function createConfig(overrides: Partial<Config> = {}): Config {
  // Try to get API key from environment or overrides
  const apiKey = overrides.apiKey || 
                 (typeof process !== 'undefined' ? process.env?.AGENTMEM_API_KEY : undefined) ||
                 (typeof window !== 'undefined' ? (window as any).AGENTMEM_API_KEY : undefined);

  if (!apiKey) {
    throw new Error('API key is required. Set AGENTMEM_API_KEY environment variable or pass apiKey in config.');
  }

  // Merge default config with environment variables and overrides
  const config: Config = {
    apiKey,
    baseUrl: overrides.baseUrl || 
             (typeof process !== 'undefined' ? process.env?.AGENTMEM_BASE_URL : undefined) ||
             DEFAULT_CONFIG.baseUrl!,
    apiVersion: overrides.apiVersion || 
                (typeof process !== 'undefined' ? process.env?.AGENTMEM_API_VERSION : undefined) ||
                DEFAULT_CONFIG.apiVersion!,
    timeout: overrides.timeout || 
             (typeof process !== 'undefined' && process.env?.AGENTMEM_TIMEOUT ? 
              parseInt(process.env.AGENTMEM_TIMEOUT, 10) : undefined) ||
             DEFAULT_CONFIG.timeout!,
    maxRetries: overrides.maxRetries || 
                (typeof process !== 'undefined' && process.env?.AGENTMEM_MAX_RETRIES ? 
                 parseInt(process.env.AGENTMEM_MAX_RETRIES, 10) : undefined) ||
                DEFAULT_CONFIG.maxRetries!,
    retryDelay: overrides.retryDelay || 
                (typeof process !== 'undefined' && process.env?.AGENTMEM_RETRY_DELAY ? 
                 parseInt(process.env.AGENTMEM_RETRY_DELAY, 10) : undefined) ||
                DEFAULT_CONFIG.retryDelay!,
    enableCompression: overrides.enableCompression !== undefined ? 
                       overrides.enableCompression :
                       (typeof process !== 'undefined' && process.env?.AGENTMEM_ENABLE_COMPRESSION ? 
                        process.env.AGENTMEM_ENABLE_COMPRESSION.toLowerCase() === 'true' : undefined) ||
                       DEFAULT_CONFIG.enableCompression!,
    enableCaching: overrides.enableCaching !== undefined ? 
                   overrides.enableCaching :
                   (typeof process !== 'undefined' && process.env?.AGENTMEM_ENABLE_CACHING ? 
                    process.env.AGENTMEM_ENABLE_CACHING.toLowerCase() === 'true' : undefined) ||
                   DEFAULT_CONFIG.enableCaching!,
    cacheTtl: overrides.cacheTtl || 
              (typeof process !== 'undefined' && process.env?.AGENTMEM_CACHE_TTL ? 
               parseInt(process.env.AGENTMEM_CACHE_TTL, 10) : undefined) ||
              DEFAULT_CONFIG.cacheTtl!,
    enableLogging: overrides.enableLogging !== undefined ? 
                   overrides.enableLogging :
                   (typeof process !== 'undefined' && process.env?.AGENTMEM_ENABLE_LOGGING ? 
                    process.env.AGENTMEM_ENABLE_LOGGING.toLowerCase() === 'true' : undefined) ||
                   DEFAULT_CONFIG.enableLogging!,
    customHeaders: { ...DEFAULT_CONFIG.customHeaders, ...overrides.customHeaders },
  };

  validateConfig(config);
  return config;
}

/**
 * Validate configuration
 */
export function validateConfig(config: Config): void {
  if (!config.apiKey) {
    throw new Error('API key is required');
  }

  if (!config.baseUrl) {
    throw new Error('Base URL is required');
  }

  if (config.timeout <= 0) {
    throw new Error('Timeout must be positive');
  }

  if (config.maxRetries < 0) {
    throw new Error('Max retries must be non-negative');
  }

  if (config.retryDelay < 0) {
    throw new Error('Retry delay must be non-negative');
  }

  if (config.cacheTtl <= 0) {
    throw new Error('Cache TTL must be positive');
  }

  // Validate URL format
  try {
    new URL(config.baseUrl);
  } catch (error) {
    throw new Error('Invalid base URL format');
  }
}

/**
 * Get the full API base URL
 */
export function getApiBaseUrl(config: Config): string {
  return `${config.baseUrl}/api/${config.apiVersion}`;
}

/**
 * Get default headers for requests
 */
export function getDefaultHeaders(config: Config): Record<string, string> {
  const headers: Record<string, string> = {
    'Authorization': `Bearer ${config.apiKey}`,
    'Content-Type': 'application/json',
    'User-Agent': '@agentmem/client/6.0.0',
    ...config.customHeaders,
  };

  if (config.enableCompression) {
    headers['Accept-Encoding'] = 'gzip, deflate';
  }

  return headers;
}
