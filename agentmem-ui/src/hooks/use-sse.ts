/**
 * Server-Sent Events (SSE) Hook for Real-time Streaming
 * 
 * Provides a React hook for SSE connections with:
 * - Automatic reconnection
 * - Message type handling
 * - Token-based authentication
 * - Event subscription system
 * - Connection state management
 * 
 * Usage:
 * ```tsx
 * const { isConnected, messages, subscribe } = useSSE(
 *   'http://localhost:8080/api/v1/sse'
 * );
 * ```
 */

import { useEffect, useRef, useState, useCallback } from 'react';

/**
 * SSE message types (aligned with backend)
 */
export type SseMessageType = 
  | 'message'        // New message notification
  | 'agent_update'   // Agent status update
  | 'memory_update'  // Memory update notification
  | 'stream_chunk'   // LLM streaming response chunk
  | 'error'          // Error notification
  | 'heartbeat';     // Keep-alive heartbeat

/**
 * SSE message structure
 */
export interface SseMessage {
  type: SseMessageType;
  data?: unknown;
  timestamp?: string;
  [key: string]: unknown;
}

/**
 * SSE connection options
 */
export interface SSEOptions {
  /** Authentication token */
  token?: string;
  /** Reconnect automatically on disconnect */
  autoReconnect?: boolean;
  /** Maximum reconnection attempts (0 = infinite) */
  maxReconnectAttempts?: number;
  /** Initial reconnection delay in ms */
  reconnectDelay?: number;
  /** Maximum reconnection delay in ms */
  maxReconnectDelay?: number;
  /** Debug mode - log all events */
  debug?: boolean;
  /** Keep messages history */
  keepHistory?: boolean;
  /** Maximum messages to keep in history */
  maxHistorySize?: number;
}

/**
 * SSE connection state
 */
export interface SSEState {
  /** Whether the SSE is connected */
  isConnected: boolean;
  /** All received messages (if keepHistory is true) */
  messages: SseMessage[];
  /** Last received message */
  lastMessage: SseMessage | null;
  /** Current connection state */
  readyState: number;
  /** Number of reconnection attempts */
  reconnectAttempts: number;
  /** Whether currently attempting to reconnect */
  isReconnecting: boolean;
  /** Last error if any */
  error: Error | null;
}

/**
 * Message event handler
 */
export type SseMessageHandler = (message: SseMessage) => void;

/**
 * Default options
 */
const DEFAULT_OPTIONS: Required<SSEOptions> = {
  token: '',
  autoReconnect: true,
  maxReconnectAttempts: 5,
  reconnectDelay: 1000,
  maxReconnectDelay: 30000,
  debug: false,
  keepHistory: false,
  maxHistorySize: 100,
};

/**
 * SSE Hook
 * 
 * @param url - SSE endpoint URL
 * @param options - Connection options
 * @returns SSE state and control functions
 */
