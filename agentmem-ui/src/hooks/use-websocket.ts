/**
 * WebSocket Hook for Real-time Communication
 * 
 * Provides a React hook for WebSocket connections with:
 * - Automatic reconnection with exponential backoff
 * - Connection state management
 * - Message type handling
 * - Token-based authentication
 * - Event subscription system
 * 
 * Usage:
 * ```tsx
 * const { isConnected, lastMessage, sendMessage, subscribe } = useWebSocket(
 *   'ws://localhost:8080/api/v1/ws'
 * );
 * ```
 */

import { useEffect, useRef, useState, useCallback } from 'react';

/**
 * WebSocket message types (aligned with backend)
 */
export type WsMessageType = 
  | 'message'        // New message notification
  | 'agent_update'   // Agent status update
  | 'memory_update'  // Memory update notification
  | 'error'          // Error notification
  | 'ping'           // Heartbeat ping
  | 'pong';          // Heartbeat pong

/**
 * WebSocket message structure
 */
export interface WsMessage {
  type: WsMessageType;
  data?: unknown;
  timestamp?: string;
  [key: string]: unknown;
}

/**
 * WebSocket connection options
 */
export interface WebSocketOptions {
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
  /** Heartbeat interval in ms (0 = disabled) */
  heartbeatInterval?: number;
  /** Debug mode - log all events */
  debug?: boolean;
}

/**
 * WebSocket connection state
 */
export interface WebSocketState {
  /** Whether the WebSocket is connected */
  isConnected: boolean;
  /** Last received message */
  lastMessage: WsMessage | null;
  /** Current connection state */
  readyState: number;
  /** Number of reconnection attempts */
  reconnectAttempts: number;
  /** Whether currently attempting to reconnect */
  isReconnecting: boolean;
}

/**
 * Message event handler
 */
export type MessageHandler = (message: WsMessage) => void;

/**
 * Default options
 */
const DEFAULT_OPTIONS: Required<WebSocketOptions> = {
  token: '',
  autoReconnect: true,
  maxReconnectAttempts: 5,
  reconnectDelay: 1000,
  maxReconnectDelay: 30000,
  heartbeatInterval: 30000,
  debug: false,
};

/**
 * WebSocket Hook
 * 
 * @param url - WebSocket URL
 * @param options - Connection options
 * @returns WebSocket state and control functions
 */
