# Notifie

A lightweight cross-platform notification system for desktop (Windows, macOS, Linux). Push real-time notifications from a server to all connected clients via WebSocket.

## Features

- Real-time notifications via WebSocket
- Cross-platform desktop client (Windows, macOS, Linux)
- System tray integration
- Configurable server address
- Native system notifications

## Architecture

```
┌────────────────────┐    WebSocket    ┌────────────────────┐
│  notifie-client    │◄────────────────►|  notifie-server   │
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
| Communication | WebSocket |

## Quick Start

### Prerequisites

- [Bun](https://bun.sh) (for client)
- [Go](https://go.dev) (for server)
- [Rust](https://rustup.rs) (for Tauri)

### Run Server

```bash
cd apps/notifie-server
go run main.go
```

The server listens on `http://localhost:8080` by default.

### Run Client

```bash
cd apps/notifie-client
bun install
bun run tauri dev
```

## API Reference

### HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | / | Health check |
| GET | /health | Returns client count |
| POST | /api/notify | Broadcast notification to all clients |
| GET | /ws | WebSocket endpoint |

### Send a Notification

```bash
curl -X POST http://localhost:8080/api/notify \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","content":"Hello World"}'
```

### WebSocket Message Format

```json
{
  "type": "notify",
  "title": "Alert",
  "content": "CPU > 90%",
  "timestamp": 1700000000
}
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

## Build

### Client

```bash
cd apps/notifie-client
bun run tauri build
```

### Server

```bash
cd apps/notifie-server
go build -o notifie-server main.go
```

## License

MIT