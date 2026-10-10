---
metadata:
  node_type: memory
name: "Lab Runs - OAuth"
description: "Forced OAuth profile linking: unconsumed code + iframe CSRF → admin link, solved"
last_updated: 2026-10-10T13:29:24+08:00
created: 2026-10-10T13:29:24+08:00
---

## 2026-02-19 lab-oauth-forced-oauth-profile-linking (arm A)

- 实例 https://0a7300490390e81980ded51300f10081.web-security-academy.net/,range_launch reused:false,exploit server https://exploit-0a6200bb03bee8808091d40f01ac003c.exploit-server.net
- 凭证分账(题面):blog wiener:peter;social media peter.wiener:hotdog。首次拿 wiener:peter 打 social provider 得到 "Invalid username/email or password" —— 两套账户不同,别复用。
- 链接流:/my-account 的 Attach a social profile 指向 /auth?...&redirect_uri=.../oauth-linking。手工走完 social 端:/auth → 302 /interaction/<id> → GET 表单 → POST /interaction/<id>/login → GET /auth/<id> 设 _session → GET /interaction/<id>(Authorize 页)→ POST /interaction/<id>/confirm → GET /auth/<id> 出 302 Location=.../oauth-linking?code=<CODE>。
- 关键纪律:http_session 默认不跟 302,所以拿到 Location 里的 code 后**不要**用自己会话取 /oauth-linking(会消费掉 code);同时 server 端不需要 state/CSRF,任何持有会话的人访问该 URL 都会把 code 里的 social profile 挂到自己账户上。
- 投递:exploit server 表单字段是 responseFile(必填且须以 / 开头,如 /exploit)+ responseHead + responseBody + formAction;缺 responseFile 报 "Missing parameter responseFile",空串报 "File must start with /"。payload = <iframe src=".../oauth-linking?code=CODE"></iframe>;STORE 后按 DELIVER_TO_VICTIM(重复三字段,带 --follow,302 /deliver-to-victim → /)。
- 夺权:再走一遍 "Login with social media"(redirect_uri=/oauth-login),social 端已登录直接 302 带 code;/oauth-login?code= 返回 "You have successfully logged in with your social media account" 且页面顶栏出现 /my-account?id=administrator。
- 收尾:GET /admin/delete?username=carlos → 302 /admin,"User deleted successfully!",banner is-solved + "Congratulations, you solved the lab!"(banner_verdict solved:true)。
- 失败点:oauth-server.net 首次 /interaction GET 报 Network is unreachable(os error 101),重试即通 —— 瞬时网络,不是路径错。

