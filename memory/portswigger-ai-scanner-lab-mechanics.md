---
metadata:
  node_type: memory
name: "PortSwigger AI scanner lab mechanics"
description: "AI 扫描器 lab 的端点/报告面/评论字段与限流实测事实"
last_updated: 2026-10-11T02:11:17+08:00
created: 2026-10-11T02:11:17+08:00
---

## 环境机制(实测)
- 扫描触发: POST /api/audit/start, body {"postId":"1"} 或 {"productId":...} → {"status":"started"}; 扫描中再触发返回 429 {"error":"Your AI-powered scanner is busy right now...","status":"rate_limited"}。
- 状态面: GET /api/audit/status → {"status":running|completed,"currentTurn":N,"maxTurns":15}; WS /api/audit/stream (wss) 只推同样的状态帧,无 transcript。
- 报告面: 扫描完成后,扫描器把报告作为**博客评论**发到被扫的 post 上,作者名 "Security Scanner"/"Security Audit"/"Security Assessment Team" 等(每次措辞都不同,LLM 生成),内容几乎固定为 /accountDetails 的 CORS 误配置。
- 敏感数据: GET /accountDetails(带会话)返回 {"username","email","apikey","sessions"}; /my-account 页由 JS fetch /accountDetails 填充 #apikey。
- 评论字段: comment/name/email/website; name 上限 64 字符(超长 400 "Name must be length 64 or less.");comment 正文 HTML 转义(尖括号变实体,无法注入标签/HTML 注释)。
- 其它: /my-account?id=carlos → 302 /login(无 IDOR);/accountDetails 忽略 user/username/id 参数;参数无反射;/post?postId=1&x=... 不回显;本实例无 exploit server(exploit-<id>.exploit-server.net → 421 Invalid host)。

