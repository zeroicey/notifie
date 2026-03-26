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
            timestamp: Utc::now().timestamp(),
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