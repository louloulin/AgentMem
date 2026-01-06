// Package agentmem provides the official Go SDK for AgentMem API
// Enterprise-grade memory management for AI agents.
package agentmem

import (
	"time"
)

// MemoryType represents the type of memory
type MemoryType string

const (
	// MemoryTypeEpisodic represents event-based memories
	MemoryTypeEpisodic MemoryType = "episodic"
	// MemoryTypeSemantic represents factual knowledge
	MemoryTypeSemantic MemoryType = "semantic"
	// MemoryTypeProcedural represents how-to knowledge
	MemoryTypeProcedural MemoryType = "procedural"
	// MemoryTypeUntyped represents unclassified memories
	MemoryTypeUntyped MemoryType = "untyped"
)

// ImportanceLevel represents memory importance levels
type ImportanceLevel int

const (
	ImportanceLow ImportanceLevel = iota + 1
	ImportanceMedium
	ImportanceHigh
	ImportanceCritical
)

// MatchType represents search match types
type MatchType string

const (
	MatchTypeExactText   MatchType = "exact_text"
	MatchTypePartialText MatchType = "partial_text"
	MatchTypeSemantic    MatchType = "semantic"
	MatchTypeMetadata    MatchType = "metadata"
)

// Memory represents a memory record
type Memory struct {
	ID           string                 `json:"id"`
	Content      string                 `json:"content"`
	MemoryType   MemoryType             `json:"memory_type"`
	AgentID      string                 `json:"agent_id"`
	UserID       *string                `json:"user_id,omitempty"`
	SessionID    *string                `json:"session_id,omitempty"`
	Importance   float64                `json:"importance"`
	Metadata     map[string]interface{} `json:"metadata,omitempty"`
	CreatedAt    *time.Time             `json:"created_at,omitempty"`
	UpdatedAt    *time.Time             `json:"updated_at,omitempty"`
	AccessCount  int                    `json:"access_count"`
	LastAccessed *time.Time             `json:"last_accessed,omitempty"`
	Embedding    []float64              `json:"embedding,omitempty"`
}

// SearchQuery represents search parameters
type SearchQuery struct {
	AgentID         string                 `json:"agent_id"`
	TextQuery       *string                `json:"text_query,omitempty"`
	VectorQuery     []float64              `json:"vector_query,omitempty"`
	MemoryType      *MemoryType            `json:"memory_type,omitempty"`
	UserID          *string                `json:"user_id,omitempty"`
	MinImportance   *float64               `json:"min_importance,omitempty"`
	MaxAgeSeconds   *int                   `json:"max_age_seconds,omitempty"`
	Limit           int                    `json:"limit"`
	MetadataFilters map[string]interface{} `json:"metadata_filters,omitempty"`
}

// SearchResult represents a search result with score and match type
type SearchResult struct {
	Memory    Memory    `json:"memory"`
	Score     float64   `json:"score"`
	MatchType MatchType `json:"match_type"`
}

// MemoryStats represents memory statistics
type MemoryStats struct {
	TotalMemories           int            `json:"total_memories"`
	MemoriesByType          map[string]int `json:"memories_by_type"`
	MemoriesByAgent         map[string]int `json:"memories_by_agent"`
	AverageImportance       float64        `json:"average_importance"`
	OldestMemoryAgeDays     float64        `json:"oldest_memory_age_days"`
	MostAccessedMemoryID    *string        `json:"most_accessed_memory_id,omitempty"`
	TotalAccessCount        int            `json:"total_access_count"`
}

// CreateMemoryParams represents parameters for creating a memory
type CreateMemoryParams struct {
	Content    string                 `json:"content"`
	AgentID    string                 `json:"agent_id"`
	MemoryType *MemoryType            `json:"memory_type,omitempty"`
	UserID     *string                `json:"user_id,omitempty"`
	SessionID  *string                `json:"session_id,omitempty"`
	Importance *float64               `json:"importance,omitempty"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
}

// UpdateMemoryParams represents parameters for updating a memory
type UpdateMemoryParams struct {
	Content    *string                `json:"content,omitempty"`
	Importance *float64               `json:"importance,omitempty"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
}

// BatchCreateMemoryParams represents parameters for batch memory creation
type BatchCreateMemoryParams struct {
	Memories []CreateMemoryParams `json:"memories"`
}

// HealthStatus represents API health status
type HealthStatus struct {
	Status    string            `json:"status"`
	Version   string            `json:"version"`
	Uptime    int64             `json:"uptime"`
	Timestamp string            `json:"timestamp"`
	Services  map[string]string `json:"services"`
}

// SystemMetrics represents system performance metrics
type SystemMetrics struct {
	RequestsPerSecond     float64 `json:"requests_per_second"`
	AverageResponseTime   float64 `json:"average_response_time"`
	ActiveConnections     int     `json:"active_connections"`
	MemoryUsage           float64 `json:"memory_usage"`
	CPUUsage              float64 `json:"cpu_usage"`
	CacheHitRate          float64 `json:"cache_hit_rate"`
}

// Config represents client configuration
type Config struct {
	// APIKey for authentication (required)
	APIKey string
	
	// BaseURL for the AgentMem API (default: https://api.agentmem.dev)
	BaseURL string
	
	// APIVersion (default: v1)
	APIVersion string
	
	// Timeout for requests (default: 30s)
	Timeout time.Duration
	
	// MaxRetries for failed requests (default: 3)
	MaxRetries int
	
	// RetryDelay between retries (default: 1s)
	RetryDelay time.Duration
	
	// EnableCompression for requests/responses (default: true)
	EnableCompression bool
	
	// EnableCaching for GET requests (default: true)
	EnableCaching bool
	
	// CacheTTL for cached responses (default: 5m)
	CacheTTL time.Duration
	
	// EnableLogging for debug output (default: false)
	EnableLogging bool
	
	// CustomHeaders to include in requests
	CustomHeaders map[string]string
}

// RequestOptions represents options for individual requests
type RequestOptions struct {
	Timeout    *time.Duration
	Retries    *int
	UseCache   *bool
	Headers    map[string]string
}

// APIResponse represents a generic API response
type APIResponse struct {
	Data    interface{} `json:"data,omitempty"`
	Error   *string     `json:"error,omitempty"`
	Message *string     `json:"message,omitempty"`
	Status  int         `json:"status"`
}

// SearchResponse represents search API response
type SearchResponse struct {
	Results []SearchResult `json:"results"`
}

// BatchCreateResponse represents batch create API response
type BatchCreateResponse struct {
	IDs []string `json:"ids"`
}

// CreateMemoryResponse represents create memory API response
type CreateMemoryResponse struct {
	ID string `json:"id"`
}