export function useSSE(
  url: string,
  options: SSEOptions = {}
) {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  
  // State
  const [state, setState] = useState<SSEState>({
    isConnected: false,
    messages: [],
    lastMessage: null,
    readyState: EventSource.CLOSED,
    reconnectAttempts: 0,
    isReconnecting: false,
    error: null,
  });

  // Refs
  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const messageHandlersRef = useRef<Map<string, Set<SseMessageHandler>>>(new Map());
  const reconnectAttemptsRef = useRef(0);

  /**
   * Log debug messages
   */
  const log = useCallback((...args: unknown[]) => {
    if (opts.debug) {
      console.log('[SSE]', ...args);
    }
  }, [opts.debug]);

  /**
   * Clear reconnection timer
   */
  const clearReconnectTimer = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, []);

  /**
   * Calculate reconnection delay with exponential backoff
   */
  const getReconnectDelay = useCallback((attempt: number): number => {
    const delay = Math.min(
      opts.reconnectDelay * Math.pow(2, attempt),
      opts.maxReconnectDelay
    );
    // Add jitter to prevent thundering herd
    return delay + Math.random() * 1000;
  }, [opts.reconnectDelay, opts.maxReconnectDelay]);

  /**
   * Add message to history
   */
  const addToHistory = useCallback((message: SseMessage) => {
    if (opts.keepHistory) {
      setState(prev => {
        const newMessages = [...prev.messages, message];
        // Keep only the last N messages
        if (newMessages.length > opts.maxHistorySize) {
          newMessages.shift();
        }
        return {
          ...prev,
          messages: newMessages,
          lastMessage: message,
        };
      });
    } else {
      setState(prev => ({
        ...prev,
        lastMessage: message,
      }));
    }
  }, [opts.keepHistory, opts.maxHistorySize]);

  /**
   * Handle incoming messages
   */
  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message: SseMessage = JSON.parse(event.data);
      log('Message received:', message.type, message);

      // Update state with message
      addToHistory(message);

      // Skip heartbeat messages for subscribers
      if (message.type === 'heartbeat') {
        log('Heartbeat received');
        return;
      }

      // Notify subscribers
      const typeHandlers = messageHandlersRef.current.get(message.type);
      const allHandlers = messageHandlersRef.current.get('*');

      if (typeHandlers) {
        typeHandlers.forEach(handler => {
          try {
            handler(message);
          } catch (error) {
            console.error('[SSE] Handler error:', error);
          }
        });
      }

      if (allHandlers) {
        allHandlers.forEach(handler => {
          try {
            handler(message);
          } catch (error) {
            console.error('[SSE] Handler error:', error);
          }
        });
      }
    } catch (error) {
      console.error('[SSE] Failed to parse message:', error);
    }
  }, [log, addToHistory]);

  /**
   * Attempt to reconnect
   */
  const reconnect = useCallback(() => {
    if (!opts.autoReconnect) {
      log('Auto-reconnect disabled');
      return;
    }

    if (opts.maxReconnectAttempts > 0 && reconnectAttemptsRef.current >= opts.maxReconnectAttempts) {
      console.error('[SSE] Max reconnection attempts reached');
      setState(prev => ({ 
        ...prev, 
        isReconnecting: false,
        error: new Error('Max reconnection attempts reached'),
      }));
      return;
    }

    setState(prev => ({ 
      ...prev, 
      isReconnecting: true,
      reconnectAttempts: reconnectAttemptsRef.current + 1,
    }));

    const delay = getReconnectDelay(reconnectAttemptsRef.current);
    log(`Reconnecting in ${delay}ms... (attempt ${reconnectAttemptsRef.current + 1})`);

    reconnectTimeoutRef.current = setTimeout(() => {
      reconnectAttemptsRef.current++;
      connect();
    }, delay);
  }, [opts.autoReconnect, opts.maxReconnectAttempts, log, getReconnectDelay]);

  /**
   * Connect to SSE
   */
  const connect = useCallback(() => {
    // Clean up existing connection
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    clearReconnectTimer();

    try {
      // Build SSE URL with token
      const sseUrl = opts.token 
        ? `${url}?token=${encodeURIComponent(opts.token)}`
        : url;

      log('Connecting to:', sseUrl.replace(/token=[^&]+/, 'token=***'));

      const eventSource = new EventSource(sseUrl);
      eventSourceRef.current = eventSource;

      // Connection opened
      eventSource.onopen = () => {
        log('Connected');
        reconnectAttemptsRef.current = 0;
        setState(prev => ({
          ...prev,
          isConnected: true,
          readyState: EventSource.OPEN,
          reconnectAttempts: 0,
          isReconnecting: false,
          error: null,
        }));
      };

      // Message received
      eventSource.onmessage = handleMessage;

      // Error occurred
      eventSource.onerror = (error) => {
        console.error('[SSE] Error:', error);
        
        const isConnected = eventSource.readyState === EventSource.OPEN;
        
        setState(prev => ({
          ...prev,
          isConnected,
          readyState: eventSource.readyState,
          error: new Error('SSE connection error'),
        }));

        // If connection is closed, attempt reconnection
        if (eventSource.readyState === EventSource.CLOSED) {
          log('Connection closed');
          eventSource.close();
          reconnect();
        }
      };
    } catch (error) {
      console.error('[SSE] Failed to connect:', error);
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error : new Error('Failed to connect'),
      }));
      reconnect();
    }
  }, [url, opts.token, log, handleMessage, reconnect, clearReconnectTimer]);

  /**
   * Disconnect from SSE
   */
  const disconnect = useCallback(() => {
    log('Disconnecting...');
    clearReconnectTimer();
    
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    setState(prev => ({
      ...prev,
      isConnected: false,
      readyState: EventSource.CLOSED,
      reconnectAttempts: 0,
      isReconnecting: false,
    }));
  }, [log, clearReconnectTimer]);

  /**
   * Subscribe to messages of a specific type
   */
  const subscribe = useCallback((
    messageType: SseMessageType | '*',
    handler: SseMessageHandler
  ): (() => void) => {
    if (!messageHandlersRef.current.has(messageType)) {
      messageHandlersRef.current.set(messageType, new Set());
    }
    messageHandlersRef.current.get(messageType)!.add(handler);
    log('Subscribed to:', messageType);

    // Return unsubscribe function
    return () => {
      const handlers = messageHandlersRef.current.get(messageType);
      if (handlers) {
        handlers.delete(handler);
        if (handlers.size === 0) {
          messageHandlersRef.current.delete(messageType);
        }
      }
      log('Unsubscribed from:', messageType);
    };
  }, [log]);

  /**
   * Clear message history
   */
  const clearHistory = useCallback(() => {
    setState(prev => ({
      ...prev,
      messages: [],
    }));
    log('History cleared');
  }, [log]);

  /**
   * Connect on mount, disconnect on unmount
   */
  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    // State
    ...state,
    
    // Control functions
    connect,
    disconnect,
    subscribe,
    clearHistory,
    
    // Utility
    isOpen: state.readyState === EventSource.OPEN,
    isClosed: state.readyState === EventSource.CLOSED,
  };
}

