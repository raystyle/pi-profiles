---
title: OAuth 族:redirect_uri 与令牌外带
---

# OAuth 族:redirect_uri 与令牌外带

同一类型漏洞:OAuth 授权码/令牌回落点的 `redirect_uri` 校验可绕,令牌被
代理页/postMessage 转交攻击者。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 代理页偷令牌 | implicit 流,redirect_uri 可绕 | 路径穿越 `/oauth-callback/../post/comment/comment-form`;代理页 `postMessage(location.href,'*')` 抛 fragment token | [[lab-oauth-stealing-oauth-access-tokens-via-a-proxy-page]] |

## 共性

1. implicit 流令牌以 **fragment** 回落(`#access_token=`),不随请求走,须靠页内 JS 转发。
2. redirect_uri 校验常只做前缀/字面比对,路径穿越即可落到同站任意页。
3. 拿 token 后打 `/me` 换 apikey,再用图片请求外带到 exploit server `/log`。

## 判定与收尾要点

- 判定锚点:读到受害者令牌/apikey 并闭环使用;`/submitSolution` 或登录为真。

## 相关族

- 利用页依赖 web message / DOM 面见 [[dom-xss-family]];方法论:web-vuln-methods(seed 层,按名引用)。
