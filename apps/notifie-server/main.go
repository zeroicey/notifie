package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/gofiber/fiber/v3/middleware/logger"
	"github.com/gofiber/fiber/v3/middleware/recover"
	"github.com/fasthttp/websocket"
	"github.com/valyala/fasthttp"
	"github.com/zeroicey/notifie/handler"
	"github.com/zeroicey/notifie/hub"
)

var (
	addr = flag.String("addr", ":8080", "server address")
)

// WebSocket upgrader 配置
var upgrader = websocket.FastHTTPUpgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

// 自定义 WebSocket 处理函数
func wsHandler(c fiber.Ctx, h *hub.Hub) error {
	log.Println("[WS] WebSocket handler called")

	// 类型断言获取 fasthttp context
	fctx, ok := c.Context().(*fasthttp.RequestCtx)
	if !ok {
		log.Printf("[WS] Failed to get fasthttp context, got: %T", c.Context())
		return fmt.Errorf("failed to get fasthttp context")
	}

	err := upgrader.Upgrade(fctx, func(conn *websocket.Conn) {
		log.Println("[WS] WebSocket upgrade successful")

		client := &hub.Client{
			ID:   generateClientID(),
			Conn: conn,
			Send: make(chan []byte, 256),
		}

		h.Register <- client

		go client.WritePump()
		go client.ReadPump(h)
	})

	if err != nil {
		log.Printf("[WS] WebSocket upgrade error: %v", err)
		return err
	}

	return nil
}

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
		return wsHandler(c, h)
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
	return fmt.Sprintf("client-%d", time.Now().UnixNano())
}