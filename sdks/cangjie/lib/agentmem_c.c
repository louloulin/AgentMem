/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file agentmem_c.c
 * @brief AgentMem C FFI Implementation - Mock implementation for Cangjie SDK testing
 */

#include "agentmem_c.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

// Mock client structure
struct AgentMemClient {
    char* config;
    bool connected;
    int memory_count;
};

// Global error state
static char* last_error_message = NULL;
static uint32_t last_error_code = 0;

// Error handling functions
const char* agentmem_get_last_error() {
    return last_error_message ? last_error_message : "";
}

uint32_t agentmem_get_last_error_code() {
    return last_error_code;
}

void agentmem_clear_last_error() {
    if (last_error_message) {
        free(last_error_message);
        last_error_message = NULL;
    }
    last_error_code = 0;
}

void agentmem_free_string(const char* str) {
    if (str) {
        free((void*)str);
    }
}

// Helper function to set error
static void set_error(uint32_t code, const char* message) {
    agentmem_clear_last_error();
    last_error_code = code;
    if (message) {
        last_error_message = strdup(message);
    }
}

// Helper function to clear error
static void clear_error(void) {
    agentmem_clear_last_error();
}

// Helper function to duplicate string
static char* safe_strdup(const char* str) {
    if (!str) return NULL;
    return strdup(str);
}

// Client management functions
AgentMemClient* agentmem_client_new(const char* config_json) {
    if (!config_json) {
        set_error(1001, "Invalid configuration JSON");
        return NULL;
    }
    
    AgentMemClient* client = malloc(sizeof(AgentMemClient));
    if (!client) {
        set_error(1014, "Out of memory");
        return NULL;
    }
    
    client->config = safe_strdup(config_json);
    client->connected = true;
    client->memory_count = 0;
    
    clear_error();
    return client;
}

void agentmem_client_destroy(AgentMemClient* client) {
    if (client) {
        if (client->config) {
            free(client->config);
        }
        free(client);
    }
}

bool agentmem_client_is_connected(AgentMemClient* client) {
    return client ? client->connected : false;
}

// Memory operations
char* agentmem_add_memory(AgentMemClient* client, const CMemory* memory) {
    if (!client || !memory) {
        set_error(1001, "Invalid parameters");
        return NULL;
    }
    
    // Generate a mock memory ID
    char* memory_id = malloc(32);
    if (!memory_id) {
        set_error(1014, "Out of memory");
        return NULL;
    }
    
    snprintf(memory_id, 32, "mem_%d_%ld", client->memory_count++, time(NULL));
    clear_error();
    return memory_id;
}

int32_t agentmem_get_memory(AgentMemClient* client, const char* memory_id, CMemory* out_memory) {
    if (!client || !memory_id || !out_memory) {
        set_error(1001, "Invalid parameters");
        return -1;
    }
    
    // Mock implementation - return a fake memory
    memset(out_memory, 0, sizeof(CMemory));
    out_memory->id = safe_strdup(memory_id);
    out_memory->agent_id = safe_strdup("agent-123");
    out_memory->content = safe_strdup("Mock memory content");
    out_memory->memory_type = 1; // Semantic
    out_memory->importance = 0.5f;
    out_memory->created_at = time(NULL);
    out_memory->last_accessed_at = time(NULL);
    out_memory->access_count = 1;
    out_memory->version = 1;
    
    clear_error();
    return 0;
}

int32_t agentmem_update_memory(AgentMemClient* client, const char* memory_id, const char* content) {
    if (!client || !memory_id || !content) {
        set_error(1001, "Invalid parameters");
        return -1;
    }
    
    // Mock implementation - always succeed
    clear_error();
    return 0;
}

int32_t agentmem_delete_memory(AgentMemClient* client, const char* memory_id) {
    if (!client || !memory_id) {
        set_error(1001, "Invalid parameters");
        return -1;
    }
    
    // Mock implementation - always succeed
    clear_error();
    return 0;
}

// Search operations
int32_t agentmem_search_memories(AgentMemClient* client, const char* query, uint32_t limit, CSearchResultArray* out_results) {
    if (!client || !query || !out_results) {
        set_error(1001, "Invalid parameters");
        return -1;
    }
    
    // Mock implementation - return empty results
    memset(out_results, 0, sizeof(CSearchResultArray));
    out_results->results = NULL;
    out_results->count = 0;
    
    clear_error();
    return 0;
}

int32_t agentmem_search_similar_memories(AgentMemClient* client, const char* memory_id, uint32_t limit, float threshold, CSearchResultArray* out_results) {
    if (!client || !memory_id || !out_results) {
        set_error(1001, "Invalid parameters");
        return -1;
    }
    
    // Mock implementation - return empty results
    memset(out_results, 0, sizeof(CSearchResultArray));
    out_results->results = NULL;
    out_results->count = 0;
    
    clear_error();
    return 0;
}



