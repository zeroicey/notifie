package hub

import (
	"encoding/json"
	"log"
	"sync"

	"github.com/fasthttp/websocket"
	"github.com/zeroicey/notifie/msg"
)

// Client WebSocket 客户端
type Client struct {
	ID   string
	Conn *websocket.Conn
	Send chan []byte
}

// Hub 管理所有客户端连接
type Hub struct {
	Clients    map[string]*Client
	Broadcast  chan []byte
	Register   chan *Client
	Unregister chan *Client
	mu         sync.RWMutex
}

// NewHub 创建新的 Hub
func NewHub() *Hub {
	return &Hub{
		Clients:    make(map[string]*Client),
		Broadcast:  make(chan []byte, 256),
		Register:   make(chan *Client),
		Unregister: make(chan *Client),
	}
}

// Run 启动 Hub
func (h *Hub) Run() {
	for {
		select {
		case client := <-h.Register:
			h.mu.Lock()
			h.Clients[client.ID] = client
			h.mu.Unlock()
			log.Printf("Client connected: %s, total: %d", client.ID, len(h.Clients))

		case client := <-h.Unregister:
			h.mu.Lock()
			if _, ok := h.Clients[client.ID]; ok {
				delete(h.Clients, client.ID)
				close(client.Send)
			}
			h.mu.Unlock()
			log.Printf("Client disconnected: %s, total: %d", client.ID, len(h.Clients))

		case message := <-h.Broadcast:
			h.mu.RLock()
			for _, client := range h.Clients {
				select {
				case client.Send <- message:
				default:
					close(client.Send)
					delete(h.Clients, client.ID)
				}
			}
			h.mu.RUnlock()
		}
	}
}

// BroadcastMessage 广播消息给所有客户端
func (h *Hub) BroadcastMessage(m *msg.NotifyMessage) {
	data, err := json.Marshal(m)
	if err != nil {
		log.Printf("Failed to marshal message: %v", err)
		return
	}
	h.Broadcast <- data
}

// ClientCount 返回客户端数量
func (h *Hub) ClientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.Clients)
}

// ReadPump 读取客户端消息
func (c *Client) ReadPump(h *Hub) {
	defer func() {
		h.Unregister <- c
		if c.Conn != nil {
			c.Conn.Close()
		}
	}()

	for {
		if c.Conn == nil {
			break
		}
		_, message, err := c.Conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("WebSocket error: %v", err)
			}
			break
		}
		// 处理客户端消息（如果有需要）
		log.Printf("Received from %s: %s", c.ID, string(message))
	}
}

// WritePump 向客户端写入消息
func (c *Client) WritePump() {
	defer func() {
		if c.Conn != nil {
			c.Conn.Close()
		}
	}()

	for {
		message, ok := <-c.Send
		if !ok {
			if c.Conn != nil {
				c.Conn.WriteMessage(websocket.CloseMessage, []byte{})
			}
			return
		}
		if c.Conn == nil {
			return
		}
		if err := c.Conn.WriteMessage(websocket.TextMessage, message); err != nil {
			log.Printf("Write error: %v", err)
			return
		}
	}
}