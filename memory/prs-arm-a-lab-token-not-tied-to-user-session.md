---
metadata:
  node_type: memory
name: "PRS arm A lab-token-not-tied-to-user-session"
description: "arm A 基线:lab-token-not-tied-to-user-session 冷实例一次通过(wiener 未绑定 token 替 carlos 改邮箱);记三坑=launch 需等构建、submitform 覆盖 --form、DELIVER 后须 GET /deliver-to-victim"
last_updated: 2026-10-09T07:17:23+08:00
created: 2026-10-09T07:17:23+08:00
---

## 2026-10-08T23:2x CSRF token not tied to user session (arm A)

- 题眼:email change 有 csrf token,但不绑会话 ⇒ 拿 wiener 的 token 就能替 carlos 提交。
- 冷实例一次通过,链:page_read 取 widget-lab-id → range_launch → web_fetch 实例根取 exploit server → GET /login 取 csrf → POST /login wiener:peter → GET /my-account 取 token(DiOX…N01F7YL)→ 存 exploit → DELIVER_TO_VICTIM → banner is-solved。
- 载荷:`<form action="https://<lab>/my-account/change-email" method=POST>` + hidden email + hidden csrf=wiener 的 token + `document.forms[0].submit()`。
- 坑 1(发起):range_launch 首射 instance_url=null,final_url 回跳 referrer;widget API 同期回「could not be started in a timely manner」⇒ 实例在构建。nap 25s 后同参重射即 up。不要立刻改参数重试。
- 坑 2(投递):`http_session submitform` 会用文件里的字段**覆盖** --form,且只收 `<input>`,`formAction` 是 `<button>` 收不到 ⇒ 存/投递要走 `http_session post <exploit>/ --form responseFile=/exploit --form responseHead=... --form responseBody=... --form formAction=STORE|DELIVER_TO_VICTIM`(post 路径会把 --form URL-encode)。
- 坑 3(投递收尾):POST DELIVER_TO_VICTIM 回 302 → /deliver-to-victim,**必须再 GET /deliver-to-victim** 才召来模拟受害者;只发 POST 不跟随后,access log 里只有自己(ureq)的请求,无 Victim 行。
- 判据:GET exploit/log 里出现 `Mozilla/5.0 (Victim)` 的 `GET /exploit/`;banner 显示 is-solved。

