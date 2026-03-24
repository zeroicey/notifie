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

# Type check
bun run build  # runs tsc && vite build
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
│   ├── notifie-client/       # Tauri desktop app
│   │   ├── src/              # React frontend
│   │   └── src-tauri/        # Rust backend
│   └── notifie-server/       # Go server
│       ├── main.go           # Entry point
│       ├── handler/          # HTTP handlers
│       ├── hub/              # WebSocket hub
│       └── msg/              # Message types
└── docs/
    └── design.md             # Design specification
```

## API Reference

### HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | / | Health check |
| GET | /health | Returns client count |
| POST | /api/notify | Broadcast notification to all clients |
| GET | /ws | WebSocket endpoint |

### WebSocket Message Format

```json
{
  "type": "notify",
  "title": "Alert",
  "content": "CPU > 90%",
  "timestamp": 1700000000
}
```

## Development Notes

- Use worktrees for feature development: `git worktree add .worktrees/<name> -b feature/<name>`
- Server already implemented and tested
- Client is work-in-progress (see implementation plan in docs/superpowers/plans/)
- Tauri plugins needed: notification, websocket, tray-icon