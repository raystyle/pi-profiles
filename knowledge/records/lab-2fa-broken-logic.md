---
title: "lab-2fa-broken-logic"
links:
  - target: authentication-family
    relation: evidences
---

# lab-2fa-broken-logic

> evidences: [[authentication-family]]

- 题面:2FA broken logic(/web-security/authentication/multi-factor/lab-2fa-broken-logic)
- 实例:https://0a0a002404b9929080cdae9800a100e2.web-security-academy.net
- 判定目标:进入 carlos 的账户页(横幅 is-solved)

## 关键步

1. 侦察:GET /login 表单无 csrf;POST /login(wiener:peter)→ 302 /login2 并 Set-Cookie `verify=wiener`(客户端可控)。
2. 缺陷面直证:jar 里 `verify` 改成 `carlos`,GET /login2 仍回 200 的四位码表单 ⇒ 该 cookie 决定校验对象与最终登录身份,与已认证会话不绑定。
3. 失败面:POST /login2 mfa-code=1234 → 200 且正文含 `Incorrect security code`(body 3184);登录表单页 body 3116。
4. 取码通道:GET /login2(verify=carlos)为 carlos 生成安全码并投递到 carlos 的邮箱(不在 wiener 的 email client 视野内)⇒ 只能枚举。
5. 枚举:项目层新件 `code_brute` 以 16 线程零填充 0000-9999 POST /login2(不跟跳转),`--fail-marker Incorrect`,命中 code=1915 → 302 `Location: /my-account?id=carlos` 并 Set-Cookie 新 session;件把该 cookie 写回 jar,自动复验 /my-account。
6. 判定:banner_verdict → `solved_class=true` 且 `<h4>Congratulations, you solved the lab!</h4>`。

## 坑

- ureq 默认跟跳转:命中响应是 302 时,跟到 /my-account 会带旧 Cookie(ureq 不维护 cookie 罐),最终落到 /login 的 200 页 ⇒ 真 session 丢失、200 页被误判命中。件已改 `.redirects(0)` 读原始 302 与其 Set-Cookie。
- 假命中会把匿名 session 写回 jar 污染会话;重跑前必须重走 POST /login(wiener)重建待验会话。
- 未登录会话直打 /login2 也回 200 表单,不能当缺陷面证据;判据看 POST 后的状态码。

## 证据摘录

- POST /login2(verify=carlos,mfa-code=1915) → 302,`Location: /my-account?id=carlos`,`Set-Cookie: session=c0pIjpggCoaUslz71Gd5LK3jG5QXkjKU`
- GET /my-account(该 session) → 200,`Your username is: carlos`
- 首页横幅 → `<h4>Congratulations, you solved the lab!</h4>`
