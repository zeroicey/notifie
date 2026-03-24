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
	if err := c.Bind().Body(&req); err != nil {
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