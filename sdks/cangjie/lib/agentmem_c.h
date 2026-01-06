/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file agentmem_c.h
 * @brief AgentMem C FFI Header - Mock implementation for Cangjie SDK testing
 */

#ifndef AGENTMEM_C_H
#define AGENTMEM_C_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declarations
typedef struct AgentMemClient AgentMemClient;

// Memory structure (simplified for testing)
typedef struct {
    char* id;
    char* agent_id;
    char* user_id;
    uint32_t memory_type;
    char* content;
    float importance;
    int64_t created_at;
    int64_t last_accessed_at;
    uint32_t access_count;
    int64_t expires_at;
    uint32_t version;
} CMemory;

// Memory array structure
typedef struct {
    CMemory* memories;
    size_t count;
} CMemoryArray;

// Search result structure
typedef struct {
    CMemory memory;
    float score;
    float relevance;
} CSearchResult;

// Search result array structure
typedef struct {
    CSearchResult* results;
    size_t count;
} CSearchResultArray;

// Batch result structure
typedef struct {
    size_t total;
    size_t success_count;
    size_t failure_count;
    char** success_ids;
    char** failure_messages;
} CBatchResult;

// Paginated result structure
typedef struct {
    CMemoryArray memories;
    uint32_t page;
    uint32_t page_size;
    uint32_t total_count;
    uint32_t total_pages;
} CPaginatedResult;

// Memory statistics structure
typedef struct {
    uint64_t total_memories;
    uint64_t episodic_count;
    uint64_t semantic_count;
    uint64_t procedural_count;
    uint64_t working_count;
    float average_importance;
    uint64_t total_size;
    uint64_t last_updated;
} CMemoryStats;

// Error handling functions
const char* agentmem_get_last_error(void);
uint32_t agentmem_get_last_error_code(void);
void agentmem_clear_last_error(void);
void agentmem_free_string(const char* str);

// Client management functions
AgentMemClient* agentmem_client_new(const char* config_json);
void agentmem_client_destroy(AgentMemClient* client);
bool agentmem_client_is_connected(AgentMemClient* client);

// Memory operations
char* agentmem_add_memory(AgentMemClient* client, const CMemory* memory);
int32_t agentmem_get_memory(AgentMemClient* client, const char* memory_id, CMemory* out_memory);
int32_t agentmem_update_memory(AgentMemClient* client, const char* memory_id, const char* content);
int32_t agentmem_delete_memory(AgentMemClient* client, const char* memory_id);

// Search operations
int32_t agentmem_search_memories(AgentMemClient* client, const char* query, uint32_t limit, CSearchResultArray* out_results);
int32_t agentmem_search_similar_memories(AgentMemClient* client, const char* memory_id, uint32_t limit, float threshold, CSearchResultArray* out_results);

// Batch operations
int32_t agentmem_add_memories_batch(AgentMemClient* client, const CMemory* memories, size_t count, CBatchResult* out_result);
int32_t agentmem_delete_memories_batch(AgentMemClient* client, const char** memory_ids, size_t count, CBatchResult* out_result);

// Paginated queries
int32_t agentmem_get_memories_paginated(AgentMemClient* client, const char* agent_id, uint32_t page, uint32_t page_size, CPaginatedResult* out_result);
int32_t agentmem_get_memories_by_type_paginated(AgentMemClient* client, const char* agent_id, uint32_t memory_type, uint32_t page, uint32_t page_size, CPaginatedResult* out_result);

// Statistics
int32_t agentmem_get_memory_stats(AgentMemClient* client, const char* agent_id, CMemoryStats* out_stats);
int32_t agentmem_get_global_stats(AgentMemClient* client, CMemoryStats* out_stats);

// Memory management
void agentmem_free_memory_array(CMemoryArray* arr);
void agentmem_free_search_result_array(CSearchResultArray* arr);
void agentmem_free_batch_result(CBatchResult* result);
void agentmem_free_paginated_result(CPaginatedResult* result);
void agentmem_free_memory_stats(CMemoryStats* stats);

// Error handling
const char* agentmem_get_last_error(void);
uint32_t agentmem_get_last_error_code(void);
void agentmem_clear_last_error(void);

// Configuration and debugging
void agentmem_set_log_level(uint32_t level);
const char* agentmem_get_version(void);
bool agentmem_health_check(AgentMemClient* client);

// Advanced features
int32_t agentmem_compress_memories(AgentMemClient* client, const char* agent_id, float compression_ratio);
int32_t agentmem_export_memories(AgentMemClient* client, const char* agent_id, const char* format, const char* output_path);
int32_t agentmem_import_memories(AgentMemClient* client, const char* agent_id, const char* format, const char* input_path, CBatchResult* out_result);

// Vector operations
int32_t agentmem_generate_embedding(AgentMemClient* client, const char* text, float** out_embedding, uint32_t* out_length);
void agentmem_free_embedding(float* embedding);

// Relationship and graph operations
int32_t agentmem_add_memory_relation(AgentMemClient* client, const char* from_memory_id, const char* to_memory_id, const char* relation_type, float strength);
int32_t agentmem_get_related_memories(AgentMemClient* client, const char* memory_id, const char* relation_type, uint32_t max_depth, CSearchResultArray* out_results);

#ifdef __cplusplus
}
#endif

#endif // AGENTMEM_C_H
