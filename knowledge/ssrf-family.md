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
| 本机/内网全读 | 响应体回显到页面 | 先打 `localhost` 再扫内网段(`192.168.0.0/24` 逐末字节) | |
| 开放重定向绕允许名单 | 允许名单只看见本站路径 | fetch 跟 302 走:允许域上的重定向端点转投内网目标 | |
| 盲 SSRF(OOB) | 无响应体回显 | 探测哪个头(Referer 类)被服务端取用;证据在带外(先 HTTP 回调,出不去只剩 DNS) | |
| 盲打内网+二段漏洞 | 盲请求触达内网服务 | 内网服务把某请求头当命令执行(如 Shellshock 形),结果经 DNS 回带 | |

## 共性

1. 判型:回取参数把响应体回注(全读型),报错文案点明过滤类型与允许域。
2. 绕法是「让校验侧与实际 fetch 看到不同 host」:短形/十进制 IP、userinfo、
   `%23`(#)、双编码路径、重定向伪装。
- URL parser 差异表(Orange Tsai 2017 谱):`@` userinfo、`#`、反斜杠、以及「校验侧与
  fetch 谁先解码」的次序差,任一处分歧即绕过面。
- scheme 与云面(HTTP 不出去时):`file`/`gopher`/`dict` 离开 HTTP;DNS rebinding(校验时
  解析允许域、取数时落环回);云元数据 IMDSv2 须先取令牌再带头发 GET,裸 GET 回空。
- 带外信道入口:自建回调/OOB 交互面是本族自己的证据通道,勿只当别族工具(裁定:lab 面用
  oastify 形公共交互服务)。
3. 内网横向:路由由 Host 决定时可配合 [[host-header-family]] 的 Host 注入探内网段。

## 判定与收尾要点

- 判定锚点:实际取回内网 admin 面板并删 `carlos`(或完成目标动作),代取被放行不算解。
- 逐变量改:先定 host 绕法,再定路径绕法;用行为差(400 vs 200)归因。


## 工具面

- `raw_matrix`:参数矩阵扫描(内网段扫 `192.168.0.{{V}}:8080` 逐值 1-254;盲打二段面同形)。
- `form_sweep`:过滤端点候选探(黑/白名单题一轮定位弱防御边界)。
- `burp_collab`:OOB 交互信道(盲 SSRF 证据面:DNS/HTTP 回带确认)。

## 族地板带

- 全读/过滤面 5-14(中位 7,全族最廉);盲打二段面 ~35(信道等待主导,非工具循环)。

## 相关族

- Host 信任/路由见 [[host-header-family]];盲面出网 oracle 见 [[blind-injection-family]]、[[oob-callback-family]];
  方法论:web-vuln-methods(seed 层,按名引用)。
