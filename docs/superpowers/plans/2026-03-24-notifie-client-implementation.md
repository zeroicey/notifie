# Notifie Client Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Tauri + React 客户端，支持 WebSocket 接收通知、系统托盘、设置页面

**Architecture:**
- 使用 Tauri 2.x + React 19
- 使用 @tauri-apps/plugin-notification 发送系统通知
- 使用 @tauri-apps/plugin-websocket 连接服务端
- 使用 Tauri tray-icon 实现系统托盘

**Tech Stack:**
- Tauri 2.x
- React 19
- Bun
- @tauri-apps/plugin-notification
- @tauri-apps/plugin-websocket

---

## Chunk 1: 项目配置

### Task 1: 添加 Tauri 插件 (notification, websocket)

**Files:**
- Modify: `apps/notifie-client/src-tauri/Cargo.toml`
- Modify: `apps/notifie-client/package.json`

- [ ] **Step 1: 添加 Rust 插件依赖**

修改 `apps/notifie-client/src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
tauri-plugin-notification = "2"
tauri-plugin-websocket = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: 添加 JavaScript 插件依赖**

修改 `apps/notifie-client/package.json`:

```json
{
  "dependencies": {
    "react": "^19.1.0",
    "react-dom": "^19.1.0",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "@tauri-apps/plugin-notification": "^2",
    "@tauri-apps/plugin-websocket": "^2"
  }
}
```

- [ ] **Step 3: 验证编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-client && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: 无错误

- [ ] **Step 4: Commit**

```bash
git add apps/notifie-client/src-tauri/Cargo.toml apps/notifie-client/package.json
git commit -m "feat(client): add notification and websocket plugins"
```

---

### Task 2: 配置 tauri.conf.json

**Files:**
- Modify: `apps/notifie-client/src-tauri/tauri.conf.json`

- [ ] **Step 1: 配置托盘和安全策略**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "notifie",
  "version": "0.1.0",
  "identifier": "icu.oicey.notifie",
  "build": {
    "devtools": true,
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "notifie",
        "width": 400,
        "height": 300,
        "center": true,
        "resizable": true,
        "visible": false,
        "decorations": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; connect-src 'self' ws: wss: http: https:; img-src 'self' data:; style-src 'self' 'unsafe-inline'"
    },
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "notification": {
      "all": true
    }
  }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/notifie-client/src-tauri/tauri.conf.json
git commit -m "feat(client): configure tauri with tray and plugins"
```

---

## Chunk 2: Rust 后端

### Task 3: 实现 Rust 后端 (lib.rs)

**Files:**
- Modify: `apps/notifie-client/src-tauri/src/lib.rs`

- [ ] **Step 1: 实现托盘和插件初始化**

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_websocket::init())
        .setup(|app| {
            // 创建托盘菜单
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("notifie - connected")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: 验证编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-client && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: 无错误

- [ ] **Step 3: Commit**

```bash
git add apps/notifie-client/src-tauri/src/lib.rs
git commit -m "feat(client): add tray and plugins initialization"
```

---

## Chunk 3: React 前端

### Task 4: 实现 WebSocket 和通知逻辑

**Files:**
- Create: `apps/notifie-client/src/hooks/useWebSocket.ts`
- Create: `apps/notifie-client/src/hooks/useNotification.ts`
- Create: `apps/notifie-client/src/components/Settings.tsx`
- Modify: `apps/notifie-client/src/App.tsx`

- [ ] **Step 1: 创建 WebSocket Hook**

```typescript
// apps/notifie-client/src/hooks/useWebSocket.ts
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
      await wsRef.current.disconnect();
    }

    try {
      const ws = await WebSocket.connect(serverUrl);
      wsRef.current = ws;
      setConnected(true);

      ws.addListener((msg) => {
        try {
          const data = JSON.parse(msg) as NotifyMessage;
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
      await wsRef.current.disconnect();
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
```

- [ ] **Step 2: 创建通知 Hook**

```typescript
// apps/notifie-client/src/hooks/useNotification.ts
import { useCallback } from 'react';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

export function useNotification() {
  const notify = useCallback(async (title: string, body: string) => {
    let permitted = await isPermissionGranted();
    if (!permitted) {
      const permission = await requestPermission();
      permitted = permission === 'granted';
    }

    if (permitted) {
      sendNotification({ title, body });
    }
  }, []);

  return { notify };
}
```

- [ ] **Step 3: 创建设置页面组件**

```tsx
// apps/notifie-client/src/components/Settings.tsx
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

const styles = {
  overlay: {
    position: 'fixed' as const,
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
    boxSizing: 'border-box' as const,
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
```

- [ ] **Step 4: 修改 App.tsx**

```tsx
// apps/notifie-client/src/App.tsx
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
```

- [ ] **Step 5: 验证编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-client && bun run build`
Expected: 无错误

- [ ] **Step 6: Commit**

```bash
git add apps/notifie-client/src/
git commit -m "feat(client): add WebSocket and notification frontend"
```

---

## Chunk 4: Capabilities 配置

### Task 5: 配置插件权限

**Files:**
- Modify: `apps/notifie-client/src-tauri/capabilities/default.json`

- [ ] **Step 1: 配置权限**

```json
{
  "$schema": "https://schema.tauri.app/config/2/capability",
  "identifier": "default",
  "description": "Default permissions for notifie",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "notification:default",
    "notification:allow-is-permission-granted",
    "notification:allow-request-permission",
    "notification:allow-notify",
    "websocket:default",
    "websocket:allow-connect",
    "websocket:allow-disconnect",
    "websocket:allow-send",
    "websocket:allow-add-listener"
  ]
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/notifie-client/src-tauri/capabilities/default.json
git commit -m "feat(client): add plugin capabilities"
```

---

## 完成

**Summary:**
- Task 1: 添加 Tauri 插件依赖
- Task 2: 配置 tauri.conf.json
- Task 3: 实现 Rust 后端（托盘、插件初始化）
- Task 4: 实现 React 前端（WebSocket、通知、设置页面）
- Task 5: 配置插件权限

**下一步:**
- 在本地测试客户端功能