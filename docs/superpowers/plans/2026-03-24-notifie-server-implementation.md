# Notifie Server Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Go + Fiber 服务端，支持 WebSocket 长连接和 HTTP API 广播通知

**Architecture:**
- 使用 Fiber v3 创建 HTTP 服务器和 WebSocket 升级处理
- 实现 Hub 模式管理所有 WebSocket 客户端连接
- 每个客户端分配唯一 UUID，支持广播和单点发送

**Tech Stack:**
- Go 1.26
- Fiber v3 (github.com/gofiber/fiber/v3)
- fasthttp/websocket (github.com/fasthttp/websocket)
- google/uuid (客户端 ID)

---

## Chunk 1: 项目基础结构

### Task 1: 创建消息结构定义

**Files:**
- Create: `apps/notifie-server/msg/message.go`

- [ ] **Step 1: 创建消息结构文件**

```go
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
```

- [ ] **Step 2: 验证代码编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && go build ./...`
Expected: 无错误

- [ ] **Step 3: Commit**

```bash
git add apps/notifie-server/msg/message.go
git commit -m "feat(server): add message structure"
```

---

### Task 2: 创建 WebSocket Hub

**Files:**
- Create: `apps/notifie-server/hub/client.go`

- [ ] **Step 1: 创建 Hub 和 Client 结构**

```go
package hub

import (
	"encoding/json"
	"log"
	"sync"
	"time"

	"github.com/fasthttp/websocket"
	"github.com/google/uuid"
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
	Unregister chan *client
	mu         sync.RWMutex
}

