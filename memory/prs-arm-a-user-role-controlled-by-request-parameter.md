---
metadata:
  node_type: memory
name: "PRS arm A user-role-controlled-by-request-parameter"
description: "arm A 基线:lab-user-role-controlled-by-request-parameter 冷实例一次通过 - Admin=false→true cookie 伪造进 /admin 删 carlos,banner solved"
last_updated: 2026-10-08T18:30:35+08:00
created: 2026-10-08T18:30:35+08:00
---

冷实例一次通过(lab-user-role-controlled-by-request-parameter,reused:false)。

链路:page_read 取 widget-lab-id → range_launch(jar /tmp/cj1.json) → GET /login 取 csrf → POST /login(wiener:peter,302 /my-account?id=wiener,响应 Set-Cookie 里可见 `Admin=false`)→ http_dump 不带 jar、手写 `Cookie: Admin=true; session=<登录后的新 session>` 打 /admin(200,管理员面板)→ 面板内 `/admin/delete?username=carlos` → 302 Location:/admin → banner_verdict solved:true。

可复用点:
- 登录后 session 会被轮换,必须用 302 响应里新发的 session 值,不能用 GET /login 时那个。
- http_dump 不带 --jar 时只发 --header 里的 Cookie,是"自己拼整条 Cookie 头"的干净通道;要替换 jar 里某个 cookie 值时优先用它而不是 http_session(后者会带上 jar 里的 Admin=false)。
- 该族题面自述"admin 面板用可伪造 cookie 识别管理员",响应里直接给出 `Admin=false; Secure; HttpOnly` 就是信号面。

