---
title: OAuth 族:redirect_uri 与令牌外带
---

# OAuth 族:redirect_uri 与令牌外带

同一类型漏洞:OAuth 授权码/令牌回落点的 `redirect_uri` 校验可绕,令牌被
代理页/postMessage 转交攻击者。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 代理页偷令牌 | implicit 流,redirect_uri 前缀匹配可穿越 | 前缀校验的回调路径穿越到站内代理页;代理页把完整 href(含 fragment token)抛给任意来源,收者以令牌换目标接口凭据(精确路径随 records) | [[lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page]] |
| redirect_uri 处置缺陷 | 四档:不校验/前缀或穿越/允许域挂开放重定向/`@` 与参数污染 | 授权码在 query 里随 302 走,落点可控即偷码 | |
| 授权码拦截换票 | code 多为一次性 | 抢在合法客户端之前把截获的 code 打到 token 端点 | |
| 缺 state 强制绑定 | 授权请求无 state | 登录 CSRF/把攻击者账号绑到受害者(state 防绑定,PKCE 防截码,两回事) | |
| implicit 身份未绑定 | 客户端信用户提交的 email 不向提供方核对主体 | 以 email 归属冒充注册 | |
| 动态注册 SSRF | `/.well-known/openid-configuration` 指出 `/reg`,注册免认证 | 授权服务器会去取 `logo_uri`/`jwks_uri`/`sector_identifier_uri`(RFC 7591),可打云元数据 | |

## 共性

1. implicit 流令牌以 **fragment** 回落(`#access_token=`),不随请求走,须靠页内 JS 转发。
2. redirect_uri 校验常只做前缀/字面比对,路径穿越即可落到同站任意页。
3. fragment 过不了跨源 302:implicit 的 token 在 fragment,须有一个落在允许域上的页面
   (代理页或开放重定向落点)把 `location` 交出去;OAuth 2.1 已废弃 implicit 流。
3. 拿 token 后打 `/me` 换 apikey,再用图片请求外带到 exploit server `/log`。

## 判定与收尾要点

- 判定锚点:读到受害者令牌/apikey 并闭环使用;`/submitSolution` 或登录为真。

## 相关族

- 利用页依赖 web message / DOM 面见 [[dom-xss-family]];方法论:web-vuln-methods(seed 层,按名引用)。

## 工具面与族地板带

- 工具面:`page_read`(题面/端点)、`http_session`(授权流往返)、`range_launch`(实例)、exploit-server 存投递页用 `page_read`+`http_session` 表单;令牌/签名铸造段对口 payload-forger 专家(构造返回不投递)。
- 族地板带:15-27,中位 23(贴线族:6 题中 4 题压 24 线内;state 绑定与令牌外带子型贴线)。
