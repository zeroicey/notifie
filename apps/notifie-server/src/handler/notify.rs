use crate::hub::Hub;
use crate::model::{HealthResponse, NotifyMessage, NotifyRequest, NotifyResponse};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
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

async fn notify(State(hub): State<Arc<Hub>>, Json(req): Json<NotifyRequest>) -> impl IntoResponse {
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
    ws: axum::extract::ws::WebSocketUpgrade,
    State(hub): State<Arc<Hub>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, hub))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, hub: Arc<Hub>) {
    use futures_util::{SinkExt, StreamExt};

    let client_id = format!("client-{}", uuid::Uuid::new_v4());
    let mut rx = hub.add_client(client_id.clone()).await;

    let (mut sender, mut receiver) = socket.split();

    // 发送任务: 从 broadcast channel 发送到 WebSocket
    let sender_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender
                .send(axum::extract::ws::Message::Text(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // 接收任务: 从 WebSocket 读取 (目前不处理客户端消息)
    let receiver_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Log or handle client messages if needed
        }
    });

    tokio::select! {
        _ = sender_task => {},
        _ = receiver_task => {},
    }

    hub.remove_client(&client_id).await;
}
