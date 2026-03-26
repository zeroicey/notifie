import { useState, useEffect, useRef, useCallback } from 'react';
import WebSocket from '@tauri-apps/plugin-websocket';

export interface NotifyMessage {
  type: string;
  title: string;
  content: string;
  timestamp: number;
}

export function useWebSocket(serverUrl: string, onMessage: (msg: NotifyMessage) => void) {
  const [connected, setConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);

  const connect = useCallback(async () => {
    console.log('[WebSocket] Connecting to:', serverUrl);

    if (wsRef.current) {
      try {
        await wsRef.current.disconnect();
      } catch (e) {}
    }

    try {
      console.log('[WebSocket] Attempting to connect...');
      const ws = await WebSocket.connect(serverUrl);
      console.log('[WebSocket] Connected successfully!');
      wsRef.current = ws;
      setConnected(true);

      ws.addListener((msg) => {
        console.log('[WebSocket] Received message:', msg);
        try {
          // Tauri websocket plugin returns { data: string, type: string }
          const messageData = typeof msg === 'string' ? msg : (msg as { data: string }).data;
          const data = JSON.parse(messageData) as NotifyMessage;
          console.log('[WebSocket] Parsed data:', data);
          onMessage(data);
        } catch (e) {
          console.error('[WebSocket] Failed to parse message:', e);
        }
      });
    } catch (e) {
      console.error('[WebSocket] Connection failed:', e);
      setConnected(false);
    }
  }, [serverUrl, onMessage]);

  const disconnect = useCallback(async () => {
    if (wsRef.current) {
      try {
        await wsRef.current.disconnect();
      } catch (e) {}
      wsRef.current = null;
      setConnected(false);
    }
  }, []);

  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  return { connected, connect, disconnect };
}