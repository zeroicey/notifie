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
    if (wsRef.current) {
      try {
        await wsRef.current.disconnect();
      } catch (e) {}
    }

    try {
      const ws = await WebSocket.connect(serverUrl);
      wsRef.current = ws;
      setConnected(true);

      ws.addListener((msg) => {
        try {
          const data = JSON.parse(msg as string) as NotifyMessage;
          onMessage(data);
        } catch (e) {
          console.error('Failed to parse message:', e);
        }
      });
    } catch (e) {
      console.error('WebSocket connection failed:', e);
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