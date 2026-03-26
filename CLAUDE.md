# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Notifie is a lightweight cross-platform notification system for desktop (Windows, macOS, Linux). It pushes real-time notifications from a server to all connected clients via WebSocket.

## Architecture

```
┌────────────────────┐    WebSocket    ┌────────────────────┐
│  notifie-client    │◄────────────────►│  notifie-server   │
│  (Tauri + React)   │                 │  (Go + Fiber)     │
└────────────────────┘                 └────────────────────┘
         │                                       ▲
         │ Notification                          │
         ▼                                       │
┌────────────────────┐                            │
│   System Native   │                            │
│   Notification    │                            │
└────────────────────┘                            │
                                    POST /api/notify
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Client Framework | Tauri 2.x |
| Frontend Runtime | Bun |
| Frontend | React 19 |
| Server | Go + Fiber |
| WebSocket | gorilla/websocket |

## Common Commands

### Client (Tauri + React + Bun)

```bash
cd apps/notifie-client

# Install dependencies
bun install

# Development
bun run tauri dev

# Build
bun run tauri build

# Type check only
bun run build  # runs tsc && vite build

# Verify Rust compiles
cargo check --manifest-path src-tauri/Cargo.toml
```

### Server (Go + Fiber)

```bash
cd apps/notifie-server

# Install dependencies
go mod download

# Run
go run main.go
# Listens on :8080 by default

# Build
go build -o notifie-server main.go
```

### Testing the System

```bash
# Start server
go run main.go &

# Send notification
curl -X POST http://localhost:8080/api/notify \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","content":"Hello"}'

# Check health
curl http://localhost:8080/health
```

## Project Structure

```
notifie/
├── apps/
│   ├── notifie-client/           # Tauri desktop app
│   │   ├── src/                  # React frontend
│   │   │   ├── App.tsx           # Main app component
│   │   │   ├── hooks/            # Custom hooks (useWebSocket, useNotification)
│   │   │   └── components/       # UI components
│   │   ├── src-tauri/            # Rust backend
│   │   │   ├── Cargo.toml        # Rust dependencies (tauri, plugins)
│   │   │   ├── tauri.conf.json   # Tauri configuration
│   │   │   ├── capabilities/     # Plugin permissions
│   │   │   └── src/lib.rs        # Rust entry with tray setup
│   │   └── package.json          # JS dependencies
│   └── notifie-server/           # Go server
│       ├── main.go               # Entry point, Fiber app setup
│       ├── handler/              # HTTP handlers (notify.go)
│       ├── hub/                  # WebSocket hub (client.go)
│       └── msg/                  # Message types
└── docs/
    ├── design.md                  # Design specification
    └── superpowers/               # Implementation plans
```

## API Reference

### HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | / | Health check - returns "Notifie Server Running" |
| GET | /health | Returns JSON with client count |
| POST | /api/notify | Broadcast notification to all connected clients |
| GET | /ws | WebSocket endpoint (note: currently disabled in main.go) |

### WebSocket Message Format

```json
{
  "type": "notify",
  "title": "Alert",
  "content": "CPU > 90%",
  "timestamp": 1700000000
}
```

## Client Configuration

### Tauri Plugins Required

- `tauri-plugin-notification` - System native notifications
- `tauri-plugin-websocket` - WebSocket connections to server
- `tauri-plugin-opener` - Open external links

### Capabilities (src-tauri/capabilities/default.json)

The client needs these permissions for plugins:
- `notification:default`, `notification:allow-is-permission-granted`, `notification:allow-request-permission`, `notification:allow-notify`
- `websocket:default`, `websocket:allow-connect`, `websocket:allow-disconnect`, `websocket:allow-send`, `websocket:allow-add-listener`

### Tray Icon

The Rust backend sets up system tray with:
- Left-click: Show window
- Menu items: "Show" and "Quit"

## Development Notes

- Use worktrees for feature development: `git worktree add .worktrees/<name> -b feature/<name>`
- Server implementation is stable; client is in-progress
- Client stores server URL in localStorage (`notifie-server-url` key)
- Default WebSocket URL: `ws://localhost:8080/ws`
- Go module path: `github.com/zeroicey/notifie`