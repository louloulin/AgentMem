package main

/*
AgentMem Go SDK Demo

This demo showcases the AgentMem Go SDK capabilities:
1. Client initialization and configuration
2. Memory management (CRUD operations)
3. Advanced search functionality
4. Batch operations
5. Statistics and monitoring
6. Error handling
*/

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"

	// For demo purposes, we'll use the local SDK
	// In real usage, you would: go get github.com/agentmem/agentmem-go
	agentmem "../../sdks/go"
)

// GoSDKDemo demonstrates AgentMem Go SDK functionality
type GoSDKDemo struct {
	client           *agentmem.Client
	demoAgentID      string
	createdMemoryIDs []string
}

// NewGoSDKDemo creates a new demo instance
func NewGoSDKDemo() (*GoSDKDemo, error) {
	// For demo purposes, we'll use a mock configuration
	// In real usage, you would set AGENTMEM_API_KEY environment variable
	config := agentmem.NewConfig("demo-api-key-12345")
	config = config.WithBaseURL("http://localhost:8080") // Mock server for demo
	config = config.WithTimeout(30 * time.Second)
	config = config.WithRetries(3, 1*time.Second)
	config = config.WithCaching(true, 5*time.Minute)
	config = config.WithLogging(true)

	client, err := agentmem.NewClient(config)
	if err != nil {
		return nil, fmt.Errorf("failed to create client: %w", err)
	}

	return &GoSDKDemo{
		client:           client,
		demoAgentID:      "demo_agent_001",
		createdMemoryIDs: make([]string, 0),
	}, nil
}

// RunDemo runs the complete SDK demo
func (d *GoSDKDemo) RunDemo(ctx context.Context) error {
	fmt.Println("🧠 AgentMem Go SDK Demo")
	fmt.Println(strings.Repeat("=", 50))

	// Test 1: Basic memory operations
	if err := d.demoBasicOperations(ctx); err != nil {
		return fmt.Errorf("basic operations demo failed: %w", err)
	}

	// Test 2: Advanced search
	if err := d.demoAdvancedSearch(ctx); err != nil {
		return fmt.Errorf("advanced search demo failed: %w", err)
	}

	// Test 3: Batch operations
	if err := d.demoBatchOperations(ctx); err != nil {
		return fmt.Errorf("batch operations demo failed: %w", err)
	}

	// Test 4: Statistics and monitoring
	if err := d.demoStatistics(ctx); err != nil {
		return fmt.Errorf("statistics demo failed: %w", err)
	}

	// Test 5: Error handling
	if err := d.demoErrorHandling(ctx); err != nil {
		return fmt.Errorf("error handling demo failed: %w", err)
	}

	fmt.Println("\n✅ All demo tests completed successfully!")

	// Cleanup
	d.cleanup(ctx)

	return nil
}

func (d *GoSDKDemo) demoBasicOperations(ctx context.Context) error {
	fmt.Println("\n📝 Demo 1: Basic Memory Operations")
	fmt.Println(strings.Repeat("-", 30))

	// Add memories
	fmt.Println("Adding memories...")

	memoryType1 := agentmem.MemoryTypeSemantic
	importance1 := 0.8
	memoryID1, err := d.client.AddMemory(ctx, agentmem.CreateMemoryParams{
		Content:    "The user prefers dark mode in the application",
		AgentID:    d.demoAgentID,
		MemoryType: &memoryType1,
		Importance: &importance1,
		Metadata: map[string]interface{}{
			"category":  "user_preferences",
			"ui_theme":  "dark",
		},
	})
	if err != nil {
		return fmt.Errorf("failed to add semantic memory: %w", err)
	}
	d.createdMemoryIDs = append(d.createdMemoryIDs, memoryID1)
	fmt.Printf("✓ Added semantic memory: %s\n", memoryID1)

	memoryType2 := agentmem.MemoryTypeEpisodic
	importance2 := 0.6
	memoryID2, err := d.client.AddMemory(ctx, agentmem.CreateMemoryParams{
		Content:    "User clicked the 'Export Data' button at 2024-01-15 14:30:00",
		AgentID:    d.demoAgentID,
		MemoryType: &memoryType2,
		Importance: &importance2,
		Metadata: map[string]interface{}{
			"action":    "export_data",
			"timestamp": "2024-01-15T14:30:00Z",
		},
	})
	if err != nil {
		return fmt.Errorf("failed to add episodic memory: %w", err)
	}
	d.createdMemoryIDs = append(d.createdMemoryIDs, memoryID2)
	fmt.Printf("✓ Added episodic memory: %s\n", memoryID2)

	memoryType3 := agentmem.MemoryTypeProcedural
	importance3 := 0.9
	memoryID3, err := d.client.AddMemory(ctx, agentmem.CreateMemoryParams{
		Content:    "To export data: 1) Go to Settings, 2) Click Export, 3) Choose format",
		AgentID:    d.demoAgentID,
		MemoryType: &memoryType3,
		Importance: &importance3,
		Metadata: map[string]interface{}{
			"procedure": "data_export",
			"steps":     3,
		},
	})
	if err != nil {
		return fmt.Errorf("failed to add procedural memory: %w", err)
	}
	d.createdMemoryIDs = append(d.createdMemoryIDs, memoryID3)
	fmt.Printf("✓ Added procedural memory: %s\n", memoryID3)

	// Retrieve memory
	fmt.Printf("\nRetrieving memory %s...\n", memoryID1)
	memory, err := d.client.GetMemory(ctx, memoryID1)
	if err != nil {
		return fmt.Errorf("failed to retrieve memory: %w", err)
	}
	fmt.Printf("✓ Retrieved: %s\n", memory.Content)
	fmt.Printf("  Type: %s\n", memory.MemoryType)
	fmt.Printf("  Importance: %.1f\n", memory.Importance)
	fmt.Printf("  Metadata: %v\n", memory.Metadata)

	// Update memory
	fmt.Printf("\nUpdating memory %s...\n", memoryID1)
	newImportance := 0.9
	updatedMemory, err := d.client.UpdateMemory(ctx, memoryID1, agentmem.UpdateMemoryParams{
		Importance: &newImportance,
		Metadata: map[string]interface{}{
			"category": "user_preferences",
			"ui_theme": "dark",
			"updated":  true,
		},
	})
	if err != nil {
		return fmt.Errorf("failed to update memory: %w", err)
	}
	fmt.Printf("✓ Updated importance to: %.1f\n", updatedMemory.Importance)

	return nil
}

