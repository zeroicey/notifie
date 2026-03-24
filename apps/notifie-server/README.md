# Notifie Server

Go + Fiber WebSocket 通知服务

## 快速开始

```bash
# 安装依赖
go mod tidy

# 运行
go run main.go
```

默认监听 `http://localhost:8080`

## API

### 发送通知

```bash
curl -X POST http://localhost:8080/api/notify \
  -H "Content-Type: application/json" \
  -d '{"title": "告警", "content": "CPU 使用率超过 90%"}'
```

### WebSocket

```
ws://localhost:8080/ws
```

客户端连接此端点接收通知推送。