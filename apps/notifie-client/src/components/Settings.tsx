import { useState, useEffect } from 'react';

interface SettingsProps {
  serverUrl: string;
  onSave: (url: string) => void;
  onClose: () => void;
}

export function Settings({ serverUrl: initialUrl, onSave, onClose }: SettingsProps) {
  const [serverUrl, setServerUrl] = useState(initialUrl);

  useEffect(() => {
    const saved = localStorage.getItem('notifie-server-url');
    if (saved) setServerUrl(saved);
  }, []);

  const handleSave = () => {
    localStorage.setItem('notifie-server-url', serverUrl);
    onSave(serverUrl);
    onClose();
  };

  return (
    <div style={styles.overlay}>
      <div style={styles.modal}>
        <h2>Settings</h2>
        <div style={styles.field}>
          <label>Server URL:</label>
          <input
            type="text"
            value={serverUrl}
            onChange={(e) => setServerUrl(e.target.value)}
            placeholder="ws://localhost:8080/ws"
            style={styles.input}
          />
        </div>
        <div style={styles.buttons}>
          <button onClick={handleSave} style={styles.saveBtn}>Save</button>
          <button onClick={onClose} style={styles.cancelBtn}>Cancel</button>
        </div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0,0,0,0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  modal: {
    background: 'white',
    padding: '20px',
    borderRadius: '8px',
    width: '300px',
  },
  field: {
    marginBottom: '15px',
  },
  input: {
    width: '100%',
    padding: '8px',
    marginTop: '5px',
    boxSizing: 'border-box',
  },
  buttons: {
    display: 'flex',
    gap: '10px',
    justifyContent: 'flex-end',
  },
  saveBtn: {
    padding: '8px 16px',
    background: '#4CAF50',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  cancelBtn: {
    padding: '8px 16px',
    background: '#ccc',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
};