#!/usr/bin/env node

/**
 * AgentMem JavaScript SDK Demo
 * 
 * This demo showcases the AgentMem JavaScript SDK capabilities:
 * 1. Client initialization and configuration
 * 2. Memory management (CRUD operations)
 * 3. Advanced search functionality
 * 4. Batch operations
 * 5. Statistics and monitoring
 * 6. Error handling
 */

// For demo purposes, we'll simulate the SDK imports
// In real usage, you would: npm install @agentmem/client
const { 
    AgentMemClient, 
    MemoryType, 
    createSearchQuery,
    createMemoryParams,
    AgentMemError,
    AuthenticationError,
    ValidationError,
    NetworkError,
    NotFoundError
} = require('../../sdks/javascript/src/index');

class JavaScriptSDKDemo {
    constructor() {
        // For demo purposes, we'll use a mock configuration
        // In real usage, you would set AGENTMEM_API_KEY environment variable
        this.client = new AgentMemClient({
            apiKey: 'demo-api-key-12345',
            baseUrl: 'http://localhost:8080', // Mock server for demo
            timeout: 30000,
            maxRetries: 3,
            enableCaching: true,
            cacheTtl: 300000,
            enableLogging: true,
        });
        
        this.demoAgentId = 'demo_agent_001';
        this.createdMemoryIds = [];
    }

    async runDemo() {
        console.log('🧠 AgentMem JavaScript SDK Demo');
        console.log('='.repeat(50));
        
        try {
            // Test 1: Basic memory operations
            await this.demoBasicOperations();
            
            // Test 2: Advanced search
            await this.demoAdvancedSearch();
            
            // Test 3: Batch operations
            await this.demoBatchOperations();
            
            // Test 4: Statistics and monitoring
            await this.demoStatistics();
            
            // Test 5: Error handling
            await this.demoErrorHandling();
            
            console.log('\n✅ All demo tests completed successfully!');
            
        } catch (error) {
            console.error(`\n❌ Demo failed with error: ${error.message}`);
            if (this.client.getConfig().enableLogging) {
                console.error(error.stack);
            }
        } finally {
            // Cleanup
            await this.cleanup();
        }
    }

    async demoBasicOperations() {
        console.log('\n📝 Demo 1: Basic Memory Operations');
        console.log('-'.repeat(30));
        
        // Add memories
        console.log('Adding memories...');
        
        const memoryId1 = await this.client.addMemory({
            content: 'The user prefers dark mode in the application',
            agent_id: this.demoAgentId,
            memory_type: MemoryType.SEMANTIC,
            importance: 0.8,
            metadata: { category: 'user_preferences', ui_theme: 'dark' }
        });
        this.createdMemoryIds.push(memoryId1);
        console.log(`✓ Added semantic memory: ${memoryId1}`);
        
        const memoryId2 = await this.client.addMemory({
            content: "User clicked the 'Export Data' button at 2024-01-15 14:30:00",
            agent_id: this.demoAgentId,
            memory_type: MemoryType.EPISODIC,
            importance: 0.6,
            metadata: { action: 'export_data', timestamp: '2024-01-15T14:30:00Z' }
        });
        this.createdMemoryIds.push(memoryId2);
        console.log(`✓ Added episodic memory: ${memoryId2}`);
        
        const memoryId3 = await this.client.addMemory({
            content: 'To export data: 1) Go to Settings, 2) Click Export, 3) Choose format',
            agent_id: this.demoAgentId,
            memory_type: MemoryType.PROCEDURAL,
            importance: 0.9,
            metadata: { procedure: 'data_export', steps: 3 }
        });
        this.createdMemoryIds.push(memoryId3);
        console.log(`✓ Added procedural memory: ${memoryId3}`);
        
        // Retrieve memory
        console.log(`\nRetrieving memory ${memoryId1}...`);
        const memory = await this.client.getMemory(memoryId1);
        console.log(`✓ Retrieved: ${memory.content}`);
        console.log(`  Type: ${memory.memory_type}`);
        console.log(`  Importance: ${memory.importance}`);
        console.log(`  Metadata:`, memory.metadata);
        
        // Update memory
        console.log(`\nUpdating memory ${memoryId1}...`);
        const updatedMemory = await this.client.updateMemory(memoryId1, {
            importance: 0.9,
            metadata: { 
                category: 'user_preferences', 
                ui_theme: 'dark', 
                updated: true 
            }
        });
        console.log(`✓ Updated importance to: ${updatedMemory.importance}`);
    }

    async demoAdvancedSearch() {
        console.log('\n🔍 Demo 2: Advanced Search');
        console.log('-'.repeat(25));
        
        // Text search
        console.log("Searching for 'user preferences'...");
        const results1 = await this.client.searchMemories({
            agent_id: this.demoAgentId,
            text_query: 'user preferences',
            limit: 5
        });
        
        console.log(`✓ Found ${results1.length} results:`);
        results1.forEach((result, i) => {
            const preview = result.memory.content.substring(0, 50) + '...';
            console.log(`  ${i + 1}. ${preview} (score: ${result.score.toFixed(3)})`);
        });
        
        // Search with filters
        console.log('\nSearching semantic memories with high importance...');
        const results2 = await this.client.searchMemories({
            agent_id: this.demoAgentId,
            text_query: 'preferences',
            memory_type: MemoryType.SEMANTIC,
            min_importance: 0.7,
            limit: 3
        });
        
        console.log(`✓ Found ${results2.length} high-importance semantic memories`);
        
        // Metadata search
        console.log('\nSearching by metadata...');
        const results3 = await this.client.searchMemories({
            agent_id: this.demoAgentId,
            metadata_filters: { category: 'user_preferences' },
            limit: 5
        });
        
        console.log(`✓ Found ${results3.length} memories with user_preferences category`);
    }

