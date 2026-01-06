package agentmem

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/go-resty/resty/v2"
)

// cacheEntry represents a cached response
type cacheEntry struct {
	data      interface{}
	timestamp time.Time
}

// Client represents the AgentMem API client
type Client struct {
	config     *Config
	httpClient *resty.Client
	cache      map[string]*cacheEntry
	cacheMutex sync.RWMutex
}

// NewClient creates a new AgentMem client with the provided configuration
func NewClient(config *Config) (*Client, error) {
	if err := config.Validate(); err != nil {
		return nil, fmt.Errorf("invalid configuration: %w", err)
	}

	client := &Client{
		config: config,
		cache:  make(map[string]*cacheEntry),
	}

	client.setupHTTPClient()
	return client, nil
}

// NewClientFromEnv creates a new client using environment variables
func NewClientFromEnv() (*Client, error) {
	config, err := NewConfigFromEnv()
	if err != nil {
		return nil, err
	}
	return NewClient(config)
}

// setupHTTPClient configures the HTTP client
func (c *Client) setupHTTPClient() {
	c.httpClient = resty.New()
	c.httpClient.SetBaseURL(c.config.GetAPIBaseURL())
	c.httpClient.SetTimeout(c.config.Timeout)
	c.httpClient.SetHeaders(c.config.GetDefaultHeaders())

	// Enable compression if configured
	if c.config.EnableCompression {
		c.httpClient.SetHeader("Accept-Encoding", "gzip, deflate")
	}

	// Setup retry logic
	c.httpClient.SetRetryCount(c.config.MaxRetries)
	c.httpClient.SetRetryWaitTime(c.config.RetryDelay)
	c.httpClient.SetRetryMaxWaitTime(c.config.RetryDelay * 10)

	// Retry on server errors and network errors
	c.httpClient.AddRetryCondition(func(r *resty.Response, err error) bool {
		return err != nil || r.StatusCode() >= 500
	})

	// Setup logging if enabled
	if c.config.EnableLogging {
		c.httpClient.SetLogger(log.Default())
		c.httpClient.EnableTrace()
	}

	// Response middleware for error handling
	c.httpClient.OnAfterResponse(func(client *resty.Client, resp *resty.Response) error {
		if resp.IsError() {
			var errorMsg string
			if resp.Body() != nil {
				var apiResp APIResponse
				if err := json.Unmarshal(resp.Body(), &apiResp); err == nil && apiResp.Error != nil {
					errorMsg = *apiResp.Error
				} else if apiResp.Message != nil {
					errorMsg = *apiResp.Message
				}
			}
			if errorMsg == "" {
				errorMsg = fmt.Sprintf("HTTP %d: %s", resp.StatusCode(), resp.Status())
			}
			return handleHTTPError(resp.StatusCode(), errorMsg)
		}
		return nil
	})
}

// getCacheKey generates a cache key for the request
func (c *Client) getCacheKey(method, endpoint string, params interface{}) string {
	key := fmt.Sprintf("%s:%s", method, endpoint)
	if params != nil {
		if paramBytes, err := json.Marshal(params); err == nil {
			key += ":" + string(paramBytes)
		}
	}
	return key
}

// getFromCache retrieves data from cache if valid
func (c *Client) getFromCache(key string) (interface{}, bool) {
	if !c.config.EnableCaching {
		return nil, false
	}

	c.cacheMutex.RLock()
	defer c.cacheMutex.RUnlock()

	entry, exists := c.cache[key]
	if !exists {
		return nil, false
	}

	// Check if cache entry is still valid
	if time.Since(entry.timestamp) > c.config.CacheTTL {
		// Cache expired, remove it
		delete(c.cache, key)
		return nil, false
	}

	return entry.data, true
}

// setCache stores data in cache
func (c *Client) setCache(key string, data interface{}) {
	if !c.config.EnableCaching {
		return
	}

	c.cacheMutex.Lock()
	defer c.cacheMutex.Unlock()

	c.cache[key] = &cacheEntry{
		data:      data,
		timestamp: time.Now(),
	}
}

