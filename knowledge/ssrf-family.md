---
title: SSRF 族:过滤对齐与解析分歧
---

# SSRF 族:过滤对齐与解析分歧

同一类型漏洞:服务端代取客户端可控 URL,过滤(黑/白名单)与实际 fetch
目标之间出现解析分歧,绕过弱防抵达环回/内网。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 黑名单 | 报「blocked for security reasons」 | host 短形 `127.1` + 路径编码 `%61dmin`/大小写 | [[lab-ssrf-with-blacklist-filter]] |
| 白名单 | 报「host must be \<allowed\>」 | `%23`(即 `#`)造成校验/fetch 分歧:`localhost%23@<allowed>/admin` | [[lab-ssrf-with-whitelist-filter]] |

## 共性

1. 判型:回取参数把响应体回注(全读型),报错文案点明过滤类型与允许域。
2. 绕法是「让校验侧与实际 fetch 看到不同 host」:短形/十进制 IP、userinfo、
   `%23`(#)、双编码路径、重定向伪装。
3. 内网横向:路由由 Host 决定时可配合 [[host-header-family]] 的 Host 注入探内网段。

## 判定与收尾要点

- 判定锚点:实际取回内网 admin 面板并删 `carlos`(或完成目标动作),代取被放行不算解。
- 逐变量改:先定 host 绕法,再定路径绕法;用行为差(400 vs 200)归因。

## 相关族

- Host 信任/路由见 [[host-header-family]];盲面出网 oracle 见 [[blind-injection-family]]、[[oob-callback-family]];
  方法论:web-vuln-methods(seed 层,按名引用)。
