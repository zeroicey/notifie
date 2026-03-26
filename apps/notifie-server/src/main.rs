use std::sync::Arc;
use notifie_server::hub::Hub;
use notifie_server::handler::NotifyHandler;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // 创建 Hub
    let hub = Arc::new(Hub::new());

    // 创建 Handler
    let handler = NotifyHandler::new(hub.clone());

    // 构建 Router
    let app = handler.router();

    // 添加 CORS 中间件
    let app = app.layer(CorsLayer::permissive());

    // 监听
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on http://0.0.0.0:8080");

    axum::serve(listener, app).await.unwrap();
}