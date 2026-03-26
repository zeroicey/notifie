use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

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

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hub_new() {
        let hub = Hub::new();
        assert_eq!(hub.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_hub_subscribe() {
        let hub = Hub::new();
        let mut rx = hub.subscribe();
        // 发送消息到 broadcast
        let _ = hub.broadcast("test".to_string());
        // 应该能收到消息
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, "test");
    }
}