    async demoBatchOperations() {
        console.log('\n📦 Demo 3: Batch Operations');
        console.log('-'.repeat(25));
        
        // Batch add memories
        console.log('Adding memories in batch...');
        const batchMemories = [
            {
                content: "User's favorite programming language is JavaScript",
                agent_id: this.demoAgentId,
                memory_type: MemoryType.SEMANTIC,
                importance: 0.7,
                metadata: { category: 'preferences', topic: 'programming' }
            },
            {
                content: 'User completed JavaScript tutorial on 2024-01-10',
                agent_id: this.demoAgentId,
                memory_type: MemoryType.EPISODIC,
                importance: 0.6,
                metadata: { achievement: 'tutorial_completion', language: 'javascript' }
            },
            {
                content: 'User asked about Node.js frameworks',
                agent_id: this.demoAgentId,
                memory_type: MemoryType.EPISODIC,
                importance: 0.5,
                metadata: { topic: 'nodejs', question: true }
            }
        ];
        
        const batchIds = await this.client.batchAddMemories({ memories: batchMemories });
        this.createdMemoryIds.push(...batchIds);
        console.log(`✓ Added ${batchIds.length} memories in batch`);
        
        batchIds.forEach((memoryId, i) => {
            console.log(`  ${i + 1}. ${memoryId}`);
        });
    }

    async demoStatistics() {
        console.log('\n📊 Demo 4: Statistics & Monitoring');
        console.log('-'.repeat(32));
        
        // Get memory statistics
        console.log('Fetching memory statistics...');
        const stats = await this.client.getMemoryStats(this.demoAgentId);
        
        console.log(`✓ Total memories: ${stats.total_memories}`);
        console.log(`✓ Average importance: ${stats.average_importance.toFixed(3)}`);
        console.log(`✓ Total access count: ${stats.total_access_count}`);
        
        console.log('\nMemories by type:');
        Object.entries(stats.memories_by_type).forEach(([type, count]) => {
            console.log(`  ${type}: ${count}`);
        });
        
        // Health check
        console.log('\nChecking API health...');
        try {
            const health = await this.client.healthCheck();
            console.log(`✓ API Status: ${health.status || 'unknown'}`);
            console.log(`✓ Version: ${health.version || 'unknown'}`);
        } catch (error) {
            console.log(`⚠️  Health check failed (expected in demo): ${error.message}`);
        }
        
        // System metrics
        console.log('\nFetching system metrics...');
        try {
            const metrics = await this.client.getMetrics();
            console.log(`✓ Active connections: ${metrics.active_connections || 'N/A'}`);
            console.log(`✓ Cache hit rate: ${metrics.cache_hit_rate || 'N/A'}`);
        } catch (error) {
            console.log(`⚠️  Metrics fetch failed (expected in demo): ${error.message}`);
        }
    }

    async demoErrorHandling() {
        console.log('\n⚠️  Demo 5: Error Handling');
        console.log('-'.repeat(25));
        
        // Test invalid memory ID
        console.log('Testing invalid memory ID...');
        try {
            await this.client.getMemory('invalid-memory-id');
        } catch (error) {
            if (error instanceof NotFoundError) {
                console.log(`✓ Caught NotFoundError: ${error.message}`);
            } else if (error instanceof NetworkError) {
                console.log(`✓ Caught NetworkError (expected in demo): ${error.message}`);
            } else {
                console.log(`✓ Caught other error (expected in demo): ${error.message}`);
            }
        }
        
        // Test invalid search query
        console.log('\nTesting invalid search parameters...');
        try {
            await this.client.searchMemories({
                agent_id: '', // Invalid empty agent ID
                text_query: 'test',
                limit: 5
            });
        } catch (error) {
            if (error instanceof ValidationError) {
                console.log(`✓ Caught ValidationError: ${error.message}`);
            } else {
                console.log(`✓ Caught other error (expected in demo): ${error.message}`);
            }
        }
    }

    async cleanup() {
        console.log('\n🧹 Cleaning up demo data...');
        
        let cleanupCount = 0;
        for (const memoryId of this.createdMemoryIds) {
            try {
                await this.client.deleteMemory(memoryId);
                cleanupCount++;
            } catch (error) {
                console.log(`⚠️  Failed to delete ${memoryId}: ${error.message}`);
            }
        }
        
        console.log(`✓ Cleaned up ${cleanupCount} memories`);
    }
}

async function main() {
    console.log('Starting AgentMem JavaScript SDK Demo...');
    console.log('Note: This demo uses mock data and may show connection errors - this is expected!');
    console.log();
    
    const demo = new JavaScriptSDKDemo();
    await demo.runDemo();
}

// Run the demo
if (require.main === module) {
    main().catch(console.error);
}

module.exports = { JavaScriptSDKDemo };
