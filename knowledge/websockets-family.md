---
title: WebSocket 族:握手面上的 CSRF 与聊天面劫持
---

# WebSocket 族:握手面上的 CSRF 与聊天面劫持

同一类型漏洞:WS 握手只认 cookie,不校验 Origin/无 CSRF token → 跨站页面可为受害者建立会话。
利用面是"用自己的页面代受害者建连并读回消息"。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 跨站 WebSocket 劫持(CSWSH) | `/chat` 表单 `action="wss://host/chat"`,握手只带 session cookie;无 Origin 校验 | 跨站页 `new WebSocket(wss://lab/chat)` + `send('READY')` + 逐帧外传 | [[lab-cross-site-websocket-hijacking]] |

## 共性

1. 先读页面上的 `chat.js`,确认协议(常是"连上先发 `READY`,服务端回历史 JSON")。
2. cookie 必须 `SameSite=None`(跨站 WS 握手才带上);实例响应里可直接看到 `Set-Cookie: session=…; SameSite=None`。
3. 外传用 exploit server 的**任意路径**(`/exfil?d=<data>`;404 也进 access log),`/log` 是环形缓冲,少轮询。
4. lab 可能不给可用登录凭据(CSWSH 题实例的 `wiener:peter` 实测无效),先看 victim 会不会直接给出凭据。

## 判定与收尾要点

- 判定锚点:读回 victim 的聊天内容(常含密码)→ 用它登录 → `solved_check` true。
- 外传证据 = exploit server `/log` 里带 victim UA 的 `/exfil` 请求。

## 相关族

- 会话 cookie 面见 [[csrf-family]];同属"受害者本尊请求"的点击劫持见 [[clickjacking-family]]。
- 工具:内置 `ws_chat`(直接以 jar 会话发/收 WS 帧)、`lab_http`(exploit server 投递与 `/log` 读取)。
