package msg

import "time"

// NotifyMessage 服务器推送给客户端的消息格式
type NotifyMessage struct {
	Type      string `json:"type"`
	Title     string `json:"title"`
	Content   string `json:"content"`
	Timestamp int64  `json:"timestamp"`
}

// NewNotifyMessage 创建新的通知消息
func NewNotifyMessage(title, content string) *NotifyMessage {
	return &NotifyMessage{
		Type:      "notify",
		Title:     title,
		Content:   content,
		Timestamp: time.Now().Unix(),
	}
}