// NewHub 创建新的 Hub
func NewHub() *Hub {
	return &Hub{
		Clients:    make(map[string]*Client),
		Broadcast:  make(chan []byte, 256),
		Register:   make(chan *Client),
		Unregister: make(chan *client),
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
```

- [ ] **Step 2: 修复 struct 错误**

注意：`Unregister` 通道类型错误，应该是 `*Client` 而不是 `*client`

- [ ] **Step 3: 验证代码编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && go build ./...`
Expected: 无错误

- [ ] **Step 4: Commit**

```bash
git add apps/notifie-server/hub/client.go
git commit -m "feat(server): add WebSocket hub"
```

---

## Chunk 2: HTTP Handler

### Task 3: 创建通知 Handler

**Files:**
- Create: `apps/notifie-server/handler/notify.go`

- [ ] **Step 1: 创建 HTTP Handler**

```go
package handler

import (
	"github.com/gofiber/fiber/v3"
	"github.com/zeroicey/notifie/hub"
	"github.com/zeroicey/notifie/msg"
)

// NotifyRequest HTTP 请求体
type NotifyRequest struct {
	Title   string `json:"title"`
	Content string `json:"content"`
}

// NotifyResponse HTTP 响应体
type NotifyResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
	Count   int    `json:"count"`
}

// NotifyHandler 处理通知请求
type NotifyHandler struct {
	Hub *hub.Hub
}

// NewNotifyHandler 创建新的 NotifyHandler
func NewNotifyHandler(h *hub.Hub) *NotifyHandler {
	return &NotifyHandler{Hub: h}
}

// HandleNotify 处理 POST /api/notify
func (h *NotifyHandler) HandleNotify(c fiber.Ctx) error {
	var req NotifyRequest
	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"success": false,
			"message": "Invalid request body",
		})
	}

	if req.Title == "" || req.Content == "" {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"success": false,
			"message": "Title and content are required",
		})
	}

	// 创建消息并广播
	message := msg.NewNotifyMessage(req.Title, req.Content)
	h.Hub.BroadcastMessage(message)

	return c.JSON(NotifyResponse{
		Success: true,
		Message: "Notification sent",
		Count:   h.Hub.ClientCount(),
	})
}

// HandleHealth 健康检查
func (h *NotifyHandler) HandleHealth(c fiber.Ctx) error {
	return c.JSON(fiber.Map{
		"status":  "ok",
		"clients": h.Hub.ClientCount(),
	})
}
```

- [ ] **Step 2: 验证代码编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && go build ./...`
Expected: 无错误

- [ ] **Step 3: Commit**

```bash
git add apps/notifie-server/handler/notify.go
git commit -m "feat(server): add HTTP notify handler"
```

---

## Chunk 3: 主入口程序

### Task 4: 创建 main.go

**Files:**
- Create: `apps/notifie-server/main.go`

- [ ] **Step 1: 创建主程序**

```go
package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/gofiber/fiber/v3/middleware/logger"
	"github.com/gofiber/fiber/v3/middleware/recover"
	"github.com/fasthttp/websocket"
	"github.com/zeroicey/notifie/handler"
	"github.com/zeroicey/notifie/hub"
)

var (
	addr = flag.String("addr", ":8080", "server address")
)

func main() {
	flag.Parse()

	// 创建 Hub
	h := hub.NewHub()
	go h.Run()

	// 创建 Fiber 应用
	app := fiber.New(fiber.Config{
		AppName: "notifie-server",
	})

	// 中间件
	app.Use(recover.New())
	app.Use(logger.New())
	app.Use(cors.New())

	// 创建 Handler
	notifyHandler := handler.NewNotifyHandler(h)

	// 路由
	app.Get("/", func(c fiber.Ctx) error {
		return c.SendString("Notifie Server Running")
	})

	app.Get("/health", notifyHandler.HandleHealth)
	app.Post("/api/notify", notifyHandler.HandleNotify)

	// WebSocket 路由
	app.Get("/ws", func(c fiber.Ctx) error {
		if !websocket.IsWebSocketUpgrade(c) {
			return c.Status(fiber.StatusUpgradeRequired).SendString("Requires WebSocket upgrade")
		}

		conn, err := websocket.Upgrade(c.Response().StdWriter(), c.Request(), nil, 1024, 0)
		if err != nil {
			log.Printf("WebSocket upgrade error: %v", err)
			return err
		}

		client := &hub.Client{
			ID:   generateClientID(),
			Conn: conn,
			Send: make(chan []byte, 256),
		}

		h.Register <- client

		go client.WritePump()
		go client.ReadPump(h)

		return nil
	})

	// 优雅关闭
	go func() {
		sigCh := make(chan os.Signal, 1)
		signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
		<-sigCh
		log.Println("Shutting down...")
		app.Shutdown()
	}()

	log.Printf("Server starting on %s", *addr)
	if err := app.Listen(*addr); err != nil {
		log.Fatalf("Server error: %v", err)
	}
}

func generateClientID() string {
	return "client-" + fmt.Sprintf("%d", time.Now().UnixNano())
}
```

- [ ] **Step 2: 修复缺失的 import**

需要添加 `"time"` 和 `"fmt"` 包

- [ ] **Step 3: 验证代码编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && go build ./...`
Expected: 无错误

- [ ] **Step 4: Commit**

```bash
git add apps/notifie-server/main.go
git commit -m "feat(server): add main entry point"
```

---

## Chunk 4: 客户端读写Pump

### Task 5: 添加客户端 Read/Write Pump

**Files:**
- Modify: `apps/notifie-server/hub/client.go`

- [ ] **Step 1: 添加 ReadPump 方法**

```go
// ReadPump 处理从客户端读取消息
func (c *Client) ReadPump(h *Hub) {
	defer func() {
		h.Unregister <- c
		c.Conn.Close()
	}()

	for {
		_, message, err := c.Conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("WebSocket error: %v", err)
			}
			break
		}

		// 目前服务端不需要处理客户端发来的消息
		// 预留扩展接口
		log.Printf("Received from %s: %s", c.ID, string(message))
	}
}
```

- [ ] **Step 2: 添加 WritePump 方法**

```go
// WritePump 处理向客户端写入消息
func (c *Client) WritePump() {
	defer c.Conn.Close()

	for {
		message, ok := <-c.Send
		if !ok {
			c.Conn.WriteMessage(websocket.CloseMessage, []byte{})
			return
		}

		if err := c.Conn.WriteMessage(websocket.TextMessage, message); err != nil {
			return
		}
	}
}
```

- [ ] **Step 3: 验证代码编译**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && go build ./...`
Expected: 无错误

- [ ] **Step 4: Commit**

```bash
git add apps/notifie-server/hub/client.go
git commit -m "feat(server): add client read/write pump"
```

---

## Chunk 5: 集成测试

### Task 6: 测试服务端

**Files:**
- Test: `apps/notifie-server/`

- [ ] **Step 1: 启动服务端**

Run: `cd /home/oicey/projects/notifie/apps/notifie-server && go run main.go &`
Expected: 服务启动，日志显示 "Server starting on :8080"

- [ ] **Step 2: 测试健康检查**

Run: `curl http://localhost:8080/health`
Expected: `{"status":"ok","clients":0}`

- [ ] **Step 3: 测试 HTTP 通知接口**

Run: `curl -X POST http://localhost:8080/api/notify -H "Content-Type: application/json" -d '{"title":"Test","content":"Hello"}'`
Expected: `{"success":true,"message":"Notification sent","count":0}`

- [ ] **Step 4: 测试 WebSocket 连接**

使用 websocat 或 wscat 测试: `websocat ws://localhost:8080/ws`

- [ ] **Step 5: 停止服务端**

Run: `pkill -f "go run main.go"` 或 `pkill -f notifie-server`

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "test(server): verify server functionality"
```

---

## 完成

**Summary:**
- 创建消息结构 (`msg/message.go`)
- 实现 WebSocket Hub (`hub/client.go`)
- 创建 HTTP Handler (`handler/notify.go`)
- 实现 main 入口 (`main.go`)
- 添加客户端读写 Pump
- 验证服务端功能

**下一步:**
- 实现 notifie-client (Tauri + React)