// makeRequest performs an HTTP request with caching support
func (c *Client) makeRequest(ctx context.Context, method, endpoint string, body interface{}, result interface{}, useCache bool) error {
	// Check cache for GET requests
	if method == "GET" && useCache {
		cacheKey := c.getCacheKey(method, endpoint, body)
		if cachedData, found := c.getFromCache(cacheKey); found {
			if c.config.EnableLogging {
				log.Printf("[AgentMem] Cache hit for %s %s", method, endpoint)
			}
			// Copy cached data to result
			if resultBytes, err := json.Marshal(cachedData); err == nil {
				return json.Unmarshal(resultBytes, result)
			}
		}
	}

	// Prepare request
	req := c.httpClient.R().SetContext(ctx)

	if body != nil {
		if method == "GET" {
			// For GET requests, body contains query parameters
			if params, ok := body.(map[string]interface{}); ok {
				for key, value := range params {
					req.SetQueryParam(key, fmt.Sprintf("%v", value))
				}
			}
		} else {
			req.SetBody(body)
		}
	}

	if result != nil {
		req.SetResult(result)
	}

	// Make request
	var resp *resty.Response
	var err error

	switch method {
	case "GET":
		resp, err = req.Get(endpoint)
	case "POST":
		resp, err = req.Post(endpoint)
	case "PUT":
		resp, err = req.Put(endpoint)
	case "DELETE":
		resp, err = req.Delete(endpoint)
	default:
		return fmt.Errorf("unsupported HTTP method: %s", method)
	}

	if err != nil {
		return NewNetworkError(fmt.Sprintf("Request failed: %v", err))
	}

	// Cache successful GET responses
	if method == "GET" && useCache && resp.IsSuccess() && result != nil {
		cacheKey := c.getCacheKey(method, endpoint, body)
		c.setCache(cacheKey, result)
	}

	return nil
}

// AddMemory adds a new memory
func (c *Client) AddMemory(ctx context.Context, params CreateMemoryParams) (string, error) {
	var response CreateMemoryResponse
	err := c.makeRequest(ctx, "POST", "/memories", params, &response, false)
	if err != nil {
		return "", err
	}
	return response.ID, nil
}

// GetMemory retrieves a memory by ID
func (c *Client) GetMemory(ctx context.Context, memoryID string) (*Memory, error) {
	var memory Memory
	err := c.makeRequest(ctx, "GET", fmt.Sprintf("/memories/%s", memoryID), nil, &memory, true)
	if err != nil {
		return nil, err
	}
	return &memory, nil
}

// UpdateMemory updates an existing memory
func (c *Client) UpdateMemory(ctx context.Context, memoryID string, params UpdateMemoryParams) (*Memory, error) {
	var memory Memory
	err := c.makeRequest(ctx, "PUT", fmt.Sprintf("/memories/%s", memoryID), params, &memory, false)
	if err != nil {
		return nil, err
	}
	return &memory, nil
}

// DeleteMemory deletes a memory
func (c *Client) DeleteMemory(ctx context.Context, memoryID string) error {
	return c.makeRequest(ctx, "DELETE", fmt.Sprintf("/memories/%s", memoryID), nil, nil, false)
}

// SearchMemories searches for memories
func (c *Client) SearchMemories(ctx context.Context, query SearchQuery) ([]SearchResult, error) {
	var response SearchResponse
	err := c.makeRequest(ctx, "POST", "/memories/search", query, &response, false)
	if err != nil {
		return nil, err
	}
	return response.Results, nil
}

// BatchAddMemories adds multiple memories in batch
func (c *Client) BatchAddMemories(ctx context.Context, params BatchCreateMemoryParams) ([]string, error) {
	var response BatchCreateResponse
	err := c.makeRequest(ctx, "POST", "/memories/batch", params, &response, false)
	if err != nil {
		return nil, err
	}
	return response.IDs, nil
}

// GetMemoryStats retrieves memory statistics for an agent
func (c *Client) GetMemoryStats(ctx context.Context, agentID string) (*MemoryStats, error) {
	var stats MemoryStats
	queryParams := map[string]interface{}{
		"agent_id": agentID,
	}
	err := c.makeRequest(ctx, "GET", "/memories/stats", queryParams, &stats, true)
	if err != nil {
		return nil, err
	}
	return &stats, nil
}

// HealthCheck checks API health status
func (c *Client) HealthCheck(ctx context.Context) (*HealthStatus, error) {
	var health HealthStatus
	err := c.makeRequest(ctx, "GET", "/health", nil, &health, true)
	if err != nil {
		return nil, err
	}
	return &health, nil
}

// GetMetrics retrieves system metrics
func (c *Client) GetMetrics(ctx context.Context) (*SystemMetrics, error) {
	var metrics SystemMetrics
	err := c.makeRequest(ctx, "GET", "/metrics", nil, &metrics, true)
	if err != nil {
		return nil, err
	}
	return &metrics, nil
}

// ClearCache clears the client's cache
func (c *Client) ClearCache() {
	c.cacheMutex.Lock()
	defer c.cacheMutex.Unlock()
	c.cache = make(map[string]*cacheEntry)
}

// GetConfig returns the client's configuration (with masked API key)
func (c *Client) GetConfig() *Config {
	config := c.config.Clone()
	config.APIKey = "***" // Mask API key for security
	return config
}