/**
 * Hook for subscribing to specific SSE message types
 * 
 * @param sse - SSE hook result
 * @param messageType - Message type to subscribe to
 * @param handler - Message handler function
 */
export function useSSESubscription(
  sse: ReturnType<typeof useSSE>,
  messageType: SseMessageType | '*',
  handler: SseMessageHandler
) {
  useEffect(() => {
    const unsubscribe = sse.subscribe(messageType, handler);
    return unsubscribe;
  }, [sse, messageType, handler]);
}

/**
 * Hook for SSE streaming (e.g., LLM responses)
 * 
 * @param url - SSE endpoint URL
 * @param options - Connection options
 * @returns Streaming chunks and control functions
 */
export function useSSEStream(
  url: string,
  options: SSEOptions = {}
) {
  const [chunks, setChunks] = useState<string[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamComplete, setStreamComplete] = useState(false);

  const sse = useSSE(url, {
    ...options,
    keepHistory: false, // Don't keep full history for streaming
  });

  // Subscribe to stream chunks
  useEffect(() => {
    const unsubscribe = sse.subscribe('stream_chunk', (message) => {
      setIsStreaming(true);
      setStreamComplete(false);
      
      if (message.data && typeof message.data === 'object' && 'chunk' in message.data) {
        const chunk = (message.data as { chunk: string }).chunk;
        setChunks(prev => [...prev, chunk]);
      }
    });

    return unsubscribe;
  }, [sse]);

  // Detect stream completion
  useEffect(() => {
    if (sse.lastMessage?.type === 'stream_chunk' && 
        sse.lastMessage?.data && 
        typeof sse.lastMessage.data === 'object' &&
        'done' in sse.lastMessage.data &&
        sse.lastMessage.data.done) {
      setIsStreaming(false);
      setStreamComplete(true);
    }
  }, [sse.lastMessage]);

  const clearChunks = useCallback(() => {
    setChunks([]);
    setIsStreaming(false);
    setStreamComplete(false);
  }, []);

  return {
    ...sse,
    chunks,
    fullText: chunks.join(''),
    isStreaming,
    streamComplete,
    clearChunks,
  };
}

