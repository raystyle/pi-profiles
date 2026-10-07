---
title: "h2-frontend-sanitizer-matrix"
---

# h2-frontend-sanitizer-matrix

# h2-frontend-sanitizer-matrix

Academy 的 h2 前端(h2→h1 降级)在 request tunnelling 题里**每个注入槽各自的处理**,用 `h2_req` 以同一段种子字节实测(种子 = CRLF CRLF 加一行 `GET /admin HTTP/1.1` 加 `Host: <inst>`):

| 注入槽 | 结果 | 判据形态 |
| --- | --- | --- |
| 头 **值** 里含 CRLF | `RST_STREAM` | 值被拦 |
| 头 **名** 里含 CRLF | `400 {"error":"Invalid request"}` | 名字被拦 |
| 头 **名** 里含裸 LF | `400 {"error":"Newlines in headers are not allowed"}` | 明确文案,可直接读 |
| 头 **名** 含**空格或制表符**(冒号前空格形) | **放行**(请求正常到达,只是过不了 auth 门) | 名字**未**被完整归一化 |
| **`:path` 里含 CRLF** | **放行且原样写进 h1** | 唯一可用的注入面 |

⇒ 实战顺序:先试 `:path`;头名变形(空格/制表)可用来绕名字黑名单,但别指望它单独越权。

## 读通道与记账 oracle(两形态)

1. **读回内层响应原文**:外层 `HEAD <path>` 加 `:path` 注入内层请求,且**内层头块不收尾**(注入不以 CRLF 结束,由前端自己的 CRLF 收尾)⇒ h2 body 直接给出内层响应的原始字节:状态行 + 全部响应头 + 体。用它可以不猜地看到"内层请求实际带着哪些头、返回什么状态"。
2. **期望长度记账**:若内层头块**自己收尾**(注入以 CRLF CRLF 结束),前端立刻回
   `500 Server Error: Received only N of expected M bytes of data`
   其中 M = 外层 path 自身响应长(前端期望长度)、N = 后端实际给出的字节数 ⇒ 这是一个**二值 oracle**:改变注入切法或垫片长度,看 N 是否够 M,就知道装配对不对,不必先猜到成功态。

## 关系

- 方法族:[[h2-tunnelling-and-h2cl-practice]]、[[request-smuggling-family]];题面级数值与帧形见 [[portswigger-platform-specifics]]。