func (d *GoSDKDemo) demoAdvancedSearch(ctx context.Context) error {
	fmt.Println("\n🔍 Demo 2: Advanced Search")
	fmt.Println(strings.Repeat("-", 25))

	// Text search
	fmt.Println("Searching for 'user preferences'...")
	textQuery := "user preferences"
	results1, err := d.client.SearchMemories(ctx, agentmem.SearchQuery{
		AgentID:   d.demoAgentID,
		TextQuery: &textQuery,
		Limit:     5,
	})
	if err != nil {
		return fmt.Errorf("failed to search memories: %w", err)
	}

	fmt.Printf("✓ Found %d results:\n", len(results1))
	for i, result := range results1 {
		preview := result.Memory.Content
		if len(preview) > 50 {
			preview = preview[:50] + "..."
		}
		fmt.Printf("  %d. %s (score: %.3f)\n", i+1, preview, result.Score)
	}

	// Search with filters
	fmt.Println("\nSearching semantic memories with high importance...")
	textQuery2 := "preferences"
	memoryType := agentmem.MemoryTypeSemantic
	minImportance := 0.7
	results2, err := d.client.SearchMemories(ctx, agentmem.SearchQuery{
		AgentID:       d.demoAgentID,
		TextQuery:     &textQuery2,
		MemoryType:    &memoryType,
		MinImportance: &minImportance,
		Limit:         3,
	})
	if err != nil {
		return fmt.Errorf("failed to search with filters: %w", err)
	}

	fmt.Printf("✓ Found %d high-importance semantic memories\n", len(results2))

	// Metadata search
	fmt.Println("\nSearching by metadata...")
	results3, err := d.client.SearchMemories(ctx, agentmem.SearchQuery{
		AgentID: d.demoAgentID,
		MetadataFilters: map[string]interface{}{
			"category": "user_preferences",
		},
		Limit: 5,
	})
	if err != nil {
		return fmt.Errorf("failed to search by metadata: %w", err)
	}

	fmt.Printf("✓ Found %d memories with user_preferences category\n", len(results3))

	return nil
}

func (d *GoSDKDemo) demoBatchOperations(ctx context.Context) error {
	fmt.Println("\n📦 Demo 3: Batch Operations")
	fmt.Println(strings.Repeat("-", 25))

	// Batch add memories
	fmt.Println("Adding memories in batch...")

	semanticType := agentmem.MemoryTypeSemantic
	episodicType := agentmem.MemoryTypeEpisodic
	importance1 := 0.7
	importance2 := 0.6
	importance3 := 0.5

	batchMemories := []agentmem.CreateMemoryParams{
		{
			Content:    "User's favorite programming language is Go",
			AgentID:    d.demoAgentID,
			MemoryType: &semanticType,
			Importance: &importance1,
			Metadata: map[string]interface{}{
				"category": "preferences",
				"topic":    "programming",
			},
		},
		{
			Content:    "User completed Go tutorial on 2024-01-10",
			AgentID:    d.demoAgentID,
			MemoryType: &episodicType,
			Importance: &importance2,
			Metadata: map[string]interface{}{
				"achievement": "tutorial_completion",
				"language":    "go",
			},
		},
		{
			Content:    "User asked about Go concurrency patterns",
			AgentID:    d.demoAgentID,
			MemoryType: &episodicType,
			Importance: &importance3,
			Metadata: map[string]interface{}{
				"topic":    "concurrency",
				"question": true,
			},
		},
	}

	batchIDs, err := d.client.BatchAddMemories(ctx, agentmem.BatchCreateMemoryParams{
		Memories: batchMemories,
	})
	if err != nil {
		return fmt.Errorf("failed to batch add memories: %w", err)
	}

	d.createdMemoryIDs = append(d.createdMemoryIDs, batchIDs...)
	fmt.Printf("✓ Added %d memories in batch\n", len(batchIDs))

	for i, memoryID := range batchIDs {
		fmt.Printf("  %d. %s\n", i+1, memoryID)
	}

	return nil
}