void agentmem_free_memory_array(CMemoryArray* arr) {
    if (arr && arr->memories) {
        for (size_t i = 0; i < arr->count; i++) {
            CMemory* mem = &arr->memories[i];
            if (mem->id) free(mem->id);
            if (mem->agent_id) free(mem->agent_id);
            if (mem->user_id) free(mem->user_id);
            if (mem->content) free(mem->content);
        }
        free(arr->memories);
        arr->memories = NULL;
        arr->count = 0;
    }
}

void agentmem_free_search_result_array(CSearchResultArray* arr) {
    if (arr && arr->results) {
        for (size_t i = 0; i < arr->count; i++) {
            CMemory* mem = &arr->results[i].memory;
            if (mem->id) free(mem->id);
            if (mem->agent_id) free(mem->agent_id);
            if (mem->user_id) free(mem->user_id);
            if (mem->content) free(mem->content);
        }
        free(arr->results);
        arr->results = NULL;
        arr->count = 0;
    }
}



// Configuration and debugging
void agentmem_set_log_level(uint32_t level) {
    // Mock implementation
    (void)level;
}

const char* agentmem_get_version(void) {
    return "AgentMem-C-Mock-1.0.0";
}

bool agentmem_health_check(AgentMemClient* client) {
    return client ? client->connected : false;
}

// Stub implementations for other functions
int32_t agentmem_add_memories_batch(AgentMemClient* client, const CMemory* memories, size_t count, CBatchResult* out_result) {
    (void)client; (void)memories; (void)count; (void)out_result;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_delete_memories_batch(AgentMemClient* client, const char** memory_ids, size_t count, CBatchResult* out_result) {
    (void)client; (void)memory_ids; (void)count; (void)out_result;
    set_error(1007, "Not implemented in mock");
    return -1;
}

void agentmem_free_batch_result(CBatchResult* result) {
    (void)result;
}

void agentmem_free_paginated_result(CPaginatedResult* result) {
    (void)result;
}

void agentmem_free_memory_stats(CMemoryStats* stats) {
    (void)stats;
}

int32_t agentmem_get_memories_paginated(AgentMemClient* client, const char* agent_id, uint32_t page, uint32_t page_size, CPaginatedResult* out_result) {
    (void)client; (void)agent_id; (void)page; (void)page_size; (void)out_result;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_get_memories_by_type_paginated(AgentMemClient* client, const char* agent_id, uint32_t memory_type, uint32_t page, uint32_t page_size, CPaginatedResult* out_result) {
    (void)client; (void)agent_id; (void)memory_type; (void)page; (void)page_size; (void)out_result;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_get_memory_stats(AgentMemClient* client, const char* agent_id, CMemoryStats* out_stats) {
    (void)client; (void)agent_id; (void)out_stats;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_get_global_stats(AgentMemClient* client, CMemoryStats* out_stats) {
    (void)client; (void)out_stats;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_compress_memories(AgentMemClient* client, const char* agent_id, float compression_ratio) {
    (void)client; (void)agent_id; (void)compression_ratio;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_export_memories(AgentMemClient* client, const char* agent_id, const char* format, const char* output_path) {
    (void)client; (void)agent_id; (void)format; (void)output_path;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_import_memories(AgentMemClient* client, const char* agent_id, const char* format, const char* input_path, CBatchResult* out_result) {
    (void)client; (void)agent_id; (void)format; (void)input_path; (void)out_result;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_generate_embedding(AgentMemClient* client, const char* text, float** out_embedding, uint32_t* out_length) {
    (void)client; (void)text; (void)out_embedding; (void)out_length;
    set_error(1007, "Not implemented in mock");
    return -1;
}

void agentmem_free_embedding(float* embedding) {
    if (embedding) {
        free(embedding);
    }
}

int32_t agentmem_add_memory_relation(AgentMemClient* client, const char* from_memory_id, const char* to_memory_id, const char* relation_type, float strength) {
    (void)client; (void)from_memory_id; (void)to_memory_id; (void)relation_type; (void)strength;
    set_error(1007, "Not implemented in mock");
    return -1;
}

int32_t agentmem_get_related_memories(AgentMemClient* client, const char* memory_id, const char* relation_type, uint32_t max_depth, CSearchResultArray* out_results) {
    (void)client; (void)memory_id; (void)relation_type; (void)max_depth; (void)out_results;
    set_error(1007, "Not implemented in mock");
    return -1;
}
