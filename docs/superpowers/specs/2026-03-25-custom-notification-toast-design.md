# 自定义通知栏架构设计

**日期:** 2026-03-25
**主题:** 自定义通知栏架构 (Custom Notification Toast)

## 1. 概述

Notifie 系统当前使用系统原生通知栏，存在以下限制：
- 无法固定通知不消失
- 无法实现定时消失
- 无法实现已读同步（一个客户端阅读后其他客户端也消失）

本设计采用**自定义通知栏**方案，通过 Tauri 创建浮动窗口覆盖在桌面顶层，实现完全自定义的通知体验。

## 2. 目标

为跨平台桌面应用（Windows、macOS、Linux）实现自定义通知栏，支持：
- A) 固定不消失 — 通知常驻，直到用户手动关闭
- B) 定时消失 — 可配置 N 秒后自动消失
- C) 已读同步 — 任何一个客户端阅读后，所有客户端的同一通知都消失

## 3. 架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      notifie-server                         │
│                                                             │
│   ┌──────────────┐    ┌──────────────┐    ┌────────────┐  │
│   │   HTTP API   │    │  WebSocket   │    │   Store    │  │
│   │  /api/notify │◄──►│    Hub       │◄──►│ (内存/持久) │  │
│   └──────────────┘    └──────────────┘    └────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                    WebSocket │ JSON
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    notifie-client                           │
│                                                             │
│   ┌──────────────┐    ┌──────────────┐    ┌────────────┐  │
│   │   WebSocket  │───►│  Notify      │───►│   Toast    │  │
│   │   Client     │    │  Manager     │    │   Window   │  │
│   └──────────────┘    └──────────────┘    └────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 组件职责

#### 服务端 (Go + Fiber)

| 组件 | 职责 |
|------|------|
| HTTP API (`/api/notify`) | 接收外部通知请求，生成通知ID，持久化存储 |
| WebSocket Hub | 管理客户端连接，广播通知，已读同步 |
| Store | 内存中存储通知历史，支持按ID查询 |

#### 客户端 (Tauri + React)

| 组件 | 职责 |
|------|------|
| WebSocket Client | 与服务端保持长连接，接收通知/已读广播 |
| Notify Manager | 管理通知状态、窗口实例、已读逻辑 |
| Toast Window | 独立的 Tauri 窗口实例，显示单个通知 |

## 4. 数据结构

### 4.1 通知消息 (Server → Client)

```json
{
  "id": "uuid-xxx",
  "type": "notify",
  "title": "Alert",
  "content": "CPU > 90%",
  "timestamp": 1700000000,
  "duration": 0
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| id | string | 通知唯一标识 UUID |
| type | string | 固定为 "notify" |
| title | string | 通知标题 |
| content | string | 通知内容 |
| timestamp | int64 | Unix 时间戳 |
| duration | int | 显示时长（毫秒），0=永久显示 |

### 4.2 已读消息 (Client → Server)

```json
{
  "type": "read",
  "id": "uuid-xxx"
}
```

### 4.3 已读广播 (Server → All Clients)

```json
{
  "type": "notification_read",
  "id": "uuid-xxx"
}
```

### 4.4 历史记录请求/响应

```json
// Request
{ "type": "history" }

// Response
{
  "type": "history",
  "notifications": [
    { "id": "uuid-1", "title": "...", "content": "...", "timestamp": 1700000000, "duration": 0 }
  ]
}
```

## 5. 消息流程

### 5.1 通知推送流程

```
1. 外部调用 POST /api/notify
2. Server: 生成 UUID，存入 Store
3. Server: 通过 WebSocket 广播给所有在线客户端
4. Client: 收到通知，创建 Toast Window 显示
5. Server: 返回通知ID给调用方
```

### 5.2 已读同步流程

```
1. User: 点击 Client A 的通知
2. Client A: 发送 {type: "read", id: "xxx"} 给 Server
3. Server: 标记通知为已读，从 Store 中移除
4. Server: 广播 {type: "notification_read", id: "xxx"} 给所有客户端
5. Client A/B/C: 收到广播，关闭对应的 Toast Window
```

### 5.3 新客户端连接流程

```
1. Client: 通过 WebSocket 连接到 Server
2. Client: 发送 {type: "history"}
3. Server: 返回所有未读通知列表
4. Client: 为每个未读通知创建 Toast Window
```

## 6. 窗口设计

### 6.1 Toast Window 特性

- **无边框透明窗口** — 浮动在桌面顶层
- **始终置顶** — 不会被其他窗口遮挡
- **右下角堆叠** — 新通知显示在上一条的上方
- **可关闭** — 右上角有关闭按钮
- **可拖拽** — 用户可自由移动位置

### 6.2 窗口通信

```
┌─────────────────┐     Tauri Commands      ┌─────────────────┐
│   React UI      │ ◄─────────────────────► │   Rust Backend  │
│  (通知内容)     │                          │  (窗口管理)     │
└─────────────────┘                          └─────────────────┘
        │                                           │
        │ WebSocket                                  │ 创建/关闭
        ▼                                           ▼
┌─────────────────┐                          ┌─────────────────┐
│   Server       │                          │   OS Window     │
└─────────────────┘                          └─────────────────┘
```

## 7. API 参考

### HTTP 端点

| Method | Path | 说明 |
|--------|------|------|
| POST | /api/notify | 推送新通知 |
| GET | /health | 健康检查 |

### WebSocket 消息

| 方向 | Type | 说明 |
|------|------|------|
| Server → Client | notify | 推送新通知 |
| Server → Client | notification_read | 已读广播 |
| Server → Client | history | 历史通知响应 |
| Client → Server | read | 标记已读 |
| Client → Server | history | 请求历史未读 |

## 8. 实现要点

### 8.1 服务端

- [ ] 扩展 NotifyMessage 添加 id 和 duration 字段
- [ ] 实现通知存储（内存 Map，key 为通知ID）
- [ ] 实现已读标记和广播逻辑
- [ ] 实现历史记录查询

### 8.2 客户端

- [ ] 创建 Toast 窗口模板
- [ ] 实现 WebSocket 重连机制
- [ ] 实现通知管理器（创建/关闭/定位窗口）
- [ ] 实现已读状态同步
- [ ] 实现定时消失逻辑（setTimeout）
- [ ] 实现窗口堆叠定位算法

## 9. 待定事项

- [ ] 服务端持久化方案（当前为内存存储，重启丢失）
- [ ] 通知最大数量限制
- [ ] 窗口样式细节（颜色、字体、阴影）