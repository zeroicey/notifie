# Notifie Server Rust Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用 Rust + Axum 重写 Go 服务器，完全兼容现有 API

**Architecture:** 使用 Axum 构建 HTTP + WebSocket 服务器，用 tokio::sync::broadcast 实现消息广播

**Tech Stack:** Axum 0.8, tokio (full), serde, tower-http

**Additional Dependencies (需要添加到 Cargo.toml):**
```toml
chrono = "0.4"      # 时间戳
uuid = { version = "1", features = ["v4"] }  # 生成客户端 ID
futures-util = "0.3" # WebSocket 处理
```

---

## File Structure

```
apps/notifie-server/
├── Cargo.toml              # 已存在，已添加依赖
└── src/
    ├── main.rs             # 创建: 入口，Axum app 搭建
    ├── handler/
    │   ├── mod.rs          # 创建: handler 模块导出
    │   └── notify.rs       # 创建: HTTP handlers
    ├── hub/
    │   ├── mod.rs          # 创建: hub 模块导出
    │   └── client.rs       # 创建: WebSocket hub (broadcast)
    └── model/
        ├── mod.rs          # 创建: model 模块导出
        └── message.rs      # 创建: 消息结构体
```

---

## Chunk 1: 基础结构

### Task 1: 创建 model/message.rs (消息结构体)

**Files:**
- Create: `apps/notifie-server/src/model/message.rs`

- [ ] **Step 1: 写测试**

创建 `apps/notifie-server/src/model/message.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_message_serialization() {
        let msg = NotifyMessage::new("Test Title".to_string(), "Test Content".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Test Title"));
        assert!(json.contains("Test Content"));
    }

    #[test]
    fn test_notify_request_deserialization() {
        let json = r#"{"title":"Alert","content":"Hello"}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.title, "Alert");
        assert_eq!(req.content, "Hello");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo test --lib model`
Expected: FAIL - module not found

- [ ] **Step 3: 写实现代码**

```rust
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyMessage {
    pub msg_type: String,
    pub title: String,
    pub content: String,
    pub timestamp: i64,
}

impl NotifyMessage {
    pub fn new(title: String, content: String) -> Self {
        Self {
            msg_type: "notify".to_string(),
            title,
            content,
            timestamp: Uti::now().timestamp(), // Intentional typo - fix in Step 3
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyResponse {
    pub success: bool,
    pub message: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub clients: usize,
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo test --lib model`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
cd /home/oicey/projects/notifie
git add apps/notifie-server/src/model/
git commit -m "feat(server): add message models"
```

---

### Task 2: 创建 model/mod.rs

**Files:**
- Create: `apps/notifie-server/src/model/mod.rs`

- [ ] **Step 1: 写模块导出**

```rust
pub mod message;

pub use message::*;
```

- [ ] **Step 2: 测试编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo build --lib`
Expected: OK (忽略测试失败，需要先修复 Uti -> Utc)

- [ ] **Step 3: 修复 typo 并提交**

Fix: `Utc` 拼写错误，然后提交

```bash
git add apps/notifie-server/src/model/
git commit -m "feat(server): add model module"
```

---

## Chunk 2: WebSocket Hub

### Task 3: 创建 hub/client.rs (WebSocket Hub)

**Files:**
- Create: `apps/notifie-server/src/hub/client.rs`

- [ ] **Step 1: 写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hub_new() {
        let hub = Hub::new();
        assert_eq!(hub.client_count(), 0);
    }

    #[tokio::test]
    async fn test_hub_subscribe() {
        let hub = Hub::new();
        let rx = hub.subscribe();
        // 发送消息到 broadcast
        let _ = hub.broadcast("test".to_string());
        // 应该能收到消息
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, "test");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo test --lib hub`
Expected: FAIL - module not found

- [ ] **Step 3: 写实现代码**

```rust
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use futures_util::{SinkExt, StreamExt};
use axum::extract::ws::WebSocket;

pub type ClientId = String;

