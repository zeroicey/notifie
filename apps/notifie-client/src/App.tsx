import { useState, useCallback, useEffect } from 'react';
import { useWebSocket, NotifyMessage } from './hooks/useWebSocket';
import { useNotification } from './hooks/useNotification';
import { Settings } from './components/Settings';

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [serverUrl, setServerUrl] = useState(() => {
    return localStorage.getItem('notifie-server-url') || 'ws://localhost:8080/ws';
  });
  const { notify } = useNotification();

  const handleMessage = useCallback((msg: NotifyMessage) => {
    notify(msg.title, msg.content);
  }, [notify]);

  const { connected, connect, disconnect } = useWebSocket(serverUrl, handleMessage);

  useEffect(() => {
    if (serverUrl) {
      connect();
    }
    return () => {
      disconnect();
    };
  }, [serverUrl, connect, disconnect]);

  const handleServerChange = (newUrl: string) => {
    setServerUrl(newUrl);
    disconnect();
    setTimeout(() => connect(), 100);
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui, sans-serif' }}>
      <h1>notifie</h1>
      <div style={{ marginBottom: '20px' }}>
        Status:{' '}
        <span style={{ color: connected ? 'green' : 'red' }}>
          {connected ? 'Connected' : 'Disconnected'}
        </span>
      </div>
      <button onClick={() => setShowSettings(true)}>Settings</button>

      {showSettings && (
        <Settings
          serverUrl={serverUrl}
          onSave={handleServerChange}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}

export default App;