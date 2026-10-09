---
metadata:
  node_type: memory
name: "PRS arm A lab-2fa-broken-logic"
description: "arm A baseline solved lab-2fa-broken-logic: verify cookie → carlos code brute (1915) via new code_brute piece; ureq redirects caused a false positive"
last_updated: 2026-10-08T19:31:58+08:00
created: 2026-10-08T19:31:58+08:00
---

## 2026-10-08 arm A 基线:lab-2fa-broken-logic solved

- instance: https://0a0a002404b9929080cdae9800a100e2.web-security-academy.net (range_launch reused:false,jar /tmp/cj1.json)
- 路径: page_read 取 widget-lab-id(注意真正 slug 在 /authentication/multi-factor/,不在 /authentication/mfa/)→ POST /login wiener:peter → 302 /login2 + Set-Cookie verify=wiener → jar 改 verify=carlos → GET /login2(为 carlos 生成码,邮件投给 carlos 不可读)→ 枚举码 → 命中 1915 → 302 /my-account?id=carlos + 新 session → /my-account "Your username is: carlos" → banner is-solved。
- 新件: `.pi-rs/rust-scripts/code_brute.rs`(项目层,1.0.0)- 数字口令并行枚举(N 线程零填充 POST 体字段,3xx 或正文缺 --fail-marker 即命中,命中后写回 jar 并复验 --confirm-path)。首轮假阳性根因 = ureq 默认跟跳转,302 后带旧 Cookie 落到 /login 的 200 页;件已固定 `.redirects(0)`。
- 会话卫生: 假命中会把匿名 session 写回 jar;重跑枚举前必须重走 POST /login 重建待验会话。GET /login2 在未登录会话下也回 200 表单,不能作为缺陷面证据。