pub struct Hub {
    clients: Arc<RwLock<HashMap<ClientId, broadcast::Sender<String>>>>,
    broadcast_tx: broadcast::Sender<String>,
}

impl Hub {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    pub async fn add_client(&self, client_id: ClientId) -> broadcast::Receiver<String> {
        let tx = self.broadcast_tx.clone();
        let mut clients = self.clients.write().await;
        clients.insert(client_id, tx.clone());
        tx.subscribe()
    }

    pub async fn remove_client(&self, client_id: &ClientId) {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
    }

    pub async fn client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }

    pub fn broadcast(&self, message: String) {
        let _ = self.broadcast_tx.send(message);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.broadcast_tx.subscribe()
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo test --lib hub`
Expected: PASS

- [ ] **Step 5: 添加依赖并提交**

需要在 Cargo.toml 添加 `futures-util` 依赖，然后提交

```bash
git add apps/notifie-server/src/hub/ apps/notifie-server/Cargo.toml
git commit -m "feat(server): add WebSocket hub with broadcast"
```

---

### Task 4: 创建 hub/mod.rs

**Files:**
- Create: `apps/notifie-server/src/hub/mod.rs`

- [ ] **Step 1: 写模块**

```rust
pub mod client;

pub use client::Hub;
```

- [ ] **Step 2: 提交**

```bash
git add apps/notifie-server/src/hub/mod.rs
git commit -m "feat(server): add hub module"
```

---

## Chunk 3: HTTP Handlers

### Task 5: 创建 handler/notify.rs

**Files:**
- Create: `apps/notifie-server/src/handler/notify.rs`

- [ ] **Step 1: 写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hub::Hub;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_handler() {
        let hub = Hub::new();
        let handler = NotifyHandler::new(hub);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = handler.handle_health(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo test --lib handler`
Expected: FAIL - module not found

- [ ] **Step 3: 写实现代码**

```rust
use crate::hub::Hub;
use crate::model::{HealthResponse, NotifyMessage, NotifyRequest, NotifyResponse};
use axum::{
    body::Body,
    extract::{State, WebSocketUpgrade},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

pub struct NotifyHandler {
    hub: Arc<Hub>,
}

impl NotifyHandler {
    pub fn new(hub: Arc<Hub>) -> Self {
        Self { hub }
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(root))
            .route("/health", get(health))
            .route("/api/notify", post(notify))
            .route("/ws", get(ws_handler))
            .with_state(self.hub.clone())
    }

    pub async fn handle_health(&self, req: Request<Body>) -> Response<Body> {
        health(State(self.hub.clone()), req).await.into_response()
    }

    pub async fn handle_notify(&self, req: Request<Body>) -> Response<Body> {
        notify(State(self.hub.clone()), req).await.into_response()
    }
}

async fn root() -> &'static str {
    "Notifie Server Running"
}

async fn health(State(hub): State<Arc<Hub>>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        clients: hub.client_count().await,
    })
}

async fn notify(
    State(hub): State<Arc<Hub>>,
    Json(req): Json<NotifyRequest>,
) -> impl IntoResponse {
    if req.title.is_empty() || req.content.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(NotifyResponse {
                success: false,
                message: "Title and content are required".to_string(),
                count: 0,
            }),
        );
    }

    let message = NotifyMessage::new(req.title, req.content);
    let json = serde_json::to_string(&message).unwrap();
    hub.broadcast(json);

    let count = hub.client_count().await;
    (
        StatusCode::OK,
        Json(NotifyResponse {
            success: true,
            message: "Notification sent".to_string(),
            count,
        }),
    )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(hub): State<Arc<Hub>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, hub))
}

async fn handle_socket(socket: WebSocket, hub: Arc<Hub>) {
    let client_id = format!("client-{}", uuid::Uuid::new_v4());
    let mut rx = hub.add_client(client_id.clone()).await;

    let (mut sender, mut receiver) = socket.split();

    // 发送任务: 从 broadcast channel 发送到 WebSocket
    let sender_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // 接收任务: 从 WebSocket 读取 (目前不处理客户端消息)
    let receiver_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // Log or handle client messages if needed
            let _ = msg;
        }
    });

    tokio::select! {
        _ = sender_task => {},
        _ = receiver_task => {},
    }

    hub.remove_client(&client_id).await;
}
```

- [ ] **Step 4: 运行测试**

需要添加 `uuid` 依赖，然后运行测试

- [ ] **Step 5: 提交**

```bash
git add apps/notifie-server/src/handler/ apps/notifie-server/Cargo.toml
git commit -m "feat(server): add HTTP handlers"
```

---

### Task 6: 创建 handler/mod.rs

**Files:**
- Create: `apps/notifie-server/src/handler/mod.rs`

- [ ] **Step 1: 写模块**

```rust
pub mod notify;

pub use notify::NotifyHandler;
```

- [ ] **Step 2: 提交**

```bash
git add apps/notifie-server/src/handler/mod.rs
git commit -m "feat(server): add handler module"
```

---

## Chunk 4: Main Entry

### Task 7: 创建 main.rs

**Files:**
- Create: `apps/notifie-server/src/main.rs`

- [ ] **Step 1: 写入口代码**

```rust
use std::sync::Arc;
use notifie_server::hub::Hub;
use notifie_server::handler::NotifyHandler;

#[tokio::main]
async fn main() {
    // 创建 Hub
    let hub = Arc::new(Hub::new());

    // 创建 Handler
    let handler = NotifyHandler::new(hub.clone());

    // 构建 Router
    let app = handler.router();

    // 添加中间件
    let app = app.layer(tower_http::cors::CorsLayer::permissive());

    // 监听
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on http://0.0.0.0:8080");

    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 2: 创建 lib.rs**

需要创建 `apps/notifie-server/src/lib.rs` 导出模块:

```rust
pub mod handler;
pub mod hub;
pub mod model;
```

- [ ] **Step 3: 编译测试**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo build`
Expected: 成功编译

- [ ] **Step 4: 运行测试服务器**

Run: `cargo run`
Expected: Server running on http://0.0.0.0:8080

- [ ] **Step 5: 测试 API**

```bash
# Health check
curl http://localhost:8080/health

# Send notification
curl -X POST http://localhost:8080/api/notify \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","content":"Hello"}'
```

- [ ] **Step 6: 提交**

```bash
git add apps/notifie-server/src/
git commit -m "feat(server): add main entry with Axum server"
```

---

## Chunk 5: 集成测试

### Task 8: 端到端测试

**Files:**
- Test: `apps/notifie-server/tests/e2e.rs`

- [ ] **Step 1: 创建 e2e 测试**

```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_root() {
    let app = notifie_server::handler::NotifyHandler::new(
        std::sync::Arc::new(notifie_server::hub::Hub::new())
    ).router();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"Notifie Server Running");
}

#[tokio::test]
async fn test_health() {
    let app = notifie_server::handler::NotifyHandler::new(
        std::sync::Arc::new(notifie_server::hub::Hub::new())
    ).router();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_notify() {
    let hub = std::sync::Arc::new(notifie_server::hub::Hub::new());
    let app = notifie_server::handler::NotifyHandler::new(hub.clone()).router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/notify")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"title":"Test","content":"Hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

- [ ] **Step 2: 运行测试**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && cargo test --test e2e`
Expected: PASS

- [ ] **Step 3: 提交**

```bash
git add apps/notifie-server/tests/
git commit -m "test(server): add e2e tests"
```

---

## 完成

**Summary:**
- Task 1-2: model 模块 (消息结构体)
- Task 3-4: hub 模块 (WebSocket 广播)
- Task 5-6: handler 模块 (HTTP 路由)
- Task 7: main 入口 (Axum 服务器)
- Task 8: 端到端测试

**下一步:**
- 删除 Go 版本文件
- 更新 CLAUDE.md
- 测试 WebSocket 连接