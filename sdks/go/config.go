package agentmem

import (
	"fmt"
	"net/url"
	"os"
	"strconv"
	"time"
)

// DefaultConfig returns the default configuration
func DefaultConfig() *Config {
	return &Config{
		BaseURL:           "https://api.agentmem.dev",
		APIVersion:        "v1",
		Timeout:           30 * time.Second,
		MaxRetries:        3,
		RetryDelay:        1 * time.Second,
		EnableCompression: true,
		EnableCaching:     true,
		CacheTTL:          5 * time.Minute,
		EnableLogging:     false,
		CustomHeaders:     make(map[string]string),
	}
}

// NewConfigFromEnv creates configuration from environment variables
func NewConfigFromEnv() (*Config, error) {
	config := DefaultConfig()
	
	// API Key (required)
	apiKey := os.Getenv("AGENTMEM_API_KEY")
	if apiKey == "" {
		return nil, fmt.Errorf("AGENTMEM_API_KEY environment variable is required")
	}
	config.APIKey = apiKey
	
	// Base URL
	if baseURL := os.Getenv("AGENTMEM_BASE_URL"); baseURL != "" {
		config.BaseURL = baseURL
	}
	
	// API Version
	if apiVersion := os.Getenv("AGENTMEM_API_VERSION"); apiVersion != "" {
		config.APIVersion = apiVersion
	}
	
	// Timeout
	if timeoutStr := os.Getenv("AGENTMEM_TIMEOUT"); timeoutStr != "" {
		if timeout, err := strconv.Atoi(timeoutStr); err == nil {
			config.Timeout = time.Duration(timeout) * time.Second
		}
	}
	
	// Max Retries
	if retriesStr := os.Getenv("AGENTMEM_MAX_RETRIES"); retriesStr != "" {
		if retries, err := strconv.Atoi(retriesStr); err == nil {
			config.MaxRetries = retries
		}
	}
	
	// Retry Delay
	if delayStr := os.Getenv("AGENTMEM_RETRY_DELAY"); delayStr != "" {
		if delay, err := strconv.Atoi(delayStr); err == nil {
			config.RetryDelay = time.Duration(delay) * time.Second
		}
	}
	
	// Enable Compression
	if compressionStr := os.Getenv("AGENTMEM_ENABLE_COMPRESSION"); compressionStr != "" {
		config.EnableCompression = compressionStr == "true"
	}
	
	// Enable Caching
	if cachingStr := os.Getenv("AGENTMEM_ENABLE_CACHING"); cachingStr != "" {
		config.EnableCaching = cachingStr == "true"
	}
	
	// Cache TTL
	if ttlStr := os.Getenv("AGENTMEM_CACHE_TTL"); ttlStr != "" {
		if ttl, err := strconv.Atoi(ttlStr); err == nil {
			config.CacheTTL = time.Duration(ttl) * time.Second
		}
	}
	
	// Enable Logging
	if loggingStr := os.Getenv("AGENTMEM_ENABLE_LOGGING"); loggingStr != "" {
		config.EnableLogging = loggingStr == "true"
	}
	
	return config, config.Validate()
}

// NewConfig creates a new configuration with the provided API key
func NewConfig(apiKey string) *Config {
	config := DefaultConfig()
	config.APIKey = apiKey
	return config
}

// Validate validates the configuration
func (c *Config) Validate() error {
	if c.APIKey == "" {
		return fmt.Errorf("API key is required")
	}
	
	if c.BaseURL == "" {
		return fmt.Errorf("base URL is required")
	}
	
	if c.Timeout <= 0 {
		return fmt.Errorf("timeout must be positive")
	}
	
	if c.MaxRetries < 0 {
		return fmt.Errorf("max retries must be non-negative")
	}
	
	if c.RetryDelay < 0 {
		return fmt.Errorf("retry delay must be non-negative")
	}
	
	if c.CacheTTL <= 0 {
		return fmt.Errorf("cache TTL must be positive")
	}
	
	// Validate URL format
	if _, err := url.Parse(c.BaseURL); err != nil {
		return fmt.Errorf("invalid base URL format: %w", err)
	}
	
	return nil
}

// GetAPIBaseURL returns the full API base URL
func (c *Config) GetAPIBaseURL() string {
	return fmt.Sprintf("%s/api/%s", c.BaseURL, c.APIVersion)
}

// GetDefaultHeaders returns default headers for requests
func (c *Config) GetDefaultHeaders() map[string]string {
	headers := map[string]string{
		"Authorization": fmt.Sprintf("Bearer %s", c.APIKey),
		"Content-Type":  "application/json",
		"User-Agent":    "agentmem-go/6.0.0",
	}
	
	if c.EnableCompression {
		headers["Accept-Encoding"] = "gzip, deflate"
	}
	
	// Add custom headers
	for key, value := range c.CustomHeaders {
		headers[key] = value
	}
	
	return headers
}

// Clone creates a copy of the configuration
func (c *Config) Clone() *Config {
	clone := *c
	
	// Deep copy custom headers
	clone.CustomHeaders = make(map[string]string)
	for key, value := range c.CustomHeaders {
		clone.CustomHeaders[key] = value
	}
	
	return &clone
}

// WithAPIKey returns a new config with the specified API key
func (c *Config) WithAPIKey(apiKey string) *Config {
	clone := c.Clone()
	clone.APIKey = apiKey
	return clone
}

// WithBaseURL returns a new config with the specified base URL
func (c *Config) WithBaseURL(baseURL string) *Config {
	clone := c.Clone()
	clone.BaseURL = baseURL
	return clone
}

// WithTimeout returns a new config with the specified timeout
func (c *Config) WithTimeout(timeout time.Duration) *Config {
	clone := c.Clone()
	clone.Timeout = timeout
	return clone
}

// WithRetries returns a new config with the specified retry settings
func (c *Config) WithRetries(maxRetries int, retryDelay time.Duration) *Config {
	clone := c.Clone()
	clone.MaxRetries = maxRetries
	clone.RetryDelay = retryDelay
	return clone
}

// WithCaching returns a new config with the specified caching settings
func (c *Config) WithCaching(enabled bool, ttl time.Duration) *Config {
	clone := c.Clone()
	clone.EnableCaching = enabled
	clone.CacheTTL = ttl
	return clone
}

// WithLogging returns a new config with logging enabled/disabled
func (c *Config) WithLogging(enabled bool) *Config {
	clone := c.Clone()
	clone.EnableLogging = enabled
	return clone
}

// WithCustomHeaders returns a new config with additional custom headers
func (c *Config) WithCustomHeaders(headers map[string]string) *Config {
	clone := c.Clone()
	for key, value := range headers {
		clone.CustomHeaders[key] = value
	}
	return clone
}
