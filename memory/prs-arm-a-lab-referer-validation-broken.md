---
metadata:
  node_type: memory
name: "PRS arm A lab-referer-validation-broken"
description: "arm A baseline solved: lab-referer-validation-broken CSRF — naive substring Referer check bypassed via pushState + Referrer-Policy unsafe-url on the exploit page"
last_updated: 2026-10-09T07:40:09+08:00
created: 2026-10-09T07:40:09+08:00
---

### 2026-10-09 arm A baseline - lab-referer-validation-broken

- 实例: range_launch launch-url `/web-security/csrf/bypassing-referer-based-defenses/lab-referer-validation-broken` -> reused:false, instance 0a1a00a7...web-security-academy.net, exploit server exploit-0a8f0068...exploit-server.net (banner 里 `#exploit-link` 给,range_launch 的 exploit_server 字段为 null 是漏报)。
- 目标面: POST /my-account/change-email, body 仅 `email=`,**无 CSRF token**,唯一防线是 Referer 校验;login POST 同样吃 Referer 校验(无 Referer -> 400 `"Invalid referer header"`),故脚本驱动登录也必须补 Referer。
- 校验缺陷(实测): 裸子串包含判断。POST change-email 带 `Referer: https://evil.com/?<lab-domain>` 直接 302 -> /my-account?id=wiener,即异源 Referer 只要串里出现 lab 域名就通过。
- 利用链: exploit server STORE `/exploit`, responseHead 加 `Referrer-Policy: unsafe-url`(默认 strict-origin-when-cross-origin 会把跨源 Referer 压成裸 origin,域名就进不去), body = `history.pushState('', '', '/?<lab-domain>')` 把域名塞进当前 URL 的 query -> 表单 POST 到 change-email -> auto submit。DELIVER_TO_VICTIM(--follow) 后回执页直接 `is-solved`。
- 复核: banner_verdict 首页 solved:true + congrats 行(`/home` 是 404,别当根路径用)。
- 件: range_launch / http_session(STORE+DELIVER 全能) / banner_verdict,未新写件。
- 一次通过(reused:false),无返工。

