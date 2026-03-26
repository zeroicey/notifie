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
    let hub = std::sync::Arc::new(notifie_server::hub::Hub::new());
    let app = notifie_server::handler::NotifyHandler::new(hub.clone()).router();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(json_str.contains("\"status\":\"ok\""));
    assert!(json_str.contains("\"clients\":0"));
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(json_str.contains("\"success\":true"));
    assert!(json_str.contains("\"message\":\"Notification sent\""));
}

#[tokio::test]
async fn test_notify_invalid_request() {
    let hub = std::sync::Arc::new(notifie_server::hub::Hub::new());
    let app = notifie_server::handler::NotifyHandler::new(hub.clone()).router();

    // Empty title and content
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/notify")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"title":"","content":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(json_str.contains("\"success\":false"));
}

#[tokio::test]
async fn test_websocket_endpoint_exists() {
    let hub = std::sync::Arc::new(notifie_server::hub::Hub::new());
    let app = notifie_server::handler::NotifyHandler::new(hub.clone()).router();

    // WebSocket upgrade request - Axum returns 426 if not proper WS request
    // This is fine - it means the endpoint exists and is configured
    let response = app
        .oneshot(
            Request::builder()
                .uri("/ws")
                .method("GET")
                .header("Upgrade", "websocket")
                .header("Connection", "Upgrade")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                .header("Sec-WebSocket-Version", "13")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Accept both 101 (switching protocols) or 426 (upgrade required - Axum behavior)
    // The important thing is the endpoint exists
    assert!(response.status() == StatusCode::SWITCHING_PROTOCOLS || response.status() == StatusCode::UPGRADE_REQUIRED);
}