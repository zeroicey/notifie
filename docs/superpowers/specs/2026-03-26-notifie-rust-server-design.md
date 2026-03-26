# Notifie Server Rust 实现设计

> **目标**: 用 Rust + Axum 重写 Go 服务器，完全兼容现有 API

## 架构

```
┌─────────────────┐    WebSocket    ┌─────────────────┐
│  notifie-client │◄────────────────►│  notifie-server │
│    (Tauri)      │                  │  (Axum + tokio) │
└─────────────────┘                  └────────┬────────┘
                                               │
                                          POST /api/notify
                                               │
                                          ┌────▼─────┐
                                          │  Hub    │
                                          │(broadcast)│
                                          └──────────┘
```

## API 端点 (完全兼容)

| Method | Path | 响应 |
|--------|------|------|
| GET | / | "Notifie Server Running" |
| GET | /health | `{"status":"ok","clients":N}` |
| POST | /api/notify | `{"success":true,"message":"Notification sent","count":N}` |
| GET | /ws | WebSocket 升级 |

## 项目结构

```
apps/notifie-server/
├── Cargo.toml
└── src/
    ├── main.rs         # 入口，Axum app 搭建
    ├── handler/
    │   └── notify.rs   # HTTP handlers
    ├── hub/
    │   └── client.rs   # WebSocket hub (broadcast)
    └── model/
        └── message.rs  # 消息结构体
```

## 依赖

```toml
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "log"] }
```

## WebSocket 广播实现

使用 `tokio::sync::broadcast` channel:
- 所有连接订阅同一个 broadcast channel
- 当收到 POST /api/notify 时，发送消息到 broadcast sender
- 每个连接接收消息并发送到 WebSocket

## 配置

- 监听地址: `:8080` (可通过命令行参数修改)
- CORS: 允许所有来源
- 日志: tower-http log middleware