func (d *GoSDKDemo) demoStatistics(ctx context.Context) error {
	fmt.Println("\n📊 Demo 4: Statistics & Monitoring")
	fmt.Println(strings.Repeat("-", 32))

	// Get memory statistics
	fmt.Println("Fetching memory statistics...")
	stats, err := d.client.GetMemoryStats(ctx, d.demoAgentID)
	if err != nil {
		return fmt.Errorf("failed to get memory stats: %w", err)
	}

	fmt.Printf("✓ Total memories: %d\n", stats.TotalMemories)
	fmt.Printf("✓ Average importance: %.3f\n", stats.AverageImportance)
	fmt.Printf("✓ Total access count: %d\n", stats.TotalAccessCount)

	fmt.Println("\nMemories by type:")
	for memoryType, count := range stats.MemoriesByType {
		fmt.Printf("  %s: %d\n", memoryType, count)
	}

	// Health check
	fmt.Println("\nChecking API health...")
	health, err := d.client.HealthCheck(ctx)
	if err != nil {
		fmt.Printf("⚠️  Health check failed (expected in demo): %v\n", err)
	} else {
		fmt.Printf("✓ API Status: %s\n", health.Status)
		fmt.Printf("✓ Version: %s\n", health.Version)
	}

	// System metrics
	fmt.Println("\nFetching system metrics...")
	metrics, err := d.client.GetMetrics(ctx)
	if err != nil {
		fmt.Printf("⚠️  Metrics fetch failed (expected in demo): %v\n", err)
	} else {
		fmt.Printf("✓ Active connections: %d\n", metrics.ActiveConnections)
		fmt.Printf("✓ Cache hit rate: %.3f\n", metrics.CacheHitRate)
	}

	return nil
}

func (d *GoSDKDemo) demoErrorHandling(ctx context.Context) error {
	fmt.Println("\n⚠️  Demo 5: Error Handling")
	fmt.Println(strings.Repeat("-", 25))

	// Test invalid memory ID
	fmt.Println("Testing invalid memory ID...")
	_, err := d.client.GetMemory(ctx, "invalid-memory-id")
	if err != nil {
		switch e := err.(type) {
		case *agentmem.NotFoundError:
			fmt.Printf("✓ Caught NotFoundError: %s\n", e.Error())
		case *agentmem.NetworkError:
			fmt.Printf("✓ Caught NetworkError (expected in demo): %s\n", e.Error())
		default:
			fmt.Printf("✓ Caught other error (expected in demo): %s\n", e.Error())
		}
	}

	// Test invalid search query
	fmt.Println("\nTesting invalid search parameters...")
	_, err = d.client.SearchMemories(ctx, agentmem.SearchQuery{
		AgentID: "", // Invalid empty agent ID
		Limit:   5,
	})
	if err != nil {
		switch e := err.(type) {
		case *agentmem.ValidationError:
			fmt.Printf("✓ Caught ValidationError: %s\n", e.Error())
		default:
			fmt.Printf("✓ Caught other error (expected in demo): %s\n", e.Error())
		}
	}

	return nil
}

func (d *GoSDKDemo) cleanup(ctx context.Context) {
	fmt.Println("\n🧹 Cleaning up demo data...")

	cleanupCount := 0
	for _, memoryID := range d.createdMemoryIDs {
		if err := d.client.DeleteMemory(ctx, memoryID); err != nil {
			fmt.Printf("⚠️  Failed to delete %s: %v\n", memoryID, err)
		} else {
			cleanupCount++
		}
	}

	fmt.Printf("✓ Cleaned up %d memories\n", cleanupCount)
}

func main() {
	fmt.Println("Starting AgentMem Go SDK Demo...")
	fmt.Println("Note: This demo uses mock data and may show connection errors - this is expected!")
	fmt.Println()

	demo, err := NewGoSDKDemo()
	if err != nil {
		log.Fatalf("Failed to create demo: %v", err)
	}

	ctx := context.Background()
	if err := demo.RunDemo(ctx); err != nil {
		log.Fatalf("Demo failed: %v", err)
	}
}