export function useWebSocket(
  url: string,
  options: WebSocketOptions = {}
) {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  
  // State
  const [state, setState] = useState<WebSocketState>({
    isConnected: false,
    lastMessage: null,
    readyState: WebSocket.CLOSED,
    reconnectAttempts: 0,
    isReconnecting: false,
  });

  // Refs
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const heartbeatIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const messageHandlersRef = useRef<Map<string, Set<MessageHandler>>>(new Map());
  const reconnectAttemptsRef = useRef(0);

  /**
   * Log debug messages
   */
  const log = useCallback((...args: unknown[]) => {
    if (opts.debug) {
      console.log('[WebSocket]', ...args);
    }
  }, [opts.debug]);

  /**
   * Clear all timers
   */
  const clearTimers = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
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
   * Send heartbeat ping
   */
  const sendHeartbeat = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const pingMessage: WsMessage = {
        type: 'ping',
        timestamp: new Date().toISOString(),
      };
      wsRef.current.send(JSON.stringify(pingMessage));
      log('Heartbeat ping sent');
    }
  }, [log]);

  /**
   * Start heartbeat interval
   */
  const startHeartbeat = useCallback(() => {
    if (opts.heartbeatInterval > 0) {
      heartbeatIntervalRef.current = setInterval(sendHeartbeat, opts.heartbeatInterval);
      log('Heartbeat started:', opts.heartbeatInterval, 'ms');
    }
  }, [opts.heartbeatInterval, sendHeartbeat, log]);

  /**
   * Stop heartbeat interval
   */
  const stopHeartbeat = useCallback(() => {
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
      log('Heartbeat stopped');
    }
  }, [log]);

  /**
   * Handle incoming messages
   */
  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message: WsMessage = JSON.parse(event.data);
      log('Message received:', message.type, message);

      // Update state with last message
      setState(prev => ({
        ...prev,
        lastMessage: message,
      }));

      // Handle pong response
      if (message.type === 'pong') {
        log('Heartbeat pong received');
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
            console.error('[WebSocket] Handler error:', error);
          }
        });
      }

      if (allHandlers) {
        allHandlers.forEach(handler => {
          try {
            handler(message);
          } catch (error) {
            console.error('[WebSocket] Handler error:', error);
          }
        });
      }
    } catch (error) {
      console.error('[WebSocket] Failed to parse message:', error);
    }
  }, [log]);

  /**
   * Attempt to reconnect
   */
  const reconnect = useCallback(() => {
    if (!opts.autoReconnect) {
      log('Auto-reconnect disabled');
      return;
    }

    if (opts.maxReconnectAttempts > 0 && reconnectAttemptsRef.current >= opts.maxReconnectAttempts) {
      console.error('[WebSocket] Max reconnection attempts reached');
      setState(prev => ({ ...prev, isReconnecting: false }));
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
  }, [opts.autoReconnect, opts.maxReconnectAttempts, log]);

  /**
   * Connect to WebSocket
   */
  const connect = useCallback(() => {
    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    clearTimers();

    try {
      // Build WebSocket URL with token
      const wsUrl = opts.token 
        ? `${url}?token=${encodeURIComponent(opts.token)}`
        : url;

      log('Connecting to:', wsUrl.replace(/token=[^&]+/, 'token=***'));

      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      // Connection opened
      ws.onopen = () => {
        log('Connected');
        reconnectAttemptsRef.current = 0;
        setState({
          isConnected: true,
          lastMessage: null,
          readyState: WebSocket.OPEN,
          reconnectAttempts: 0,
          isReconnecting: false,
        });
        startHeartbeat();
      };

      // Message received
      ws.onmessage = handleMessage;

      // Connection closed
      ws.onclose = (event) => {
        log('Disconnected:', event.code, event.reason);
        stopHeartbeat();
        setState(prev => ({
          ...prev,
          isConnected: false,
          readyState: WebSocket.CLOSED,
        }));

        // Attempt reconnection if not a normal closure
        if (event.code !== 1000 && event.code !== 1001) {
          reconnect();
        }
      };

      // Error occurred
      ws.onerror = (error) => {
        console.error('[WebSocket] Error:', error);
        setState(prev => ({
          ...prev,
          isConnected: false,
          readyState: ws.readyState,
        }));
      };
    } catch (error) {
      console.error('[WebSocket] Failed to connect:', error);
      reconnect();
    }
  }, [url, opts.token, log, handleMessage, startHeartbeat, stopHeartbeat, reconnect, clearTimers]);

  /**
   * Disconnect from WebSocket
   */
  const disconnect = useCallback(() => {
    log('Disconnecting...');
    clearTimers();
    
    if (wsRef.current) {
      // Prevent reconnection on manual disconnect
      wsRef.current.onclose = null;
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setState({
      isConnected: false,
      lastMessage: null,
      readyState: WebSocket.CLOSED,
      reconnectAttempts: 0,
      isReconnecting: false,
    });
  }, [log, clearTimers]);

  /**
   * Send a message
   */
  const sendMessage = useCallback((message: WsMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
      log('Message sent:', message.type, message);
      return true;
    } else {
      console.warn('[WebSocket] Cannot send message: not connected');
      return false;
    }
  }, [log]);

  /**
   * Subscribe to messages of a specific type
   */
  const subscribe = useCallback((
    messageType: WsMessageType | '*',
    handler: MessageHandler
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
    sendMessage,
    subscribe,
    
    // Utility
    isOpen: state.readyState === WebSocket.OPEN,
    isClosed: state.readyState === WebSocket.CLOSED,
  };
}

/**
 * Hook for subscribing to specific message types
 * 
 * @param ws - WebSocket hook result
 * @param messageType - Message type to subscribe to
 * @param handler - Message handler function
 */
export function useWebSocketSubscription(
  ws: ReturnType<typeof useWebSocket>,
  messageType: WsMessageType | '*',
  handler: MessageHandler
) {
  useEffect(() => {
    const unsubscribe = ws.subscribe(messageType, handler);
    return unsubscribe;
  }, [ws, messageType, handler]);
}

