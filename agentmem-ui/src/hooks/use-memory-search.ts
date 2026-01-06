/**
 * Memory Search Hook
 * 
 * Provides automatic memory search functionality with debouncing.
 */

'use client';

import { useState, useCallback, useRef, useEffect } from 'react';
import { MemoryDisplayItem } from '@/components/MemoryPanel';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

interface UseMemorySearchOptions {
  agentId: string | null;
  userId: string;
  enabled?: boolean;
  debounceMs?: number;
}

interface UseMemorySearchResult {
  memories: MemoryDisplayItem[];
  loading: boolean;
  searchMemories: (query: string) => Promise<void>;
  clearMemories: () => void;
}

export function useMemorySearch({
  agentId,
  userId,
  enabled = true,
  debounceMs = 800,
}: UseMemorySearchOptions): UseMemorySearchResult {
  const [memories, setMemories] = useState<MemoryDisplayItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [lastQuery, setLastQuery] = useState('');
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  
  const searchMemories = useCallback(async (query: string) => {
    // Don't search if disabled or query is empty/unchanged
    if (!enabled || !agentId || !query.trim() || query === lastQuery) {
      return;
    }
    
    // Cancel previous timeout
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    
    // Debounce search
    timeoutRef.current = setTimeout(async () => {
      setLoading(true);
      setLastQuery(query);
      
      try {
        // 🔧 修复: 使用 POST 方法，因为服务器端定义的是 POST
        const url = `${API_BASE_URL}/api/v1/memories/search`;
        
        // Get token from localStorage
        const token = typeof window !== 'undefined' ? localStorage.getItem('auth_token') : null;
        
        // Construct request body
        const requestBody = {
          query: query,
          user_id: userId,
          limit: 10,
          ...(agentId ? { agent_id: agentId } : {}),
        };
        
        const response = await fetch(url, {
          method: 'POST',  // ✅ 修复: 改为 POST
          headers: {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
          },
          body: JSON.stringify(requestBody),
        });
        
        if (!response.ok) {
          throw new Error(`Search failed: ${response.statusText}`);
        }
        
        const data = await response.json();
        
        // 🔧 修复: 后端返回的是 SearchResponse，需要从 results 字段获取数据
        const results = data.data?.results || data.data || [];
        
        // Transform API response to display format
        const displayMemories: MemoryDisplayItem[] = results.map((mem: any) => ({
          id: mem.id,
          title: extractTitle(mem.content),
          content: mem.content,
          memory_type: mem.memory_type || 'Unknown',
          relevance_score: mem.score || 0,
          created_at: mem.created_at,
          scope: mem.scope || 'unknown',
          metadata: mem.metadata,
        }));
        
        setMemories(displayMemories);
      } catch (err) {
        console.error('[useMemorySearch] Search failed:', err);
        setMemories([]);
      } finally {
        setLoading(false);
      }
    }, debounceMs);
  }, [enabled, agentId, userId, lastQuery, debounceMs]);
  
  const clearMemories = useCallback(() => {
    setMemories([]);
    setLastQuery('');
  }, []);
  
  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);
  
  return {
    memories,
    loading,
    searchMemories,
    clearMemories,
  };
}

// Extract title from content (first 50 characters)
function extractTitle(content: string): string {
  const cleaned = content.trim();
  if (cleaned.length <= 50) {
    return cleaned;
  }
  
  // Try to break at word boundary
  const truncated = cleaned.substring(0, 50);
  const lastSpace = truncated.lastIndexOf(' ');
  
  if (lastSpace > 30) {
    return truncated.substring(0, lastSpace) + '...';
  }
  
  return truncated + '...';
}

