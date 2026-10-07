---
metadata:
  node_type: memory
name: "Regression Batch R1 - already-solved labs re-run"
description: "回归批 R1(4 题冷会话重解):H2.CL / CSP-dangling / GraphQL 三题 solved,capture-other-users 走私链通过但捕获的 victim session 匿名(账号步未收口);lab 发射钥匙 = portswigger.net .AspNetCore.CookiesC1/C2(存在 /tmp/cj1.json),auth0 已过期;chrome_cookies 默认输出会覆盖 chrome-jar.json"
last_updated: 2026-10-07T18:26:42+08:00
created: 2026-10-07T18:26:42+08:00
---

## 回归批 R1(第五路首跑,4 题已解题重解,冷会话 → congrats)

题目与实例(2026-10-07,全部新开):
1. `request-smuggling/advanced/lab-request-smuggling-h2-cl-request-smuggling` — 实例 `0a3800fc0487a913802d35ce00a600a1`,exploit `exploit-0a8d00e40416a932808934f101a800a6` — **solved**。
2. `request-smuggling/exploiting/lab-capture-other-users-requests` — 实例 `0a6f004c04e8508981183e9c00980092` — **链通过 / 账号步未收口**。
3. `cross-site-scripting/.../lab-very-strict-csp-with-dangling-markup-attack` — 实例 `0a040031042cb88583bb4bfc00b900a9`,exploit `exploit-0ad9007a04feb8ac83994a26017200cf` — **solved**。
4. `graphql/lab-graphql-find-the-endpoint` — 实例 `0a890082030cb669807c3f9000e40038` — **solved**。

结果:#1/#3/#4 从冷会话到 congrats 全链一次通过(件:page_read → range_launch → http_session/objref_scan/h2cl_seq → banner_verdict)。#2 走私链完全复现(捕获受害者完整请求 + 全 cookie),但捕获到的 session 是**匿名**的(`/my-account` 一律 302 /login,单发/带 secret/带 victim-fingerprint/带 Victim UA/h1 与 h2 全试),该实例 victim bot 似未登录 ⇒ 拿不到 administrator;另该实例 `wiener:peter` 登录被拒(#3 同凭据可用)。

关键环境事实(会话底座,重要):
- 驱动 `range_launch` 的钥匙是 **`portswigger.net` 的 `.AspNetCore.CookiesC1/C2` 应用会话**(session cookie、无 exp),**不是** `login.portswigger.net` 的 `auth0`(各 profile DB 里 auth0 exp 均 ≤2026-10-06,已过期;`~/.pi-rs/agent/lab-jar.json` 只有 auth0 → 发射落 Auth0 登录页)。
- 应用会话现存 `/tmp/cj1.json`;每实例先 `cp /tmp/cj1.json /tmp/<lab>-jar.json` 再 `range_launch … --jar`。
- **事故/坑**:`chrome_cookies` 默认输出路径就是 `~/.pi-rs/agent/chrome-jar.json`,不带 `--out` 跑一次即覆盖掉 campaign 会话(chrome-jar.json / lab-jar.json 的 auth0 已失效,别再依赖)。

deficit 校准(可复现):capture 题的受害者请求头块本实例 = **818B**;CL=89+818=907 才截到完整 32 字符 session(815 → 只剩 29 字符)。

未 git 提交;知识库 4 条 record 追加 R1 回归节 + `lab-launch` 增「会话底座」节。

